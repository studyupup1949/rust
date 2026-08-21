use crate::compiler::ir::*;
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
    fn compile(&self, ir: &QueryIR) -> (String, Vec<SqlValue>) {
        let mut bindings = Vec::new();
        let mut sql = String::new();

        sql.push_str("SELECT ");
        let select_parts: Vec<String> = ir.selects.iter().map(|s| match s {
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

        sql.push_str(&format!(" FROM `{}`.`{}`", ir.schema, ir.table));
        if ir.use_final {
            sql.push_str(" FINAL");
        }

        let where_clause = compile_filter(&ir.filters, &mut bindings);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        // Auto-detect AggregatingMergeTree queries: if any SELECT column contains
        // a -Merge combinator (e.g. argMaxMerge, sumMerge), auto-add GROUP BY
        // for all non-aggregate columns.
        let effective_group_by = if !ir.group_by.is_empty() {
            ir.group_by.clone()
        } else {
            let has_merge_cols = ir.selects.iter().any(|s| match s {
                SelectExpr::Column { column, .. } => column.contains("Merge("),
                SelectExpr::Aggregate { .. } => true,
            });
            if has_merge_cols {
                ir.selects.iter().filter_map(|s| match s {
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

        let having_clause = compile_filter(&ir.having, &mut bindings);
        if !having_clause.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&having_clause);
        }

        if !ir.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let parts: Vec<String> = ir.order_by.iter().map(|o| {
                let dir = if o.descending { "DESC" } else { "ASC" };
                format!("`{}` {dir}", o.column)
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

        (sql, bindings)
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("`{name}`")
    }

    fn name(&self) -> &str {
        "ClickHouse"
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
        };
        let (sql, bindings) = ch().compile(&ir);
        assert_eq!(sql, "SELECT `tx_hash`, `token_a_amount` FROM `default`.`dwd_dex_trades` LIMIT 10");
        assert!(bindings.is_empty());
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
        };
        let (sql, _) = ch().compile(&ir);
        assert!(sql.contains("FROM `db`.`tokens` FINAL"), "FINAL should be appended, got: {sql}");
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
            limit_by: None,
            use_final: false,
        };
        let (sql, _) = ch().compile(&ir);
        assert!(sql.contains("uniq(`wallet`) AS `__uniq`"), "ClickHouse should use native uniq(), got: {sql}");
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
            limit_by: None,
            use_final: false,
        };
        let (sql, _) = ch().compile(&ir);
        assert!(sql.contains("count() AS `__count`"), "ClickHouse should use count() not COUNT(*), got: {sql}");
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
            limit_by: None,
            use_final: false,
        };
        let (sql, _) = ch().compile(&ir);
        assert!(sql.contains("sum(`amount`) AS `__sum`"), "ClickHouse functions should be lowercase, got: {sql}");
        assert!(sql.contains("avg(`price`) AS `__avg`"), "got: {sql}");
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
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = ch().compile(&ir);
        assert!(sql.contains("WHERE (`chain_id` = ? AND `amount_usd` > ?)"));
        assert!(sql.contains("ORDER BY `block_timestamp` DESC"));
        assert_eq!(bindings.len(), 2);
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
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = ch().compile(&ir);
        assert!(sql.contains("GROUP BY `token_address`"));
        assert!(sql.contains("HAVING sum(`amount_usd`) > ?"), "got: {sql}");
        assert_eq!(bindings.len(), 1);
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
            use_final: false,
        };
        let (sql, _) = ch().compile(&ir);
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
            use_final: false,
        };
        let (sql, _) = ch().compile(&ir);
        assert!(sql.contains("LIMIT 5 BY `token`, `wallet` OFFSET 2"), "multi-column LIMIT BY with offset, got: {sql}");
    }
}
