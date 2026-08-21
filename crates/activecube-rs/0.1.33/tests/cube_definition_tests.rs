use std::collections::HashMap;
use activecube_rs::*;

/// bid: prefix stripping — Bitquery-compat invariant (never remove).
/// Verifies a StripPrefix rule converts `bid:solana:<addr>` / `bid:bitcoin` into
/// the DB-native form, and that values without the prefix pass through unchanged.
#[test]
fn test_strip_prefix_bid_invariant() {
    let rules = vec![
        FilterValueTransform::StripPrefix { column: "token_id".into(), prefix: "bid:".into() },
    ];

    let mut f1 = FilterNode::Condition {
        column: "token_id".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("bid:solana:27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4".into()),
    };
    f1.apply_filter_value_transforms(&rules);
    match &f1 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "solana:27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4");
        }
        _ => panic!("expected Condition after transform"),
    }

    let mut f2 = FilterNode::Condition {
        column: "token_id".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("solana:27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4".into()),
    };
    f2.apply_filter_value_transforms(&rules);
    match &f2 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "solana:27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4",
                       "non-prefixed value must pass through unchanged");
        }
        _ => panic!("expected Condition after transform"),
    }

    let mut f3 = FilterNode::Condition {
        column: "token_id".into(),
        op: CompareOp::In,
        value: SqlValue::String("bid:bitcoin,bid:solana:abc,eth:0x123".into()),
    };
    f3.apply_filter_value_transforms(&rules);
    match &f3 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "bitcoin,solana:abc,eth:0x123",
                       "list should strip prefix per element, leave non-prefixed parts intact");
        }
        _ => panic!("expected Condition after transform"),
    }

    let mut f4 = FilterNode::Condition {
        column: "other_col".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("bid:solana:xyz".into()),
    };
    f4.apply_filter_value_transforms(&rules);
    match &f4 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "bid:solana:xyz", "non-matching column must not be touched");
        }
        _ => panic!("expected Condition after transform"),
    }
}

/// FormatAddress::EvmEip55 — re-encode EVM filter values to EIP-55 canonical form.
/// Lower-case / upper-case / mixed-case inputs should all produce identical output.
#[test]
fn test_format_address_eip55() {
    let rules = vec![
        FilterValueTransform::FormatAddress {
            column: "pool_smart_contract".into(),
            format: AddressFormat::EvmEip55,
        },
    ];

    // Lower-case input → EIP55
    let mut f_lower = FilterNode::Condition {
        column: "pool_smart_contract".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("0x5d4f3c6fa16908609bac31ff148bd002aa6b8c83".into()),
    };
    f_lower.apply_filter_value_transforms(&rules);
    match &f_lower {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "0x5d4F3C6fA16908609BAC31Ff148Bd002AA6b8c83");
        }
        _ => panic!("expected Condition"),
    }

    // Mixed-case input → same EIP55
    let mut f_mixed = FilterNode::Condition {
        column: "pool_smart_contract".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("0x5D4F3C6fA16908609BAC31Ff148Bd002AA6b8c83".into()),
    };
    f_mixed.apply_filter_value_transforms(&rules);
    match &f_mixed {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "0x5d4F3C6fA16908609BAC31Ff148Bd002AA6b8c83");
        }
        _ => panic!("expected Condition"),
    }

    // In-list: each element re-encoded
    let mut f_list = FilterNode::Condition {
        column: "pool_smart_contract".into(),
        op: CompareOp::In,
        value: SqlValue::String(
            "0x5AAEB6053F3E94C9B9A09F33669435E7EF1BEAED, 0xFB6916095ca1df60bb79ce92ce3ea74c37c5d359".into(),
        ),
    };
    f_list.apply_filter_value_transforms(&rules);
    match &f_list {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed,0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
        }
        _ => panic!("expected list Condition"),
    }

    // Invalid value (not 0x+40 hex) passes through unchanged
    let mut f_invalid = FilterNode::Condition {
        column: "pool_smart_contract".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("not-an-address".into()),
    };
    f_invalid.apply_filter_value_transforms(&rules);
    match &f_invalid {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "not-an-address", "invalid values must pass through unchanged");
        }
        _ => panic!("expected Condition"),
    }

    // Non-matching column untouched
    let mut f_other = FilterNode::Condition {
        column: "other_col".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("0x5d4f3c6fa16908609bac31ff148bd002aa6b8c83".into()),
    };
    f_other.apply_filter_value_transforms(&rules);
    match &f_other {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "0x5d4f3c6fa16908609bac31ff148bd002aa6b8c83", "non-matching column untouched");
        }
        _ => panic!("expected Condition"),
    }
}

/// FormatAddress::Base58 pass-through — valid base58 unchanged, invalid unchanged.
#[test]
fn test_format_address_base58_passthrough() {
    let rules = vec![
        FilterValueTransform::FormatAddress {
            column: "mint_address".into(),
            format: AddressFormat::Base58,
        },
    ];
    let sol = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";
    let mut f = FilterNode::Condition {
        column: "mint_address".into(),
        op: CompareOp::Eq,
        value: SqlValue::String(sol.into()),
    };
    f.apply_filter_value_transforms(&rules);
    match &f {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, sol, "valid base58 passes through unchanged");
        }
        _ => panic!(),
    }
}

/// FormatAddress::Upper / Lower — generic case conversion (non-address columns).
#[test]
fn test_format_address_upper_lower() {
    let up_rules = vec![FilterValueTransform::FormatAddress {
        column: "tag".into(),
        format: AddressFormat::Upper,
    }];
    let mut f1 = FilterNode::Condition {
        column: "tag".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("hello".into()),
    };
    f1.apply_filter_value_transforms(&up_rules);
    match &f1 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => assert_eq!(s, "HELLO"),
        _ => panic!(),
    }

    let low_rules = vec![FilterValueTransform::FormatAddress {
        column: "tag".into(),
        format: AddressFormat::Lower,
    }];
    let mut f2 = FilterNode::Condition {
        column: "tag".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("HELLO".into()),
    };
    f2.apply_filter_value_transforms(&low_rules);
    match &f2 {
        FilterNode::Condition { value: SqlValue::String(s), .. } => assert_eq!(s, "hello"),
        _ => panic!(),
    }
}

/// MultiplyBy transform — numeric unit conversion (e.g. minutes → seconds).
#[test]
fn test_multiply_by_transform() {
    let rules = vec![
        FilterValueTransform::MultiplyBy { column: "interval_duration".into(), factor: 60 },
    ];

    let mut f_int = FilterNode::Condition {
        column: "interval_duration".into(),
        op: CompareOp::Eq,
        value: SqlValue::Int(1),
    };
    f_int.apply_filter_value_transforms(&rules);
    match &f_int {
        FilterNode::Condition { value: SqlValue::Int(n), .. } => assert_eq!(*n, 60),
        _ => panic!("expected Int(60)"),
    }

    let mut f_str = FilterNode::Condition {
        column: "interval_duration".into(),
        op: CompareOp::Eq,
        value: SqlValue::String("15".into()),
    };
    f_str.apply_filter_value_transforms(&rules);
    match &f_str {
        FilterNode::Condition { value: SqlValue::String(s), .. } => assert_eq!(s, "900"),
        _ => panic!("expected String(\"900\")"),
    }

    let mut f_list = FilterNode::Condition {
        column: "interval_duration".into(),
        op: CompareOp::In,
        value: SqlValue::String("1,5,15,60".into()),
    };
    f_list.apply_filter_value_transforms(&rules);
    match &f_list {
        FilterNode::Condition { value: SqlValue::String(s), .. } => {
            assert_eq!(s, "60,300,900,3600");
        }
        _ => panic!("expected list multiply"),
    }
}

#[test]
fn test_cube_definition_table_resolution() {
    let cube = CubeDefinition {
        name: "Test".into(),
        schema: "test_db".into(),
        table_pattern: "{chain}_trades".into(),
        chain_column: None,
        dimensions: vec![],
        metrics: vec![],
        selectors: vec![],
        default_filters: vec![],
        default_limit: 10,
        max_limit: 1000,
        use_final: false,
        description: String::new(),
        joins: vec![],
        table_routes: vec![],
        custom_query_builder: None,
        from_subquery: None,
        required_group_by: vec![],
        chain_groups: vec![],
        chain_overrides: HashMap::new(),
        lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
    };

    assert_eq!(cube.table_for_chain("sol"), "sol_trades");
    assert_eq!(cube.table_for_chain("eth"), "eth_trades");
    assert_eq!(cube.qualified_table("bsc"), "test_db.bsc_trades");
}

#[test]
fn test_flat_dimensions() {
    let cube = CubeDefinition {
        name: "Test".into(),
        schema: "db".into(),
        table_pattern: "t".into(),
        chain_column: None,
        dimensions: vec![
            dim_group("Block", vec![
                dim("Date", "block_time", DimType::DateTime),
                dim("Number", "block_number", DimType::Int),
            ]),
            dim_group("Trade", vec![
                dim_group("Buy", vec![
                    dim("Amount", "buy_amount", DimType::Float),
                    dim_group("Currency", vec![
                        dim("Symbol", "buy_symbol", DimType::String),
                    ]),
                ]),
            ]),
            dim("Success", "success", DimType::Bool),
        ],
        metrics: vec![],
        selectors: vec![],
        default_filters: vec![],
        default_limit: 25,
        max_limit: 10000,
        use_final: false,
        description: String::new(),
        joins: vec![],
        table_routes: vec![],
        custom_query_builder: None,
        from_subquery: None,
        required_group_by: vec![],
        chain_groups: vec![],
        chain_overrides: HashMap::new(),
        lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
    };

    let flat = cube.flat_dimensions();
    let paths: Vec<&str> = flat.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(paths, vec![
        "Block_Date", "Block_Number",
        "Trade_Buy_Amount", "Trade_Buy_Currency_Symbol",
        "Success",
    ]);
}

#[test]
fn test_chain_column_table_name() {
    let cube = CubeDefinition {
        name: "Pools".into(),
        schema: "dexes_dwd".into(),
        table_pattern: "dex_pool_liquidities".into(),
        chain_column: Some("chain".into()),
        dimensions: vec![],
        metrics: vec![],
        selectors: vec![],
        default_filters: vec![],
        default_limit: 25,
        max_limit: 10000,
        use_final: false,
        description: String::new(),
        joins: vec![],
        table_routes: vec![],
        custom_query_builder: None,
        from_subquery: None,
        required_group_by: vec![],
        chain_groups: vec![],
        chain_overrides: HashMap::new(),
        lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
    };

    assert_eq!(cube.table_for_chain("sol"), "dex_pool_liquidities");
    assert_eq!(cube.table_for_chain("eth"), "dex_pool_liquidities");
}

#[test]
fn test_registry_lookup() {
    let cubes = vec![
        CubeDefinition {
            name: "A".into(), schema: "s".into(), table_pattern: "t".into(),
            chain_column: None,
            dimensions: vec![], metrics: vec![], selectors: vec![],
            default_filters: vec![], default_limit: 10, max_limit: 100,
            use_final: false, description: String::new(), joins: vec![],
            table_routes: vec![], custom_query_builder: None, from_subquery: None, required_group_by: vec![],
            chain_groups: vec![], chain_overrides: HashMap::new(), lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
        },
        CubeDefinition {
            name: "B".into(), schema: "s".into(), table_pattern: "t".into(),
            chain_column: None,
            dimensions: vec![], metrics: vec![], selectors: vec![],
            default_filters: vec![], default_limit: 10, max_limit: 100,
            use_final: false, description: String::new(), joins: vec![],
            table_routes: vec![], custom_query_builder: None, from_subquery: None, required_group_by: vec![],
            chain_groups: vec![], chain_overrides: HashMap::new(), lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
        },
    ];

    let registry = CubeRegistry::from_cubes(cubes);
    assert!(registry.get("A").is_some());
    assert!(registry.get("B").is_some());
    assert!(registry.get("C").is_none());
    assert_eq!(registry.cube_names().len(), 2);
}

#[test]
fn test_resolve_table_with_routes() {
    let cube = CubeDefinition {
        name: "TokenTradeStats".into(),
        schema: "dexes_dwm".into(),
        table_pattern: "{chain}_token_trade_stats_1m".into(),
        chain_column: None,
        dimensions: vec![
            dim("token_address", "token_address", DimType::String),
            dim("volume_usd", "volume_usd", DimType::Float),
            dim("trade_count", "trade_count", DimType::Int),
        ],
        metrics: vec![],
        selectors: vec![],
        default_filters: vec![],
        default_limit: 25,
        max_limit: 10000,
        use_final: false,
        description: String::new(),
        joins: vec![],
        table_routes: vec![
            TableRoute {
                schema: "dexes_dws".into(),
                table_pattern: "{chain}_token_trade_stats_daily".into(),
                available_columns: vec!["token_address".into(), "volume_usd".into()],
                priority: 1,
            },
        ],
        custom_query_builder: None,
        from_subquery: None,
        required_group_by: vec![],
        chain_groups: vec![],
        chain_overrides: HashMap::new(),
        lowercase_filter_columns: vec![], filter_value_transforms: vec![], aggregate_only_fields: vec![],
    };

    // When requested columns fit the route, use the routed table
    let (schema, table) = cube.resolve_table("sol", &["token_address".into(), "volume_usd".into()]);
    assert_eq!(schema, "dexes_dws");
    assert_eq!(table, "sol_token_trade_stats_daily");

    // When requested columns don't fit, fall back to primary
    let (schema, table) = cube.resolve_table("sol", &["token_address".into(), "trade_count".into()]);
    assert_eq!(schema, "dexes_dwm");
    assert_eq!(table, "sol_token_trade_stats_1m");
}

#[test]
fn test_metric_def_standard_and_custom() {
    let std_metric = MetricDef::standard("count");
    assert_eq!(std_metric.name, "count");
    assert!(std_metric.expression_template.is_none());
    assert!(std_metric.supports_where);

    let custom = MetricDef::custom("netFlow", "sumIf({column}, direction='in') - sumIf({column}, direction='out')")
        .with_description("Net token flow");
    assert_eq!(custom.name, "netFlow");
    assert!(custom.expression_template.is_some());
    assert!(!custom.supports_where);
    assert_eq!(custom.description.as_deref(), Some("Net token flow"));

    let batch = standard_metrics(&["count", "sum", "avg"]);
    assert_eq!(batch.len(), 3);
    assert_eq!(batch[1].name, "sum");
}
