use crate::compiler::ir::*;
use crate::sql::dialect::SqlDialect;

pub struct StarRocksDialect;

impl StarRocksDialect {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StarRocksDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlDialect for StarRocksDialect {
    fn compile(&self, ir: &QueryIR) -> (String, Vec<SqlValue>) {
        let mut bindings = Vec::new();
        let mut sql = String::new();

        sql.push_str("SELECT ");
        let select_parts: Vec<String> = ir.selects.iter().map(|s| match s {
            SelectExpr::Column { column, alias } => match alias {
                Some(a) => format!("`{column}` AS `{a}`"),
                None => format!("`{column}`"),
            },
            SelectExpr::Aggregate { function, column, alias, condition } => {
                let func = function.to_uppercase();
                match (func.as_str(), column.as_str(), condition) {
                    ("COUNT", "*", None) => format!("COUNT(*) AS `{alias}`"),
                    ("COUNT", "*", Some(cond)) => format!("COUNT(IF({cond}, 1, NULL)) AS `{alias}`"),
                    ("UNIQ", col, None) => format!("COUNT(DISTINCT `{col}`) AS `{alias}`"),
                    ("UNIQ", col, Some(cond)) => format!("COUNT(DISTINCT IF({cond}, `{col}`, NULL)) AS `{alias}`"),
                    (f, col, None) => format!("{f}(`{col}`) AS `{alias}`"),
                    (f, col, Some(cond)) => format!("{f}(IF({cond}, `{col}`, NULL)) AS `{alias}`"),
                }
            }
        }).collect();
        sql.push_str(&select_parts.join(", "));

        sql.push_str(&format!(" FROM `{}`.`{}`", ir.schema, ir.table));

        let where_clause = compile_filter(&ir.filters, &mut bindings);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        if !ir.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            let cols: Vec<String> = ir.group_by.iter().map(|c| format!("`{c}`")).collect();
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
        "StarRocks"
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

/// Quote a column identifier, but leave aggregate expressions (containing `(`)
/// unquoted so that `SUM(\`col\`)` doesn't become `` `SUM(\`col\`)` ``.
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

    fn make_dialect() -> StarRocksDialect { StarRocksDialect::new() }

    #[test]
    fn test_simple_select() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_activities".into(),
            selects: vec![
                SelectExpr::Column { column: "tx_hash".into(), alias: None },
                SelectExpr::Column { column: "buy_amount".into(), alias: None },
            ],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 0,
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = make_dialect().compile(&ir);
        assert_eq!(sql, "SELECT `tx_hash`, `buy_amount` FROM `dexes_dwd`.`sol_activities` LIMIT 10");
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_where_and_order() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_activities".into(),
            selects: vec![SelectExpr::Column { column: "tx_hash".into(), alias: None }],
            filters: FilterNode::And(vec![
                FilterNode::Condition { column: "buy_amount_usd".into(), op: CompareOp::Gt, value: SqlValue::Float(1000.0) },
                FilterNode::Condition { column: "success".into(), op: CompareOp::Eq, value: SqlValue::Bool(true) },
            ]),
            having: FilterNode::Empty, group_by: vec![],
            order_by: vec![OrderExpr { column: "buy_amount_usd".into(), descending: true }],
            limit: 25, offset: 0,
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = make_dialect().compile(&ir);
        assert!(sql.contains("WHERE (`buy_amount_usd` > ? AND `success` = ?)"));
        assert!(sql.contains("ORDER BY `buy_amount_usd` DESC"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_or_condition() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_activities".into(),
            selects: vec![SelectExpr::Column { column: "tx_hash".into(), alias: None }],
            filters: FilterNode::And(vec![
                FilterNode::Condition { column: "buy_amount_usd".into(), op: CompareOp::Gt, value: SqlValue::Float(1000.0) },
                FilterNode::Or(vec![
                    FilterNode::Condition { column: "buy_token_symbol".into(), op: CompareOp::Eq, value: SqlValue::String("SOL".into()) },
                    FilterNode::Condition { column: "sell_token_symbol".into(), op: CompareOp::Eq, value: SqlValue::String("SOL".into()) },
                ]),
            ]),
            having: FilterNode::Empty, group_by: vec![], order_by: vec![], limit: 25, offset: 0,
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = make_dialect().compile(&ir);
        assert!(sql.contains("(`buy_token_symbol` = ? OR `sell_token_symbol` = ?)"));
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_aggregate_with_having() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_activities".into(),
            selects: vec![
                SelectExpr::Column { column: "buy_token_symbol".into(), alias: None },
                SelectExpr::Aggregate { function: "SUM".into(), column: "buy_amount_usd".into(), alias: "__sum".into(), condition: None },
            ],
            filters: FilterNode::Empty,
            having: FilterNode::Condition {
                column: "SUM(`buy_amount_usd`)".into(), op: CompareOp::Gt, value: SqlValue::Float(1000000.0),
            },
            group_by: vec!["buy_token_symbol".into()], order_by: vec![], limit: 25, offset: 0,
            limit_by: None,
            use_final: false,
        };
        let (sql, bindings) = make_dialect().compile(&ir);
        assert!(sql.contains("GROUP BY `buy_token_symbol`"));
        assert!(sql.contains("HAVING SUM(`buy_amount_usd`) > ?"), "HAVING clause should not backtick-wrap aggregate expressions, got: {sql}");
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_offset() {
        let ir = QueryIR {
            cube: "DEXTrades".into(), schema: "dexes_dwd".into(),
            table: "sol_activities".into(),
            selects: vec![SelectExpr::Column { column: "tx_hash".into(), alias: None }],
            filters: FilterNode::Empty, having: FilterNode::Empty,
            group_by: vec![], order_by: vec![], limit: 10, offset: 20,
            limit_by: None,
            use_final: false,
        };
        let (sql, _) = make_dialect().compile(&ir);
        assert!(sql.ends_with("LIMIT 10 OFFSET 20"));
    }
}
