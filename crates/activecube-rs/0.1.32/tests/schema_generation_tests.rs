use std::sync::Arc;
use activecube_rs::*;
use activecube_rs::schema::generator::{build_schema, QueryExecutor, SchemaConfig, ChainGroupConfig};
use activecube_rs::sql::clickhouse::ClickHouseDialect;

fn test_cube() -> CubeDefinition {
    CubeBuilder::new("TestTrades")
        .schema("test_db")
        .table("{chain}_trades")
        .dimension(dim_group("Block", vec![
            dim("Number", "block_num", DimType::Int),
            dim("Date", "block_time", DimType::DateTime),
        ]))
        .dimension(dim_group("Trade", vec![
            dim("Amount", "amount", DimType::Float),
            dim("Symbol", "symbol", DimType::String),
        ]))
        .metrics(&["count", "sum"])
        .default_filter("success", "true")
        .chain_groups(vec![ChainGroup::Evm, ChainGroup::Solana])
        .build()
}

fn test_schema_config() -> SchemaConfig {
    SchemaConfig {
        chain_groups: vec![
            ChainGroupConfig {
                name: "EVM".into(),
                group: ChainGroup::Evm,
                networks: vec!["eth".into()],
                has_network_arg: true,
                extra_args: vec![],
                network_enum_name: None,
                network_aliases: std::collections::HashMap::new(),
            },
            ChainGroupConfig {
                name: "Solana".into(),
                group: ChainGroup::Solana,
                networks: vec!["sol".into()],
                has_network_arg: false,
                extra_args: vec![],
                network_enum_name: None,
                network_aliases: std::collections::HashMap::new(),
            },
        ],
        root_query_name: "ChainStream".into(),
        stats_callback: None,
        extra_types: vec![],
        table_name_transform: None,
    }
}

fn noop_executor() -> QueryExecutor {
    Arc::new(|_sql, _bindings| {
        Box::pin(async { Ok(vec![]) })
    })
}

#[tokio::test]
async fn test_schema_builds_successfully() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config());
    assert!(schema.is_ok(), "Schema build failed: {:?}", schema.err());
}

#[tokio::test]
async fn test_schema_introspection_contains_cube() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute("{ __schema { queryType { name } } }").await;
    let data = result.data.into_json().unwrap();
    let query_type_name = data["__schema"]["queryType"]["name"].as_str().unwrap();
    assert_eq!(query_type_name, "Query");
}

#[tokio::test]
async fn test_schema_has_network_enum() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(r#"{ __type(name: "EVMNetwork") { enumValues { name } } }"#).await;
    let data = result.data.into_json().unwrap();
    let values = data["__type"]["enumValues"].as_array().unwrap();
    let names: Vec<&str> = values.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"eth"));
}

#[tokio::test]
async fn test_schema_has_cube_filter_type() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(r#"{ __type(name: "TestTradesFilter") { inputFields { name } } }"#).await;
    let data = result.data.into_json().unwrap();
    let fields = data["__type"]["inputFields"].as_array().unwrap();
    let names: Vec<&str> = fields.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"any"), "Filter should have 'any' for OR support");
    assert!(names.contains(&"Block"), "Filter should have 'Block' dimension group");
    assert!(names.contains(&"Trade"), "Filter should have 'Trade' dimension group");
}

#[tokio::test]
async fn test_schema_has_cube_record_type() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(r#"{ __type(name: "TestTradesRecord") { fields { name } } }"#).await;
    let data = result.data.into_json().unwrap();
    let fields = data["__type"]["fields"].as_array().unwrap();
    let names: Vec<&str> = fields.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Block"), "Record should have 'Block' group");
    assert!(names.contains(&"Trade"), "Record should have 'Trade' group");
    assert!(names.contains(&"count"), "Record should have 'count' metric");
    assert!(names.contains(&"sum"), "Record should have 'sum' metric");
}

#[tokio::test]
async fn test_query_execution_calls_executor() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor: QueryExecutor = Arc::new(move |sql, _bindings| {
        called_clone.store(true, Ordering::SeqCst);
        assert!(sql.contains("FROM `test_db`.`sol_trades`"), "SQL should target sol_trades, got: {sql}");
        Box::pin(async { Ok(vec![]) })
    });

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(r#"
        {
            Solana {
                TestTrades(limit: { count: 5 }) {
                    Block { Number Date }
                    Trade { Amount Symbol }
                }
            }
        }
    "#).await;

    assert!(result.errors.is_empty(), "Query errors: {:?}", result.errors);
    assert!(called.load(Ordering::SeqCst), "Executor should have been called");
}

#[tokio::test]
async fn test_query_with_where_filter() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor: QueryExecutor = Arc::new(move |sql, bindings| {
        assert!(sql.contains("`amount` > ?"), "SQL should have amount filter, got: {sql}");
        let has_1000 = bindings.iter().any(|b| matches!(b, SqlValue::Float(f) if *f == 1000.0));
        assert!(has_1000, "Bindings should contain 1000.0, got: {:?}", bindings);
        Box::pin(async { Ok(vec![]) })
    });

    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(r#"
        {
            EVM(network: eth) {
                TestTrades(
                    where: { Trade: { Amount: { gt: 1000.0 } } }
                    limit: { count: 10 }
                ) {
                    Trade { Amount }
                }
            }
        }
    "#).await;

    assert!(result.errors.is_empty(), "Query errors: {:?}", result.errors);
}

// ---------------------------------------------------------------------------
// Array dimension + Union type tests
// ---------------------------------------------------------------------------

fn instructions_cube() -> CubeDefinition {
    use activecube_rs::{dim_desc, dim_group_desc, dim_array_desc, array_field, variant, ArrayFieldType};

    CubeBuilder::new("Instructions")
        .schema("dexes_dwd2")
        .table("sol_instructions")
        .dimension(dim_group_desc("Instruction", "Instruction details", vec![
            dim_group_desc("Program", "Program", vec![
                dim_desc("Address", "instruction_program_address", DimType::String, "Program address"),
                dim_desc("Method", "instruction_program_method", DimType::String, "Method name"),
                dim_array_desc("Arguments", "Parsed ABI arguments", vec![
                    array_field("Name", "instruction_arg_names", ArrayFieldType::Scalar(DimType::String)),
                    array_field("Type", "instruction_arg_types", ArrayFieldType::Scalar(DimType::String)),
                    array_field("Value", "instruction_arg_values", ArrayFieldType::Union(vec![
                        variant("Solana_ABI_Integer_Value_Arg", "integer", DimType::Int),
                        variant("Solana_ABI_BigInt_Value_Arg", "bigInteger", DimType::String),
                        variant("Solana_ABI_String_Value_Arg", "string", DimType::String),
                        variant("Solana_ABI_Address_Value_Arg", "address", DimType::String),
                        variant("Solana_ABI_Boolean_Value_Arg", "bool", DimType::Bool),
                        variant("Solana_ABI_Float_Value_Arg", "float", DimType::Float),
                        variant("Solana_ABI_Bytes_Value_Arg", "hex", DimType::String),
                        variant("Solana_ABI_Json_Value_Arg", "json", DimType::String),
                    ])),
                ]),
                dim_array_desc("Accounts", "Accounts involved", vec![
                    array_field("Address", "instruction_accounts", ArrayFieldType::Scalar(DimType::String)),
                    array_field("Name", "instruction_account_names", ArrayFieldType::Scalar(DimType::String)),
                ]),
            ]),
        ]))
        .metrics(&["count"])
        .chain_groups(vec![ChainGroup::Evm, ChainGroup::Solana])
        .build()
}

#[tokio::test]
async fn test_schema_with_array_dimension_builds() {
    let registry = CubeRegistry::from_cubes(vec![instructions_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, test_schema_config());
    assert!(schema.is_ok(), "Schema with array dimensions should build: {:?}", schema.err());
}

#[tokio::test]
async fn test_schema_has_array_element_type() {
    let registry = CubeRegistry::from_cubes(vec![instructions_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();
    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(
        r#"{ __type(name: "Instructions_Instruction_Program_Arguments_Element") { fields { name } } }"#
    ).await;
    let data = result.data.into_json().unwrap();
    let fields = data["__type"]["fields"].as_array()
        .expect("Arguments Element type should exist");
    let names: Vec<&str> = fields.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Name"), "Element should have Name field, got: {:?}", names);
    assert!(names.contains(&"Type"), "Element should have Type field, got: {:?}", names);
    assert!(names.contains(&"Value"), "Element should have Value field, got: {:?}", names);
}

#[tokio::test]
async fn test_schema_has_union_type() {
    let registry = CubeRegistry::from_cubes(vec![instructions_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();
    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(
        r#"{ __type(name: "Instructions_Instruction_Program_Arguments_Value_Union") { kind possibleTypes { name } } }"#
    ).await;
    let data = result.data.into_json().unwrap();
    let kind = data["__type"]["kind"].as_str().expect("Union type should exist");
    assert_eq!(kind, "UNION", "Value should be a UNION type");

    let possible: Vec<&str> = data["__type"]["possibleTypes"].as_array().unwrap()
        .iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(possible.contains(&"Solana_ABI_Integer_Value_Arg"), "Union should include Integer variant, got: {:?}", possible);
    assert!(possible.contains(&"Solana_ABI_BigInt_Value_Arg"), "Union should include BigInt variant, got: {:?}", possible);
    assert!(possible.contains(&"Solana_ABI_Json_Value_Arg"), "Union should include Json variant, got: {:?}", possible);
}

#[tokio::test]
async fn test_schema_has_includes_filter() {
    let registry = CubeRegistry::from_cubes(vec![instructions_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();
    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(
        r#"{ __type(name: "Instructions_Instruction_Program_Arguments_IncludesFilter") { inputFields { name } } }"#
    ).await;
    let data = result.data.into_json().unwrap();
    let fields = data["__type"]["inputFields"].as_array()
        .expect("IncludesFilter type should exist");
    let names: Vec<&str> = fields.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Name"), "IncludesFilter should have Name field, got: {:?}", names);
    assert!(names.contains(&"Value"), "IncludesFilter should have Value field, got: {:?}", names);
}

#[tokio::test]
async fn test_schema_accounts_array_element() {
    let registry = CubeRegistry::from_cubes(vec![instructions_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(ClickHouseDialect::new());
    let executor = noop_executor();
    let schema = build_schema(registry, dialect, executor, test_schema_config()).unwrap();

    let result = schema.execute(
        r#"{ __type(name: "Instructions_Instruction_Program_Accounts_Element") { fields { name } } }"#
    ).await;
    let data = result.data.into_json().unwrap();
    let fields = data["__type"]["fields"].as_array()
        .expect("Accounts Element type should exist");
    let names: Vec<&str> = fields.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Address"), "Element should have Address field, got: {:?}", names);
    assert!(names.contains(&"Name"), "Element should have Name field, got: {:?}", names);
}
