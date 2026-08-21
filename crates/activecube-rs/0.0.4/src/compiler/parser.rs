use async_graphql::dynamic::ObjectAccessor;

use crate::compiler::filter;
use crate::compiler::ir::*;
use crate::cube::definition::{CubeDefinition, SelectorDef};

/// Describes a metric requested in the GraphQL selection set.
pub struct MetricRequest {
    pub function: String,
    pub of_dimension: String,
    /// The raw selectWhere value extracted from GraphQL arguments.
    pub select_where_value: Option<async_graphql::Value>,
}

pub fn parse_cube_query(
    cube: &CubeDefinition,
    network: &str,
    args: &ObjectAccessor,
    metrics: &[MetricRequest],
) -> Result<QueryIR, async_graphql::Error> {
    let table = cube.table_for_chain(network);

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
    // For tables that use a chain column instead of chain-prefixed table names,
    // inject a WHERE chain = ? filter automatically.
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
    let order_by = parse_order_by(args, cube)?;

    let flat = cube.flat_dimensions();
    let mut selects: Vec<SelectExpr> = flat
        .iter()
        .map(|(_path, dim)| SelectExpr::Column {
            column: dim.column.clone(),
            alias: None,
        })
        .collect();

    let mut group_by = Vec::new();
    let mut having = FilterNode::Empty;

    if !metrics.is_empty() {
        group_by = flat.iter().map(|(_, dim)| dim.column.clone()).collect();

        for m in metrics {
            let dim_col = flat
                .iter()
                .find(|(path, _)| path == &m.of_dimension)
                .map(|(_, dim)| dim.column.clone())
                .unwrap_or_else(|| "*".to_string());

            let func = m.function.to_uppercase();
            let alias = format!("__{}", m.function);

            selects.push(SelectExpr::Aggregate {
                function: func.clone(),
                column: dim_col.clone(),
                alias,
            });

            if let Some(async_graphql::Value::Object(ref obj)) = m.select_where_value {
                let agg_expr = if func == "COUNT" && dim_col == "*" {
                    "COUNT(*)".to_string()
                } else if func == "UNIQ" {
                    format!("COUNT(DISTINCT `{dim_col}`)")
                } else {
                    format!("{func}(`{dim_col}`)")
                };

                let h = parse_select_where_from_value(obj, &agg_expr)?;
                if !h.is_empty() {
                    having = if having.is_empty() {
                        h
                    } else {
                        FilterNode::And(vec![having, h])
                    };
                }
            }
        }
    }

    Ok(QueryIR {
        cube: cube.name.clone(),
        schema: cube.schema.clone(),
        table,
        selects,
        filters,
        having,
        group_by,
        order_by,
        limit,
        offset,
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
) -> Result<Vec<OrderExpr>, async_graphql::Error> {
    let order_val = match args.try_get("orderBy") {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let enum_str = order_val
        .enum_name()
        .map_err(|_| async_graphql::Error::new("orderBy must be an enum value"))?;

    let flat = cube.flat_dimensions();
    let (descending, field_path) = if let Some(path) = enum_str.strip_suffix("_DESC") {
        (true, path)
    } else if let Some(path) = enum_str.strip_suffix("_ASC") {
        (false, path)
    } else {
        return Err(async_graphql::Error::new(format!(
            "Invalid orderBy value: {enum_str}"
        )));
    };

    let column = flat
        .iter()
        .find(|(p, _)| p == field_path)
        .map(|(_, dim)| dim.column.clone())
        .ok_or_else(|| {
            async_graphql::Error::new(format!("Unknown orderBy field: {field_path}"))
        })?;

    Ok(vec![OrderExpr { column, descending }])
}
