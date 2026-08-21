use std::collections::HashMap;

use crate::compiler::ir::*;
use crate::compiler::ir::{CompileResult, JoinType};
use crate::sql::dialect::SqlDialect;

pub struct ClickHouseDialect;

impl ClickHouseDialect {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClickHouseDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for ClickHouseDialect {
    fn compile(&self, ir: &QueryIR) -> CompileResult {
        if let Some(ref builder) = ir.custom_query_builder {
            return (builder.0)(ir);
        }

        let mut bindings = Vec::new();
        let mut alias_remap: Vec<(String, String)> = Vec::new();

        if ir.joins.is_empty() {
            let inner_sql = self.compile_inner(ir, &mut bindings, &mut alias_remap);
            return CompileResult { sql: inner_sql, bindings, alias_remap };
        }

        // When JOINs are present, force aliases on function expression columns
        // (e.g. `argMaxMerge(latest_balance)`) so the outer SELECT can reference
        // them by simple identifier names. Without aliases, `_main.argMaxMerge(x)`
        // is parsed as a function call — a ClickHouse syntax error.
        let mut ir_mod = ir.clone();
        let mut mc_counter = 0u32;
        for sel in &mut ir_mod.selects {
            if let SelectExpr::Column { column, alias } = sel {
                if column.contains('(') && alias.is_none() {
                    let a = format!("__mc_{mc_counter}");
                    mc_counter += 1;
                    alias_remap.push((a.clone(), column.clone()));
                    *alias = Some(a);
                }
            }
        }

        let mut jc_counter = 0u32;
        for join in &mut ir_mod.joins {
            for sel in &mut join.selects {
                if let SelectExpr::Column { column, alias } = sel {
                    if column.contains('(') && alias.is_none() {
                        let a = format!("__jc_{jc_counter}");
                        jc_counter += 1;
                        *alias = Some(a);
                    }
                }
            }
        }

        let inner_sql = self.compile_inner(&ir_mod, &mut bindings, &mut alias_remap);

        // Build outer SELECT with explicit column listing.
        // Always backtick-quote to prevent ClickHouse auto-prefixing ambiguous
        // names when multiple JOINed tables share column names.
        let main_cols: Vec<String> = ir_mod.selects.iter().map(|s| {
            let col = match s {
                SelectExpr::Column { column, alias } => alias.as_ref().unwrap_or(column).clone(),
                SelectExpr::Aggregate { alias, .. } => alias.clone(),
            };
            format!("_main.`{}` AS `{}`", col, col)
        }).collect();

        let mut sql = String::from("SELECT ");
        sql.push_str(&main_cols.join(", "));

        // Collect joined column aliases for the outer SELECT
        for join in &ir_mod.joins {
            for sel in &join.selects {
                let col_name = match sel {
                    SelectExpr::Column { column, alias } => alias.as_ref().unwrap_or(column).clone(),
                    SelectExpr::Aggregate { alias, .. } => alias.clone(),
                };
                let outer_alias = format!("{}.{}", join.alias, col_name);
                if let SelectExpr::Column { column, alias: Some(_) } = sel {
                    if column.contains('(') {
                        let outer_original = format!("{}.{}", join.alias, column);
                        alias_remap.push((outer_alias.clone(), outer_original));
                    }
                }
                sql.push_str(&format!(", {}.`{}` AS `{}`",
                    join.alias, col_name, outer_alias));
            }
        }

        sql.push_str(&format!(" FROM ({}) AS _main", inner_sql));

        // Build a lookup for main query aliases to resolve ON condition columns
        // that might be function expressions (aliased above).
        let main_alias_map: HashMap<String, String> = ir_mod.selects.iter()
            .filter_map(|s| {
                if let SelectExpr::Column { column, alias: Some(a) } = s {
                    if column.contains('(') { Some((column.clone(), a.clone())) } else { None }
                } else { None }
            })
            .collect();

        for join in &ir_mod.joins {
            let join_kw = join.join_type.sql_keyword();

            if join.is_aggregate {
                // Mode B: subquery JOIN for AggregatingMergeTree targets
                sql.push_str(&format!(" {} (SELECT ", join_kw));
                let mut sub_parts: Vec<String> = Vec::new();
                for gb_col in &join.group_by {
                    sub_parts.push(quote_col(gb_col));
                }
                for sel in &join.selects {
                    match sel {
                        SelectExpr::Column { column, alias } => {
                            let col = if column.contains('(') { column.clone() } else { format!("`{column}`") };
                            if let Some(a) = alias {
                                sub_parts.push(format!("{col} AS `{a}`"));
                            } else if column.contains('(') || !join.group_by.contains(column) {
                                sub_parts.push(col);
                            }
                        }
                        SelectExpr::Aggregate { function, column, alias, condition } => {
                            let func = function.to_lowercase();
                            let expr = match (func.as_str(), column.as_str(), condition) {
                                ("count", "*", None) => format!("count() AS `{alias}`"),
                                ("uniq", col, None) => format!("uniq(`{col}`) AS `{alias}`"),
                                (f, col, None) => format!("{f}(`{col}`) AS `{alias}`"),
                                (f, col, Some(cond)) => format!("{f}If(`{col}`, {cond}) AS `{alias}`"),
                            };
                            sub_parts.push(expr);
                        }
                    }
                }
                sql.push_str(&sub_parts.join(", "));
                sql.push_str(&format!(" FROM `{}`.`{}`", join.schema, join.table));
                if !join.group_by.is_empty() {
                    sql.push_str(" GROUP BY ");
                    let gb: Vec<String> = join.group_by.iter().map(|c| quote_col(c)).collect();
                    sql.push_str(&gb.join(", "));
                }
                sql.push_str(&format!(") AS {}", join.alias));
            } else {
                // Mode A: direct JOIN for MergeTree / ReplacingMergeTree targets
                sql.push_str(&format!(" {} `{}`.`{}` AS {}",
                    join_kw, join.schema, join.table, join.alias));
                if join.use_final {
                    sql.push_str(" FINAL");
                }
            }

            if join.join_type == JoinType::Cross {
                // CROSS JOIN has no ON clause
                continue;
            }

            // ON conditions — use alias if the local column was a func expression
            let on_parts: Vec<String> = join.conditions.iter().map(|(local, remote)| {
                let local_ref = main_alias_map.get(local).unwrap_or(local);
                format!("_main.`{}` = {}.`{}`", local_ref, join.alias, remote)
            }).collect();
            sql.push_str(" ON ");
            sql.push_str(&on_parts.join(" AND "));
        }

        CompileResult { sql, bindings, alias_remap }
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("`{name}`")
    }

    fn name(&self) -> &str {
        "ClickHouse"
    }
}

impl ClickHouseDialect {
    fn compile_inner(
        &self,
        ir: &QueryIR,
        bindings: &mut Vec<SqlValue>,
        alias_remap: &mut Vec<(String, String)>,
    ) -> String {
        let mut sql = String::new();

        let mut augmented_selects = ir.selects.clone();
        let mut agg_alias_map: HashMap<String, String> = HashMap::new();
        let mut alias_counter = 0u32;

        let having_cols: std::collections::HashSet<String> =
            collect_filter_columns(&ir.having).into_iter().collect();
        let has_having_agg = having_cols.iter().any(|c| c.contains('('));

        if has_having_agg {
            for sel in &mut augmented_selects {
                if let SelectExpr::Column { column, alias } = sel {
                    if column.contains('(') && having_cols.contains(column.as_str()) {
                        if alias.is_none() {
                            let a = format!("__f_{alias_counter}");
                            alias_counter += 1;
                            alias_remap.push((a.clone(), column.clone()));
                            agg_alias_map.insert(column.clone(), a.clone());
                            *alias = Some(a);
                        } else if let Some(existing) = alias {
                            agg_alias_map.insert(column.clone(), existing.clone());
                        }
                    }
                }
            }
            for col in &having_cols {
                if col.contains('(') && !agg_alias_map.contains_key(col.as_str()) {
                    let a = format!("__f_{alias_counter}");
                    alias_counter += 1;
                    agg_alias_map.insert(col.clone(), a.clone());
                    augmented_selects.push(SelectExpr::Column {
                        column: col.clone(),
                        alias: Some(a),
                    });
                }
            }
        }

        sql.push_str("SELECT ");
        let select_parts: Vec<String> = augmented_selects.iter().map(|s| match s {
            SelectExpr::Column { column, alias } => {
                let col = if column.contains('(') { column.clone() } else { format!("`{column}`") };
                match alias {
                    Some(a) => format!("{col} AS `{a}`"),
                    None => col,
                }
            },
            SelectExpr::Aggregate { function, column, alias, condition } => {
                let func = function.to_uppercase();
                match (func.as_str(), column.as_str(), condition) {
                    ("COUNT", "*", None) => format!("count() AS `{alias}`"),
                    ("COUNT", "*", Some(cond)) => format!("countIf({cond}) AS `{alias}`"),
                    ("UNIQ", col, None) => format!("uniq(`{col}`) AS `{alias}`"),
                    ("UNIQ", col, Some(cond)) => format!("uniqIf(`{col}`, {cond}) AS `{alias}`"),
                    (_, col, None) => format!("{f}(`{col}`) AS `{alias}`", f = func.to_lowercase()),
                    (_, col, Some(cond)) => format!("{f}If(`{col}`, {cond}) AS `{alias}`", f = func.to_lowercase()),
                }
            }
        }).collect();
        sql.push_str(&select_parts.join(", "));

        if let Some(ref subquery) = ir.from_subquery {
            sql.push_str(&format!(" FROM ({}) AS _t", subquery));
        } else {
            sql.push_str(&format!(" FROM `{}`.`{}`", ir.schema, ir.table));
            if ir.use_final {
                sql.push_str(" FINAL");
            }
        }

        let where_clause = compile_filter(&ir.filters, bindings);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        let effective_group_by = if !ir.group_by.is_empty() {
            ir.group_by.clone()
        } else {
            let has_merge_cols = augmented_selects.iter().any(|s| match s {
                SelectExpr::Column { column, .. } => column.contains("Merge("),
                SelectExpr::Aggregate { .. } => true,
            });
            if has_merge_cols {
                augmented_selects.iter().filter_map(|s| match s {
                    SelectExpr::Column { column, .. } if !column.contains("Merge(") && !column.contains('(') => {
                        Some(column.clone())
                    }
                    _ => None,
                }).collect()
            } else {
                vec![]
            }
        };

        if !effective_group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            let cols: Vec<String> = effective_group_by.iter().map(|c| format!("`{c}`")).collect();
            sql.push_str(&cols.join(", "));
        }

        if has_having_agg {
            let having_clause = compile_filter_with_aliases(&ir.having, bindings, &agg_alias_map);
            if !having_clause.is_empty() {
                sql.push_str(" HAVING ");
                sql.push_str(&having_clause);
            }
        } else {
            let having_clause = compile_filter(&ir.having, bindings);
            if !having_clause.is_empty() {
                sql.push_str(" HAVING ");
                sql.push_str(&having_clause);
            }
        }

        if !ir.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let parts: Vec<String> = ir.order_by.iter().map(|o| {
                let col = if o.column.contains('(') {
                    agg_alias_map.get(&o.column)
                        .map(|a| format!("`{a}`"))
                        .unwrap_or_else(|| o.column.clone())
                } else {
                    format!("`{}`", o.column)
                };
                let dir = if o.descending { "DESC" } else { "ASC" };
                format!("{col} {dir}")
            }).collect();
            sql.push_str(&parts.join(", "));
        }

        if let Some(ref lb) = ir.limit_by {
            let by_cols: Vec<String> = lb.columns.iter().map(|c| format!("`{c}`")).collect();
            sql.push_str(&format!(" LIMIT {} BY {}", lb.count, by_cols.join(", ")));
            if lb.offset > 0 {
                sql.push_str(&format!(" OFFSET {}", lb.offset));
            }
        }

        sql.push_str(&format!(" LIMIT {}", ir.limit));
        if ir.offset > 0 {
            sql.push_str(&format!(" OFFSET {}", ir.offset));
        }

        sql
    }
}

/// Collect all column names referenced in a filter tree.
fn collect_filter_columns(node: &FilterNode) -> Vec<String> {
    match node {
        FilterNode::Empty => vec![],
        FilterNode::Condition { column, .. } => vec![column.clone()],
        FilterNode::And(children) | FilterNode::Or(children) => {
            children.iter().flat_map(collect_filter_columns).collect()
        }
    }
}

/// Like `compile_filter` but replaces aggregate expression columns with their
/// SELECT aliases so ClickHouse can resolve them in HAVING scope.
fn compile_filter_with_aliases(
    node: &FilterNode,
    bindings: &mut Vec<SqlValue>,
    aliases: &HashMap<String, String>,
) -> String {
    match node {
        FilterNode::Empty => String::new(),
        FilterNode::Condition { column, op, value } => {
            let effective_col = aliases.get(column)
                .map(|a| a.as_str())
                .unwrap_or(column.as_str());
            compile_condition(effective_col, op, value, bindings)
        }
        FilterNode::And(children) => {
            let parts: Vec<String> = children.iter()
                .map(|c| compile_filter_with_aliases(c, bindings, aliases))
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
                .map(|c| compile_filter_with_aliases(c, bindings, aliases))
                .filter(|s| !s.is_empty())
                .collect();
            match parts.len() {
                0 => String::new(),
                1 => parts.into_iter().next().unwrap(),
                _ => format!("({})", parts.join(" OR ")),
            }
        }
    }
}

fn compile_filter(node: &FilterNode, bindings: &mut Vec<SqlValue>) -> String {
    match node {
        FilterNode::Empty => String::new(),
        FilterNode::Condition { column, op, value } => {
            compile_condition(column, op, value, bindings)
        }
        FilterNode::And(children) => {
            let parts: Vec<String> = children.iter()
                .map(|c| compile_filter(c, bindings))
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
                .map(|c| compile_filter(c, bindings))
                .filter(|s| !s.is_empty())
                .collect();
            match parts.len() {
                0 => String::new(),
                1 => parts.into_iter().next().unwrap(),
                _ => format!("({})", parts.join(" OR ")),
            }
        }
    }
}

fn quote_col(column: &str) -> String {
    if column.contains('(') {
        column.to_string()
    } else {
        format!("`{column}`")
    }
}

fn compile_condition(
    column: &str, op: &CompareOp, value: &SqlValue, bindings: &mut Vec<SqlValue>,
) -> String {
    let col = quote_col(column);
    match op {
        CompareOp::In | CompareOp::NotIn => {
            if let SqlValue::String(csv) = value {
                let items: Vec<&str> = csv.split(',').collect();
                let placeholders: Vec<&str> = items.iter().map(|_| "?").collect();
                for item in &items {
                    bindings.push(SqlValue::String(item.trim().to_string()));
                }
                format!("{col} {} ({})", op.sql_op(), placeholders.join(", "))
            } else {
                bindings.push(value.clone());
                format!("{col} {} (?)", op.sql_op())
            }
        }
        CompareOp::Includes => {
            if let SqlValue::String(s) = value {
                bindings.push(SqlValue::String(format!("%{s}%")));
            } else {
                bindings.push(value.clone());
            }
            format!("{col} LIKE ?")
        }
        CompareOp::IsNull | CompareOp::IsNotNull => {
            format!("{col} {}", op.sql_op())
        }
        _ => {
            bindings.push(value.clone());
            format!("{col} {} ?", op.sql_op())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> ClickHouseDialect { ClickHouseDialect::new() }

    #[test]
    fn test_simple_select() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "default".into(),
            table: "dwd_dex_trades".into(),
            selects: vec![
                SelectExpr::Column { column: "tx_hash".into(), alias: None },
                SelectExpr::Column { column: "token_a_amount".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None,
            use_final: false,
            joins: vec![],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert_eq!(r.sql, "SELECT `tx_hash`, `token_a_amount` FROM `default`.`dwd_dex_trades` LIMIT 10");
        assert!(r.bindings.is_empty());
    }

    #[test]
    fn test_final_keyword() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "tokens".into(),
            selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None,
            use_final: true,
            joins: vec![],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("FROM `db`.`tokens` FINAL"), "FINAL should be appended, got: {}", r.sql);
    }

    #[test]
    fn test_uniq_uses_native_function() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Aggregate { function: "UNIQ".into(), column: "wallet".into(), alias: "__uniq".into(), condition: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("uniq(`wallet`) AS `__uniq`"), "ClickHouse should use native uniq(), got: {}", r.sql);
    }

    #[test]
    fn test_count_star() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Aggregate { function: "COUNT".into(), column: "*".into(), alias: "__count".into(), condition: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("count() AS `__count`"), "ClickHouse should use count() not COUNT(*), got: {}", r.sql);
    }

    #[test]
    fn test_aggregate_lowercase() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Aggregate { function: "SUM".into(), column: "amount".into(), alias: "__sum".into(), condition: None },
                SelectExpr::Aggregate { function: "AVG".into(), column: "price".into(), alias: "__avg".into(), condition: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("sum(`amount`) AS `__sum`"), "ClickHouse functions should be lowercase, got: {}", r.sql);
        assert!(r.sql.contains("avg(`price`) AS `__avg`"), "got: {}", r.sql);
    }

    #[test]
    fn test_where_and_order() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
            filters: FilterNode::And(vec![
                FilterNode::Condition { column: "chain_id".into(), op: CompareOp::Eq, value: SqlValue::Int(1) },
                FilterNode::Condition { column: "amount_usd".into(), op: CompareOp::Gt, value: SqlValue::Float(1000.0) },
            ]),
            having: FilterNode::Empty, group_by: vec![],
            order_by: vec![OrderExpr { column: "block_timestamp".into(), descending: true }],
            limit: 25, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("WHERE (`chain_id` = ? AND `amount_usd` > ?)"));
        assert!(r.sql.contains("ORDER BY `block_timestamp` DESC"));
        assert_eq!(r.bindings.len(), 2);
    }

    #[test]
    fn test_having_with_aggregate_expr() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Column { column: "token_address".into(), alias: None },
                SelectExpr::Aggregate { function: "SUM".into(), column: "amount_usd".into(), alias: "__sum".into(), condition: None },
            ],
            filters: FilterNode::Empty,
            having: FilterNode::Condition {
                column: "sum(`amount_usd`)".into(), op: CompareOp::Gt, value: SqlValue::Float(1000000.0),
            },
            group_by: vec!["token_address".into()], order_by: vec![], limit: 25, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("GROUP BY `token_address`"));
        assert!(r.sql.contains("HAVING `__f_0` > ?"), "expected alias in HAVING, got: {}", r.sql);
        assert!(r.sql.contains("sum(`amount_usd`) AS `__f_0`"), "expected alias in SELECT, got: {}", r.sql);
        assert_eq!(r.bindings.len(), 1);
    }

    #[test]
    fn test_having_appends_missing_agg_column() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Column { column: "pool_address".into(), alias: None },
                SelectExpr::Column { column: "argMaxMerge(latest_liquidity_usd_state)".into(), alias: None },
            ],
            filters: FilterNode::Empty,
            having: FilterNode::And(vec![
                FilterNode::Condition {
                    column: "argMaxMerge(latest_liquidity_usd_state)".into(),
                    op: CompareOp::Gt, value: SqlValue::Float(2.0),
                },
                FilterNode::Condition {
                    column: "argMaxMerge(latest_token_a_amount_state)".into(),
                    op: CompareOp::Gt, value: SqlValue::Float(3.0),
                },
            ]),
            group_by: vec!["pool_address".into()], order_by: vec![], limit: 25, offset: 0,
            limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("argMaxMerge(latest_liquidity_usd_state) AS `__f_0`"),
            "existing HAVING col should be aliased, got: {}", r.sql);
        assert!(r.sql.contains("argMaxMerge(latest_token_a_amount_state) AS `__f_1`"),
            "missing agg col should be appended, got: {}", r.sql);
        assert!(r.sql.contains("HAVING (`__f_0` > ? AND `__f_1` > ?)"),
            "HAVING should use aliases, got: {}", r.sql);
        assert_eq!(r.bindings.len(), 2);
        assert_eq!(r.alias_remap.len(), 1);
        assert_eq!(r.alias_remap[0], ("__f_0".to_string(), "argMaxMerge(latest_liquidity_usd_state)".to_string()));
    }

    #[test]
    fn test_limit_by() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Column { column: "owner".into(), alias: None },
                SelectExpr::Column { column: "amount".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], 
            order_by: vec![OrderExpr { column: "amount".into(), descending: true }],
            limit: 100, offset: 0,
            limit_by: Some(LimitByExpr { count: 3, offset: 0, columns: vec!["owner".into()] }),
            use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        let sql = &r.sql;
        assert!(sql.contains("LIMIT 3 BY `owner`"), "LIMIT BY should be present, got: {sql}");
        assert!(sql.contains("ORDER BY `amount` DESC"), "ORDER BY should be present, got: {sql}");
        assert!(sql.contains("LIMIT 100"), "outer LIMIT should be present, got: {sql}");
        let order_by_pos = sql.find("ORDER BY").unwrap();
        let limit_by_pos = sql.find("LIMIT 3 BY").unwrap();
        let limit_pos = sql.rfind("LIMIT 100").unwrap();
        assert!(order_by_pos < limit_by_pos, "ORDER BY should come before LIMIT BY in ClickHouse");
        assert!(limit_by_pos < limit_pos, "LIMIT BY should come before outer LIMIT");
    }

    #[test]
    fn test_limit_by_with_offset() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: Some(LimitByExpr { count: 5, offset: 2, columns: vec!["token".into(), "wallet".into()] }),
            use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("LIMIT 5 BY `token`, `wallet` OFFSET 2"), "multi-column LIMIT BY with offset, got: {}", r.sql);
    }

    #[test]
    fn test_join_direct() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_dex_trades".into(),
            selects: vec![
                SelectExpr::Column { column: "tx_hash".into(), alias: None },
                SelectExpr::Column { column: "buy_token_address".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 25, offset: 0,
            limit_by: None, use_final: false,
            joins: vec![JoinExpr {
                schema: "dexes_dim".into(), table: "sol_tokens".into(),
                alias: "_j0".into(),
                conditions: vec![("buy_token_address".into(), "token_address".into())],
                selects: vec![
                    SelectExpr::Column { column: "name".into(), alias: None },
                    SelectExpr::Column { column: "symbol".into(), alias: None },
                ],
                group_by: vec![], use_final: true, is_aggregate: false,
                target_cube: "TokenSearch".into(), join_field: "joinBuyToken".into(),
                join_type: JoinType::Left,
            }],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("FROM (SELECT"), "main query should be wrapped, got: {}", r.sql);
        assert!(r.sql.contains("LEFT JOIN `dexes_dim`.`sol_tokens` AS _j0 FINAL"),
            "direct JOIN with FINAL after alias, got: {}", r.sql);
        assert!(r.sql.contains("_main.`buy_token_address` = _j0.`token_address`"),
            "ON condition, got: {}", r.sql);
        assert!(r.sql.contains("_j0.`name` AS `_j0.name`"), "joined col alias, got: {}", r.sql);
    }

    #[test]
    fn test_join_aggregate_subquery() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_dex_trades".into(),
            selects: vec![
                SelectExpr::Column { column: "tx_hash".into(), alias: None },
                SelectExpr::Column { column: "buy_token_address".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false,
            joins: vec![JoinExpr {
                schema: "dexes_dws".into(), table: "sol_token_market_cap".into(),
                alias: "_j0".into(),
                conditions: vec![("buy_token_address".into(), "token_address".into())],
                selects: vec![
                    SelectExpr::Column { column: "argMaxMerge(latest_market_cap_usd_state)".into(), alias: None },
                ],
                group_by: vec!["token_address".into()],
                use_final: false, is_aggregate: true,
                target_cube: "TokenMarketCap".into(), join_field: "joinBuyTokenMarketCap".into(),
                join_type: JoinType::Left,
            }],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("LEFT JOIN (SELECT"), "aggregate should use subquery, got: {}", r.sql);
        assert!(r.sql.contains("GROUP BY `token_address`"), "subquery GROUP BY, got: {}", r.sql);
        assert!(r.sql.contains("FROM `dexes_dws`.`sol_token_market_cap`"), "subquery FROM, got: {}", r.sql);
        assert!(r.sql.contains("argMaxMerge(latest_market_cap_usd_state) AS `__jc_0`"),
            "join func expr should be aliased in subquery, got: {}", r.sql);
        assert!(r.sql.contains("_j0.`__jc_0` AS `_j0.__jc_0`"),
            "outer SELECT should use alias for join func col, got: {}", r.sql);
    }

    #[test]
    fn test_join_main_query_func_expression_columns() {
        let ir = QueryIR {
            cube: "TokenHolders".into(), schema: "dws".into(),
            table: "sol_token_holders".into(),
            selects: vec![
                SelectExpr::Column { column: "token".into(), alias: None },
                SelectExpr::Column { column: "holder".into(), alias: None },
                SelectExpr::Column { column: "argMaxMerge(latest_balance)".into(), alias: None },
                SelectExpr::Column { column: "argMaxMerge(latest_balance_usd)".into(), alias: None },
                SelectExpr::Column { column: "minMerge(first_seen)".into(), alias: None },
                SelectExpr::Column { column: "maxMerge(last_seen)".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![
                OrderExpr { column: "argMaxMerge(latest_balance_usd)".into(), descending: true },
            ],
            limit: 100, offset: 0,
            limit_by: None, use_final: false,
            joins: vec![JoinExpr {
                schema: "dim".into(), table: "sol_tokens".into(),
                alias: "_j0".into(),
                conditions: vec![("token".into(), "token_address".into())],
                selects: vec![
                    SelectExpr::Column { column: "name".into(), alias: None },
                    SelectExpr::Column { column: "symbol".into(), alias: None },
                ],
                group_by: vec![], use_final: true, is_aggregate: false,
                target_cube: "TokenSearch".into(), join_field: "joinToken".into(),
                join_type: JoinType::Left,
            }],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        let sql = &r.sql;

        assert!(sql.contains("_main.`__mc_0` AS `__mc_0`"),
            "func expr should use alias __mc_0 in outer SELECT, got: {sql}");
        assert!(sql.contains("_main.`__mc_1` AS `__mc_1`"),
            "func expr should use alias __mc_1, got: {sql}");
        assert!(sql.contains("_main.`token` AS `token`"),
            "simple col should be backtick-quoted, got: {sql}");

        assert!(!sql.contains("_main.argMaxMerge("),
            "outer SELECT must NOT have bare _main.argMaxMerge(...), got: {sql}");

        assert!(sql.contains("argMaxMerge(latest_balance) AS `__mc_0`"),
            "inner query should alias func expr, got: {sql}");

        assert!(r.alias_remap.iter().any(|(a, o)| a == "__mc_0" && o == "argMaxMerge(latest_balance)"),
            "alias_remap should map __mc_0 → original, got: {:?}", r.alias_remap);
        assert!(r.alias_remap.iter().any(|(a, o)| a == "__mc_1" && o == "argMaxMerge(latest_balance_usd)"),
            "alias_remap should map __mc_1, got: {:?}", r.alias_remap);
    }

    #[test]
    fn test_join_inner_type() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_dex_trades".into(),
            selects: vec![
                SelectExpr::Column { column: "tx_hash".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false,
            joins: vec![JoinExpr {
                schema: "dexes_dim".into(), table: "sol_tokens".into(),
                alias: "_j0".into(),
                conditions: vec![("buy_token_address".into(), "token_address".into())],
                selects: vec![
                    SelectExpr::Column { column: "name".into(), alias: None },
                ],
                group_by: vec![], use_final: false, is_aggregate: false,
                target_cube: "TokenSearch".into(), join_field: "joinBuyToken".into(),
                join_type: JoinType::Inner,
            }],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("INNER JOIN `dexes_dim`.`sol_tokens` AS _j0"),
            "should use INNER JOIN, got: {}", r.sql);
    }

    #[test]
    fn test_join_full_outer_type() {
        let ir = QueryIR {
            cube: "T".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Column { column: "id".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false,
            joins: vec![JoinExpr {
                schema: "db2".into(), table: "t2".into(),
                alias: "_j0".into(),
                conditions: vec![("id".into(), "ref_id".into())],
                selects: vec![
                    SelectExpr::Column { column: "val".into(), alias: None },
                ],
                group_by: vec![], use_final: false, is_aggregate: false,
                target_cube: "Other".into(), join_field: "joinOther".into(),
                join_type: JoinType::Full,
            }],
            custom_query_builder: None,
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert!(r.sql.contains("FULL OUTER JOIN `db2`.`t2` AS _j0"),
            "should use FULL OUTER JOIN, got: {}", r.sql);
    }

    #[test]
    fn test_custom_query_builder() {
        let ir = QueryIR {
            cube: "Custom".into(), schema: "db".into(), table: "t".into(),
            selects: vec![
                SelectExpr::Column { column: "id".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false, joins: vec![],
            custom_query_builder: Some(QueryBuilderFn(std::sync::Arc::new(|_ir| {
                CompileResult {
                    sql: "SELECT 1 FROM custom_view".into(),
                    bindings: vec![],
                    alias_remap: vec![],
                }
            }))),
            from_subquery: None,
        };
        let r = ch().compile(&ir);
        assert_eq!(r.sql, "SELECT 1 FROM custom_view",
            "custom builder should bypass standard compilation, got: {}", r.sql);
    }

    #[test]
    fn test_from_subquery() {
        let ir = QueryIR {
            cube: "DEXTradeByTokens".into(), schema: "dwd".into(),
            table: "sol_trades".into(),
            selects: vec![
                SelectExpr::Column { column: "amount".into(), alias: None },
                SelectExpr::Column { column: "side_type".into(), alias: None },
            ],
            filters: FilterNode::Condition {
                column: "token".into(), op: CompareOp::Eq,
                value: SqlValue::String("SOL".into()),
            },
            having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None, use_final: false, joins: vec![],
            custom_query_builder: None,
            from_subquery: Some(
                "SELECT amount, 'buy' AS side_type, token FROM dwd.sol_a UNION ALL SELECT amount, 'sell' AS side_type, token FROM dwd.sol_b".into()
            ),
        };
        let r = ch().compile(&ir);
        assert!(r.sql.starts_with("SELECT `amount`, `side_type` FROM (SELECT"),
            "should use subquery in FROM, got: {}", r.sql);
        assert!(r.sql.contains("UNION ALL"),
            "subquery should contain UNION ALL, got: {}", r.sql);
        assert!(r.sql.contains(") AS _t"),
            "subquery should be aliased as _t, got: {}", r.sql);
        assert!(r.sql.contains("WHERE `token` = ?"),
            "WHERE clause should be applied to subquery result, got: {}", r.sql);
        assert!(!r.sql.contains("FROM `dwd`.`sol_trades`"),
            "should NOT use schema.table when from_subquery is set, got: {}", r.sql);
    }
}
