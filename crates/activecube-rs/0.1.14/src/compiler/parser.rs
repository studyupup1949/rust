use std::collections::{HashMap, HashSet};

use async_graphql::dynamic::ObjectAccessor;

use crate::compiler::filter;
use crate::compiler::ir::*;
use crate::cube::definition::{CubeDefinition, SelectorDef};
use crate::schema::generator::{
    CalculateRequest, DimAggRequest, FieldAliasMap, QuantileRequest, TimeIntervalRequest,
    metric_key, dim_agg_key,
};

/// Describes a metric requested in the GraphQL selection set.
pub struct MetricRequest {
    pub function: String,
    /// GraphQL alias (e.g. `sum_in` in `sum_in: sum(...)`). Falls back to function name.
    pub alias: String,
    pub of_dimension: String,
    /// The raw selectWhere value extracted from GraphQL arguments.
    pub select_where_value: Option<async_graphql::Value>,
    /// Pre-parsed condition filter for conditional aggregation (countIf/sumIf).
    pub condition_filter: Option<FilterNode>,
}

#[allow(clippy::too_many_arguments)]
pub fn parse_cube_query(
    cube: &CubeDefinition,
    network: &str,
    args: &ObjectAccessor,
    metrics: &[MetricRequest],
    quantiles: &[QuantileRequest],
    calculates: &[CalculateRequest],
    field_aliases: &FieldAliasMap,
    dim_aggs: &[DimAggRequest],
    time_intervals: &[TimeIntervalRequest],
    requested_fields: Option<HashSet<String>>,
) -> Result<QueryIR, async_graphql::Error> {
    let flat = cube.flat_dimensions();
    let requested_cols: Vec<String> = flat.iter()
        .filter(|(path, _)| {
            requested_fields.as_ref().is_none_or(|rf| rf.contains(path))
        })
        .map(|(_, dim)| dim.column.clone())
        .collect();
    let (schema, table) = cube.resolve_table(network, &requested_cols);

    let filters = if let Ok(where_val) = args.try_get("where") {
        if let Ok(where_obj) = where_val.object() {
            filter::parse_where(&where_obj, &cube.dimensions)?
        } else {
            FilterNode::Empty
        }
    } else {
        FilterNode::Empty
    };

    let filters = merge_selector_filters(filters, args, &cube.selectors)?;
    let filters = if let Some(ref chain_col) = cube.chain_column {
        let chain_filter = FilterNode::Condition {
            column: chain_col.clone(),
            op: CompareOp::Eq,
            value: SqlValue::String(network.to_string()),
        };
        if filters.is_empty() {
            chain_filter
        } else {
            FilterNode::And(vec![chain_filter, filters])
        }
    } else {
        filters
    };
    let filters = apply_default_filters(filters, &cube.default_filters);
    let (limit, offset) = parse_limit(args, cube.default_limit, cube.max_limit)?;

    let mut selects: Vec<SelectExpr> = flat
        .iter()
        .filter(|(path, _)| {
            requested_fields
                .as_ref()
                .is_none_or(|rf| rf.contains(path))
        })
        .map(|(_, dim)| SelectExpr::Column {
            column: dim.column.clone(),
            alias: None,
        })
        .collect();

    // Include Array columns when the GraphQL selection requests them.
    // Array dimensions are not in flat_dimensions(), but their parallel
    // ClickHouse columns must appear in SELECT for the resolver to work.
    let array_cols = cube.array_columns();
    if !array_cols.is_empty() {
        let selected_cols: HashSet<String> = selects.iter()
            .filter_map(|s| match s {
                SelectExpr::Column { column, .. } => Some(column.clone()),
                _ => None,
            })
            .collect();
        for (path, col) in &array_cols {
            if selected_cols.contains(col) {
                continue;
            }
            let should_include = requested_fields.as_ref().is_none_or(|rf| {
                // Array path like "Instruction_Program_Arguments_Name".
                // The parent prefix (e.g. "Instruction_Program_Arguments") must
                // be a prefix of at least one requested field path for us to
                // include this column.
                let parent = path.rsplit_once('_').map(|(p, _)| p).unwrap_or(path);
                rf.iter().any(|f| f.starts_with(parent))
            });
            if should_include {
                selects.push(SelectExpr::Column {
                    column: col.clone(),
                    alias: None,
                });
            }
        }
    }

    if selects.is_empty() && !flat.is_empty() && metrics.is_empty() && dim_aggs.is_empty() {
        selects = flat
            .iter()
            .map(|(_, dim)| SelectExpr::Column {
                column: dim.column.clone(),
                alias: None,
            })
            .collect();
    }

    // --- Time interval bucketing: replace raw column with toStartOfInterval ---
    for ti in time_intervals {
        let interval_expr = time_interval_sql(&ti.column, &ti.unit, ti.count);
        let alias = dim_agg_key(&ti.graphql_alias);
        for sel in &mut selects {
            if let SelectExpr::Column { column, alias: ref mut a } = sel {
                if column == &ti.column {
                    *column = interval_expr.clone();
                    *a = Some(alias.clone());
                    break;
                }
            }
        }
    }

    let (filters, agg_having) = split_aggregate_filters(filters);
    let mut group_by = Vec::new();
    let mut having = agg_having;

    // --- Aggregation mode: triggered by metrics, dim aggs, or quantiles ---
    if !metrics.is_empty() || !dim_aggs.is_empty() {
        let agg_columns: HashSet<String> = dim_aggs.iter()
            .map(|da| da.value_column.clone())
            .collect();

        group_by = selects
            .iter()
            .filter_map(|s| match s {
                SelectExpr::Column { column, .. } if !agg_columns.contains(column) => {
                    Some(column.clone())
                }
                _ => None,
            })
            .collect();

        for da in dim_aggs {
            selects.retain(|s| !matches!(s, SelectExpr::Column { column, .. } if column == &da.value_column));
            let alias = dim_agg_key(&da.graphql_alias);
            let condition = da.condition_filter.as_ref().and_then(|f| {
                let sql = compile_filter_inline(f);
                if sql.is_empty() { None } else { Some(sql) }
            });
            let func_name = match da.agg_type {
                DimAggType::ArgMax => "argMax",
                DimAggType::ArgMin => "argMin",
            };

            selects.push(SelectExpr::DimAggregate {
                agg_type: da.agg_type.clone(),
                value_column: da.value_column.clone(),
                compare_column: da.compare_column.clone(),
                alias: alias.clone(),
                condition,
            });

            if let Some(async_graphql::Value::Object(ref obj)) = da.select_where_value {
                let agg_expr = format!("{func_name}(`{}`, `{}`)", da.value_column, da.compare_column);
                let h = parse_select_where_from_value(obj, &agg_expr)?;
                if !h.is_empty() {
                    having = if having.is_empty() { h } else { FilterNode::And(vec![having, h]) };
                }
            }
        }

        for m in metrics {
            let dim_col = flat.iter()
                .find(|(path, _)| path == &m.of_dimension)
                .map(|(_, dim)| dim.column.clone())
                .unwrap_or_else(|| "*".to_string());
            let alias = metric_key(&m.alias);
            let metric_def = cube.find_metric(&m.function);

            if let Some(md) = metric_def.filter(|md| md.expression_template.is_some()) {
                let tmpl = md.expression_template.as_ref().unwrap();
                let expanded = tmpl.replace("{column}", &dim_col);
                selects.push(SelectExpr::Column { column: expanded, alias: Some(alias) });
            } else {
                let func = m.function.to_uppercase();
                let condition = m.condition_filter.as_ref().and_then(|f| {
                    let sql = compile_filter_inline(f);
                    if sql.is_empty() { None } else { Some(sql) }
                });
                selects.push(SelectExpr::Aggregate {
                    function: func.clone(), column: dim_col.clone(),
                    alias: alias.clone(), condition,
                });
                if let Some(async_graphql::Value::Object(ref obj)) = m.select_where_value {
                    let agg_expr = if func == "COUNT" && dim_col == "*" { "COUNT(*)".into() }
                        else if func == "COUNT" || func == "UNIQ" { format!("COUNT(DISTINCT `{dim_col}`)") }
                        else { format!("{func}(`{dim_col}`)") };
                    let h = parse_select_where_from_value(obj, &agg_expr)?;
                    if !h.is_empty() {
                        having = if having.is_empty() { h } else { FilterNode::And(vec![having, h]) };
                    }
                }
            }
        }
    }

    for q in quantiles {
        let dim_col = flat.iter()
            .find(|(path, _)| path == &q.of_dimension)
            .map(|(_, dim)| dim.column.clone())
            .unwrap_or_else(|| "*".to_string());
        let alias = metric_key(&q.alias);
        let expr = format!("quantile({})(`{}`)", q.level, dim_col);
        selects.push(SelectExpr::Column { column: expr, alias: Some(alias) });
        if group_by.is_empty() && !selects.iter().any(|s| matches!(s, SelectExpr::Aggregate { .. })) {
            group_by = selects.iter().filter_map(|s| match s {
                SelectExpr::Column { column, alias } if alias.is_none() && !column.contains('(') => Some(column.clone()),
                _ => None,
            }).collect();
        }
    }

    // --- Build unified allowed_keys from finalized selects (Bitquery pattern) ---
    let allowed_keys = collect_select_keys(&selects, &flat, field_aliases, dim_aggs, time_intervals);

    for calc in calculates {
        let alias = metric_key(&calc.alias);
        let resolved = resolve_calculate_expr(&calc.expression, &allowed_keys);
        selects.push(SelectExpr::Column {
            column: format!("ifNotFinite(({resolved}), 0)"),
            alias: Some(alias),
        });
    }

    ensure_having_columns_in_selects(&having, &mut selects);

    let allowed_keys = collect_select_keys(&selects, &flat, field_aliases, dim_aggs, time_intervals);
    let order_by = parse_order_by(args, cube, &allowed_keys)?;

    // Validate ORDER BY columns are compatible with GROUP BY context.
    // In aggregation mode, ClickHouse requires ORDER BY columns to be either
    // in GROUP BY or be aggregate expressions. Mirrors Ruby activecube behaviour
    // which rejects ordering by fields not present in the query.
    if !group_by.is_empty() && !order_by.is_empty() {
        let group_set: HashSet<&str> = group_by.iter().map(|s| s.as_str()).collect();
        let select_exprs: HashSet<&str> = selects.iter().map(|s| match s {
            SelectExpr::Column { column, .. } => column.as_str(),
            SelectExpr::Aggregate { alias, .. } => alias.as_str(),
            SelectExpr::DimAggregate { alias, .. } => alias.as_str(),
        }).collect();
        for o in &order_by {
            let col = o.column.as_str();
            let is_in_group = group_set.contains(col);
            let is_aggregate = col.contains('(') || select_exprs.contains(col);
            if !is_in_group && !is_aggregate {
                let field_name = flat.iter()
                    .find(|(_, dim)| dim.column == col)
                    .map(|(path, _)| path.as_str())
                    .unwrap_or(col);
                return Err(async_graphql::Error::new(format!(
                    "Cannot order by '{}' in aggregation query — add the field to your selection or order by an aggregated metric instead.",
                    field_name,
                )));
            }
        }
    }

    let limit_by = parse_limit_by(args, cube)?;

    let from_subquery = cube.from_subquery.as_ref().map(|s| {
        s.replace("{schema}", &schema).replace("{chain}", network)
    });

    Ok(QueryIR {
        cube: cube.name.clone(),
        schema,
        table,
        selects,
        filters,
        having,
        group_by,
        order_by,
        limit,
        offset,
        limit_by,
        use_final: cube.use_final,
        joins: Vec::new(),
        custom_query_builder: cube.custom_query_builder.clone(),
        from_subquery,
    })
}

/// Parse a selectWhere value object (from GraphQL Value, not ObjectAccessor)
/// into a HAVING FilterNode.
fn parse_select_where_from_value(
    obj: &indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
    aggregate_expr: &str,
) -> Result<FilterNode, async_graphql::Error> {
    let mut conditions = Vec::new();

    for (key, op) in &[
        ("eq", CompareOp::Eq),
        ("ne", CompareOp::Ne),
        ("gt", CompareOp::Gt),
        ("ge", CompareOp::Ge),
        ("lt", CompareOp::Lt),
        ("le", CompareOp::Le),
    ] {
        if let Some(val) = obj.get(*key) {
            let sql_val = match val {
                async_graphql::Value::String(s) => {
                    if let Ok(f) = s.parse::<f64>() {
                        SqlValue::Float(f)
                    } else {
                        SqlValue::String(s.clone())
                    }
                }
                async_graphql::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        SqlValue::Float(f)
                    } else {
                        SqlValue::Int(n.as_i64().unwrap_or(0))
                    }
                }
                _ => continue,
            };
            conditions.push(FilterNode::Condition {
                column: aggregate_expr.to_string(),
                op: op.clone(),
                value: sql_val,
            });
        }
    }

    Ok(match conditions.len() {
        0 => FilterNode::Empty,
        1 => conditions.into_iter().next().unwrap(),
        _ => FilterNode::And(conditions),
    })
}

fn merge_selector_filters(
    base: FilterNode,
    args: &ObjectAccessor,
    selectors: &[SelectorDef],
) -> Result<FilterNode, async_graphql::Error> {
    let mut extra = Vec::new();

    for sel in selectors {
        if let Ok(val) = args.try_get(&sel.graphql_name) {
            if let Ok(obj) = val.object() {
                let leaf_filters =
                    filter::parse_leaf_filter_for_selector(&obj, &sel.column, &sel.dim_type)?;
                extra.extend(leaf_filters);
            }
        }
    }

    if extra.is_empty() {
        return Ok(base);
    }
    if base.is_empty() {
        return Ok(if extra.len() == 1 {
            extra.remove(0)
        } else {
            FilterNode::And(extra)
        });
    }
    extra.push(base);
    Ok(FilterNode::And(extra))
}

fn apply_default_filters(user_filters: FilterNode, defaults: &[(String, String)]) -> FilterNode {
    if defaults.is_empty() {
        return user_filters;
    }

    let mut default_nodes: Vec<FilterNode> = defaults
        .iter()
        .map(|(col, val)| {
            let sql_val = if val == "true" || val == "false" {
                SqlValue::Bool(val == "true")
            } else if let Ok(n) = val.parse::<i64>() {
                SqlValue::Int(n)
            } else {
                SqlValue::String(val.clone())
            };
            FilterNode::Condition {
                column: col.clone(),
                op: CompareOp::Eq,
                value: sql_val,
            }
        })
        .collect();

    if user_filters.is_empty() {
        if default_nodes.len() == 1 {
            return default_nodes.remove(0);
        }
        return FilterNode::And(default_nodes);
    }

    default_nodes.push(user_filters);
    FilterNode::And(default_nodes)
}

fn parse_limit(
    args: &ObjectAccessor,
    default: u32,
    max: u32,
) -> Result<(u32, u32), async_graphql::Error> {
    let mut limit = default;
    let mut offset = 0u32;

    if let Ok(limit_val) = args.try_get("limit") {
        if let Ok(limit_obj) = limit_val.object() {
            if let Ok(count) = limit_obj.try_get("count") {
                limit = (count.i64()? as u32).min(max);
            }
            if let Ok(off) = limit_obj.try_get("offset") {
                offset = off.i64()? as u32;
            }
        }
    }

    Ok((limit, offset))
}

fn parse_order_by(
    args: &ObjectAccessor,
    cube: &CubeDefinition,
    allowed_keys: &HashMap<String, String>,
) -> Result<Vec<OrderExpr>, async_graphql::Error> {
    let order_val = match args.try_get("orderBy") {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let obj = order_val.object()
        .map_err(|_| async_graphql::Error::new("orderBy must be an object"))?;
    let flat = cube.flat_dimensions();

    if let Ok(field) = obj.try_get("descending") {
        let path = field.enum_name()
            .map_err(|_| async_graphql::Error::new("orderBy.descending must be an enum value"))?;
        let column = flat.iter()
            .find(|(p, _)| p == path)
            .map(|(_, dim)| dim.column.clone())
            .ok_or_else(|| async_graphql::Error::new(format!("Unknown orderBy field: {path}")))?;
        return Ok(vec![OrderExpr { column, descending: true }]);
    }

    if let Ok(field) = obj.try_get("ascending") {
        let path = field.enum_name()
            .map_err(|_| async_graphql::Error::new("orderBy.ascending must be an enum value"))?;
        let column = flat.iter()
            .find(|(p, _)| p == path)
            .map(|(_, dim)| dim.column.clone())
            .ok_or_else(|| async_graphql::Error::new(format!("Unknown orderBy field: {path}")))?;
        return Ok(vec![OrderExpr { column, descending: false }]);
    }

    if let Ok(field_str) = obj.try_get("descendingByField") {
        let name = field_str.string()
            .map_err(|_| async_graphql::Error::new("descendingByField must be a string"))?;
        let column = resolve_field_in_keys(name, allowed_keys)?;
        return Ok(vec![OrderExpr { column, descending: true }]);
    }

    if let Ok(field_str) = obj.try_get("ascendingByField") {
        let name = field_str.string()
            .map_err(|_| async_graphql::Error::new("ascendingByField must be a string"))?;
        let column = resolve_field_in_keys(name, allowed_keys)?;
        return Ok(vec![OrderExpr { column, descending: false }]);
    }

    Ok(vec![])
}

/// Resolve a field reference against the unified allowed_keys registry.
/// Tries the name as-is, then with metric/dim_agg prefixes.
fn resolve_field_in_keys(
    name: &str,
    allowed_keys: &HashMap<String, String>,
) -> Result<String, async_graphql::Error> {
    if let Some(expr) = allowed_keys.get(name) { return Ok(expr.clone()); }
    Err(async_graphql::Error::new(format!(
        "Can't use '{name}' in sorting/ordering. Field not found in executed query."
    )))
}

/// Build unified allowed_keys from finalized selects.
/// Maps user-facing reference names → SQL column/expression.
/// Follows Bitquery convention for `descendingByField`:
///   - `{alias}` for metrics/dim aggs (e.g. "count", "high")
///   - `{parent}_{alias}_{suffix}` for dim aggs (e.g. "BalanceUpdate_Balance_maximum")
///   - `{parent}_{alias}` for time intervals (e.g. "Block_Timefield")
fn collect_select_keys(
    selects: &[SelectExpr],
    flat: &[(String, crate::cube::definition::Dimension)],
    field_aliases: &FieldAliasMap,
    dim_aggs: &[DimAggRequest],
    time_intervals: &[TimeIntervalRequest],
) -> HashMap<String, String> {
    let mut keys = HashMap::new();
    for sel in selects {
        match sel {
            SelectExpr::Column { column, alias: Some(a) } => {
                keys.insert(a.clone(), column.clone());
                if let Some(name) = a.strip_prefix("__da_") {
                    keys.insert(name.to_string(), column.clone());
                } else if let Some(name) = a.strip_prefix("__") {
                    keys.insert(name.to_string(), column.clone());
                }
            }
            SelectExpr::Column { column, alias: None } => {
                if let Some((path, _)) = flat.iter().find(|(_, d)| d.column == *column) {
                    keys.insert(path.clone(), column.clone());
                }
                keys.insert(column.clone(), column.clone());
            }
            SelectExpr::Aggregate { alias, function, column, .. } => {
                let expr = format_agg_sql(function, column);
                keys.insert(alias.clone(), expr.clone());
                if let Some(name) = alias.strip_prefix("__") {
                    keys.insert(name.to_string(), expr);
                }
            }
            SelectExpr::DimAggregate { alias, agg_type, value_column, compare_column, .. } => {
                let expr = format_dim_agg_sql(agg_type, value_column, compare_column);
                keys.insert(alias.clone(), expr.clone());
                if let Some(name) = alias.strip_prefix("__da_") {
                    keys.insert(name.to_string(), expr);
                }
            }
        }
    }
    // Bitquery convention: dim agg references — both with and without _maximum/_minimum suffix
    for da in dim_aggs {
        let suffix = match da.agg_type { DimAggType::ArgMax => "maximum", DimAggType::ArgMin => "minimum" };
        let expr = format_dim_agg_sql(&da.agg_type, &da.value_column, &da.compare_column);
        // {alias}_{suffix} — e.g. "QuotePostAmount_maximum"
        keys.entry(format!("{}_{suffix}", da.graphql_alias)).or_insert_with(|| expr.clone());
        // {field_path}_{suffix} — e.g. "Pool_Quote_PostAmount_maximum"
        keys.entry(format!("{}_{suffix}", da.field_path)).or_insert_with(|| expr.clone());
        if let Some(i) = da.field_path.rfind('_') {
            let parent = &da.field_path[..i];
            // {parent}_{alias}_{suffix} — e.g. "Pool_Quote_QuotePostAmount_maximum"
            keys.entry(format!("{parent}_{}_{suffix}", da.graphql_alias)).or_insert_with(|| expr.clone());
            // {parent}_{alias} (no suffix) — for calculate $variable refs, e.g. "Trade_CurrentPrice"
            keys.entry(format!("{parent}_{}", da.graphql_alias)).or_insert_with(|| expr.clone());
        }
    }
    // Bitquery convention: time interval → {parent}_{alias}
    for ti in time_intervals {
        let expr = time_interval_sql(&ti.column, &ti.unit, ti.count);
        if let Some(i) = ti.field_path.rfind('_') {
            let parent = &ti.field_path[..i];
            keys.entry(format!("{parent}_{}", ti.graphql_alias)).or_insert_with(|| expr);
        }
    }
    for (alias_path, column) in field_aliases {
        keys.entry(alias_path.clone()).or_insert_with(|| format!("`{column}`"));
    }
    keys
}

fn format_agg_sql(function: &str, column: &str) -> String {
    let func = function.to_uppercase();
    let qcol = if column.contains('(') { column.to_string() } else { format!("`{column}`") };
    match (func.as_str(), column) {
        ("COUNT", "*") => "count()".to_string(),
        ("UNIQ", _) => format!("uniq({qcol})"),
        (f, _) => format!("{}({qcol})", f.to_lowercase()),
    }
}

fn format_dim_agg_sql(agg_type: &DimAggType, value_column: &str, compare_column: &str) -> String {
    let func = match agg_type { DimAggType::ArgMax => "argMax", DimAggType::ArgMin => "argMin" };
    let qval = if value_column.contains('(') { value_column.to_string() } else { format!("`{value_column}`") };
    let qcmp = if compare_column.contains('(') { compare_column.to_string() } else { format!("`{compare_column}`") };
    format!("{func}({qval}, {qcmp})")
}

/// Resolve `$variable` references in a calculate expression using allowed_keys.
fn resolve_calculate_expr(expression: &str, allowed_keys: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = expression.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                if let Some(resolved) = allowed_keys.get(&var_name) {
                    let col_ref = if resolved.contains('(') { resolved.clone() } else { format!("`{resolved}`") };
                    result.push_str(&format!("toFloat64({col_ref})"));
                } else {
                    result.push_str(&format!("toFloat64(`{}`)", metric_key(&var_name)));
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn time_interval_sql(column: &str, unit: &str, count: i64) -> String {
    let unit_sql = match unit {
        "seconds" => "SECOND", "minutes" => "MINUTE", "hours" => "HOUR",
        "days" => "DAY", "weeks" => "WEEK", "months" => "MONTH", _ => "MINUTE",
    };
    format!("toStartOfInterval(`{column}`, INTERVAL {count} {unit_sql})")
}

/// Compile a FilterNode into an inline SQL fragment (no parameterized bindings).
/// Used for embedding conditions inside aggregate functions (countIf, sumIf).
fn compile_filter_inline(node: &FilterNode) -> String {
    match node {
        FilterNode::Empty => String::new(),
        FilterNode::Condition { column, op, value } => {
            let col = if column.contains('(') { column.clone() } else { format!("`{column}`") };
            if op.is_unary() {
                return format!("{col} {}", op.sql_op());
            }
            let val_str = match value {
                SqlValue::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                SqlValue::Int(i) => i.to_string(),
                SqlValue::Float(f) => f.to_string(),
                SqlValue::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                SqlValue::Expression(e) => e.clone(),
            };
            match op {
                CompareOp::In | CompareOp::NotIn => {
                    if let SqlValue::String(csv) = value {
                        let items: Vec<String> = csv.split(',')
                            .map(|s| format!("'{}'", s.trim().replace('\'', "\\'")))
                            .collect();
                        format!("{col} {} ({})", op.sql_op(), items.join(", "))
                    } else {
                        format!("{col} {} ({val_str})", op.sql_op())
                    }
                }
                CompareOp::Includes => {
                    let like_val = match value {
                        SqlValue::String(s) => format!("'%{}%'", s.replace('\'', "\\'")),
                        _ => val_str,
                    };
                    format!("{col} LIKE {like_val}")
                }
                _ => format!("{col} {} {val_str}", op.sql_op()),
            }
        }
        FilterNode::And(children) => {
            let parts: Vec<String> = children.iter()
                .map(compile_filter_inline)
                .filter(|s| !s.is_empty())
                .collect();
            match parts.len() {
                0 => String::new(),
                1 => parts.into_iter().next().unwrap(),
                _ => format!("({})", parts.join(" AND ")),
            }
        }
        FilterNode::Or(children) => {
            let parts: Vec<String> = children.iter()
                .map(compile_filter_inline)
                .filter(|s| !s.is_empty())
                .collect();
            match parts.len() {
                0 => String::new(),
                1 => parts.into_iter().next().unwrap(),
                _ => format!("({})", parts.join(" OR ")),
            }
        }
        FilterNode::ArrayIncludes { .. } => {
            // ArrayIncludes is compiled by the SQL dialect, not inline.
            // In conditional aggregation context, this is not expected.
            String::new()
        }
    }
}

/// Walk a HAVING FilterNode and append any referenced aggregate columns that
/// are missing from `selects`. The SQL dialect will assign aliases later.
fn ensure_having_columns_in_selects(having: &FilterNode, selects: &mut Vec<SelectExpr>) {
    let cols = collect_having_columns(having);
    for col in cols {
        if !col.contains('(') {
            continue;
        }
        let already_present = selects.iter().any(|s| match s {
            SelectExpr::Column { column, .. } => column == &col,
            _ => false,
        });
        if !already_present {
            selects.push(SelectExpr::Column {
                column: col,
                alias: None,
            });
        }
    }
}

fn collect_having_columns(node: &FilterNode) -> Vec<String> {
    match node {
        FilterNode::Empty => vec![],
        FilterNode::Condition { column, .. } => vec![column.clone()],
        FilterNode::And(children) | FilterNode::Or(children) => {
            children.iter().flat_map(collect_having_columns).collect()
        }
        FilterNode::ArrayIncludes { array_columns, .. } => array_columns.clone(),
    }
}

/// Detect if a column expression is an aggregate function call.
/// Matches patterns like `argMaxMerge(...)`, `countMerge(...)`, `sumMerge(...)`, etc.
fn is_aggregate_column(column: &str) -> bool {
    column.contains('(')
}

/// Walk a FilterNode tree and split it into (where_part, having_part).
/// Leaf conditions on aggregate columns go to HAVING; everything else stays in WHERE.
fn split_aggregate_filters(node: FilterNode) -> (FilterNode, FilterNode) {
    match node {
        FilterNode::Empty => (FilterNode::Empty, FilterNode::Empty),
        FilterNode::Condition { ref column, .. } => {
            if is_aggregate_column(column) {
                (FilterNode::Empty, node)
            } else {
                (node, FilterNode::Empty)
            }
        }
        FilterNode::And(children) => {
            let mut where_parts = Vec::new();
            let mut having_parts = Vec::new();
            for child in children {
                let (w, h) = split_aggregate_filters(child);
                if !w.is_empty() { where_parts.push(w); }
                if !h.is_empty() { having_parts.push(h); }
            }
            let where_node = match where_parts.len() {
                0 => FilterNode::Empty,
                1 => where_parts.into_iter().next().unwrap(),
                _ => FilterNode::And(where_parts),
            };
            let having_node = match having_parts.len() {
                0 => FilterNode::Empty,
                1 => having_parts.into_iter().next().unwrap(),
                _ => FilterNode::And(having_parts),
            };
            (where_node, having_node)
        }
        FilterNode::Or(children) => {
            let any_aggregate = children.iter().any(filter_has_aggregate);
            if any_aggregate {
                (FilterNode::Empty, FilterNode::Or(children))
            } else {
                (FilterNode::Or(children), FilterNode::Empty)
            }
        }
        FilterNode::ArrayIncludes { .. } => {
            // ArrayIncludes always goes to WHERE, never HAVING
            (node, FilterNode::Empty)
        }
    }
}

fn filter_has_aggregate(node: &FilterNode) -> bool {
    match node {
        FilterNode::Empty => false,
        FilterNode::Condition { column, .. } => is_aggregate_column(column),
        FilterNode::And(children) | FilterNode::Or(children) => {
            children.iter().any(filter_has_aggregate)
        }
        FilterNode::ArrayIncludes { .. } => false,
    }
}

fn parse_limit_by(
    args: &ObjectAccessor,
    cube: &CubeDefinition,
) -> Result<Option<LimitByExpr>, async_graphql::Error> {
    let lb_val = match args.try_get("limitBy") {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let lb_obj = lb_val.object()?;
    let count = lb_obj.try_get("count")?.i64()? as u32;
    let offset = lb_obj
        .try_get("offset")
        .ok()
        .and_then(|v| v.i64().ok())
        .unwrap_or(0) as u32;
    let by_val = lb_obj.try_get("by")?;
    let by_str = by_val.enum_name()?;

    let flat = cube.flat_dimensions();
    let column = flat.iter()
        .find(|(path, _)| path == by_str)
        .map(|(_, dim)| dim.column.clone())
        .ok_or_else(|| async_graphql::Error::new(
            format!("Unknown limitBy field: {by_str}")
        ))?;

    Ok(Some(LimitByExpr { count, offset, columns: vec![column] }))
}
