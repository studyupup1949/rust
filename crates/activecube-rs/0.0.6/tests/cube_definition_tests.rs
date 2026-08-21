use activecube_rs::*;

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
    };

    // Table name should remain literal (no {chain} replacement)
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
            use_final: false,
        },
        CubeDefinition {
            name: "B".into(), schema: "s".into(), table_pattern: "t".into(),
            chain_column: None,
            dimensions: vec![], metrics: vec![], selectors: vec![],
            default_filters: vec![], default_limit: 10, max_limit: 100,
            use_final: false,
        },
    ];

    let registry = CubeRegistry::from_cubes(cubes);
    assert!(registry.get("A").is_some());
    assert!(registry.get("B").is_some());
    assert!(registry.get("C").is_none());
    assert_eq!(registry.cube_names().len(), 2);
}
