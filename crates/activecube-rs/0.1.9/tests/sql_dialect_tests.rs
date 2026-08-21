use activecube_rs::*;
use activecube_rs::sql::dialect::SqlDialect;
use activecube_rs::sql::clickhouse::ClickHouseDialect;

#[test]
fn test_clickhouse_quote_identifier() {
    let d = ClickHouseDialect::new();
    assert_eq!(d.quote_identifier("column_name"), "`column_name`");
}

#[test]
fn test_in_filter_splits_csv() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::Condition {
            column: "symbol".into(),
            op: CompareOp::In,
            value: SqlValue::String("SOL,ETH,BTC".into()),
        },
        having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("IN (?, ?, ?)"));
    assert_eq!(r.bindings.len(), 3);
}

#[test]
fn test_includes_generates_like_with_wildcards() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::Condition {
            column: "name".into(),
            op: CompareOp::Includes,
            value: SqlValue::String("test".into()),
        },
        having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("LIKE ?"));
    if let SqlValue::String(s) = &r.bindings[0] {
        assert_eq!(s, "%test%");
    } else {
        panic!("Expected String binding");
    }
}

#[test]
fn test_uniq_generates_native_uniq() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Aggregate {
                function: "UNIQ".into(),
                column: "wallet".into(),
                alias: "__uniq".into(),
                condition: None,
            },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("uniq(`wallet`) AS `__uniq`"));
}

#[test]
fn test_deeply_nested_and_or() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::And(vec![
            FilterNode::Condition {
                column: "a".into(), op: CompareOp::Eq, value: SqlValue::Int(1),
            },
            FilterNode::Or(vec![
                FilterNode::And(vec![
                    FilterNode::Condition { column: "b".into(), op: CompareOp::Gt, value: SqlValue::Int(10) },
                    FilterNode::Condition { column: "c".into(), op: CompareOp::Lt, value: SqlValue::Int(20) },
                ]),
                FilterNode::Condition { column: "d".into(), op: CompareOp::Eq, value: SqlValue::String("x".into()) },
            ]),
        ]),
        having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("(`a` = ? AND ((`b` > ? AND `c` < ?) OR `d` = ?))"));
    assert_eq!(r.bindings.len(), 4);
}

#[test]
fn test_is_null_operator() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::Condition {
            column: "name".into(),
            op: CompareOp::IsNull,
            value: SqlValue::Bool(true),
        },
        having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("`name` IS NULL"), "got: {}", r.sql);
    assert!(r.bindings.is_empty());
}

#[test]
fn test_is_not_null_operator() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::Condition {
            column: "email".into(),
            op: CompareOp::IsNotNull,
            value: SqlValue::Bool(false),
        },
        having: FilterNode::Empty,
        group_by: vec![], order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("`email` IS NOT NULL"), "got: {}", r.sql);
    assert!(r.bindings.is_empty());
}

#[test]
fn test_count_star_aggregate() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Column { column: "category".into(), alias: None },
            SelectExpr::Aggregate { function: "COUNT".into(), column: "*".into(), alias: "__count".into(), condition: None },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec!["category".into()], order_by: vec![], limit: 100, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("count() AS `__count`"), "got: {}", r.sql);
    assert!(r.sql.contains("GROUP BY `category`"), "got: {}", r.sql);
}

#[test]
fn test_dialect_name() {
    let d = ClickHouseDialect::new();
    assert_eq!(d.name(), "ClickHouse");
}

#[test]
fn test_multiple_order_by() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![SelectExpr::Column { column: "id".into(), alias: None }],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec![],
        order_by: vec![
            OrderExpr { column: "created_at".into(), descending: true },
            OrderExpr { column: "id".into(), descending: false },
        ],
        limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("ORDER BY `created_at` DESC, `id` ASC"), "got: {}", r.sql);
}

#[test]
fn test_dim_aggregate_argmax() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Column { column: "owner".into(), alias: None },
            SelectExpr::DimAggregate {
                agg_type: DimAggType::ArgMax,
                value_column: "post_balance".into(),
                compare_column: "block_slot".into(),
                alias: "__da_max_post_balance".into(),
                condition: None,
            },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec!["owner".into()],
        order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("argMax(`post_balance`, `block_slot`) AS `__da_max_post_balance`"),
        "got: {}", r.sql);
    assert!(r.sql.contains("GROUP BY `owner`"), "got: {}", r.sql);
}

#[test]
fn test_dim_aggregate_argmin() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::DimAggregate {
                agg_type: DimAggType::ArgMin,
                value_column: "price_in_usd".into(),
                compare_column: "ts".into(),
                alias: "__da_min_price_in_usd".into(),
                condition: None,
            },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec![],
        order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("argMin(`price_in_usd`, `ts`) AS `__da_min_price_in_usd`"),
        "got: {}", r.sql);
}

#[test]
fn test_dim_aggregate_argmax_if_with_condition() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Column { column: "mint".into(), alias: None },
            SelectExpr::DimAggregate {
                agg_type: DimAggType::ArgMax,
                value_column: "post_balance".into(),
                compare_column: "block_slot".into(),
                alias: "__da_max_post_balance".into(),
                condition: Some("`post_balance` > 0".into()),
            },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec!["mint".into()],
        order_by: vec![], limit: 10, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("argMaxIf(`post_balance`, `block_slot`, `post_balance` > 0) AS `__da_max_post_balance`"),
        "got: {}", r.sql);
}

#[test]
fn test_dim_aggregate_with_having() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Column { column: "mint".into(), alias: None },
            SelectExpr::DimAggregate {
                agg_type: DimAggType::ArgMax,
                value_column: "post_balance".into(),
                compare_column: "block_slot".into(),
                alias: "__da_max_post_balance".into(),
                condition: None,
            },
        ],
        filters: FilterNode::Empty,
        having: FilterNode::Condition {
            column: "argMax(`post_balance`, `block_slot`)".into(),
            op: CompareOp::Gt,
            value: SqlValue::Float(0.0),
        },
        group_by: vec!["mint".into()],
        order_by: vec![], limit: 100, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("HAVING"), "expected HAVING clause, got: {}", r.sql);
    assert!(r.sql.contains("argMax(`post_balance`, `block_slot`)"),
        "HAVING should reference argMax, got: {}", r.sql);
}

#[test]
fn test_dim_aggregate_with_order_by_agg_column() {
    let d = ClickHouseDialect::new();
    let ir = QueryIR {
        cube: "T".into(), schema: "s".into(), table: "t".into(),
        selects: vec![
            SelectExpr::Column { column: "mint".into(), alias: None },
            SelectExpr::DimAggregate {
                agg_type: DimAggType::ArgMax,
                value_column: "post_balance".into(),
                compare_column: "block_slot".into(),
                alias: "__da_max_post_balance".into(),
                condition: None,
            },
        ],
        filters: FilterNode::Empty, having: FilterNode::Empty,
        group_by: vec!["mint".into()],
        order_by: vec![OrderExpr {
            column: "argMax(`post_balance`, `block_slot`)".into(),
            descending: true,
        }],
        limit: 50, offset: 0,
        limit_by: None, use_final: false, joins: vec![], custom_query_builder: None, from_subquery: None,
    };
    let r = d.compile(&ir);
    assert!(r.sql.contains("ORDER BY"), "expected ORDER BY, got: {}", r.sql);
    assert!(r.sql.contains("DESC"), "expected DESC, got: {}", r.sql);
}
