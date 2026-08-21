use std::sync::Arc;
use activecube_rs::*;
use activecube_rs::schema::generator::{build_schema, QueryExecutor, SchemaConfig};
use activecube_rs::sql::starrocks::StarRocksDialect;

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
        .build()
}

fn noop_executor() -> QueryExecutor {
    Arc::new(|_sql, _bindings| {
        Box::pin(async { Ok(vec![]) })
    })
}

#[tokio::test]
async fn test_schema_builds_successfully() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default());
    assert!(schema.is_ok(), "Schema build failed: {:?}", schema.err());
}

#[tokio::test]
async fn test_schema_introspection_contains_cube() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

    let result = schema.execute("{ __schema { queryType { name } } }").await;
    let data = result.data.into_json().unwrap();
    let query_type_name = data["__schema"]["queryType"]["name"].as_str().unwrap();
    assert_eq!(query_type_name, "Query");
}

#[tokio::test]
async fn test_schema_has_network_enum() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

    let result = schema.execute(r#"{ __type(name: "Network") { enumValues { name } } }"#).await;
    let data = result.data.into_json().unwrap();
    let values = data["__type"]["enumValues"].as_array().unwrap();
    let names: Vec<&str> = values.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"sol"));
    assert!(names.contains(&"eth"));
    assert!(names.contains(&"bsc"));
}

#[tokio::test]
async fn test_schema_has_cube_filter_type() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

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
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor = noop_executor();

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

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
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor: QueryExecutor = Arc::new(move |sql, _bindings| {
        called_clone.store(true, Ordering::SeqCst);
        assert!(sql.contains("FROM `test_db`.`sol_trades`"), "SQL should target sol_trades, got: {sql}");
        assert!(sql.contains("`success` = ?"), "SQL should have default filter, got: {sql}");
        Box::pin(async { Ok(vec![]) })
    });

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

    let result = schema.execute(r#"
        {
            TestTrades(network: sol, limit: { count: 5 }) {
                Block { Number Date }
                Trade { Amount Symbol }
            }
        }
    "#).await;

    assert!(result.errors.is_empty(), "Query errors: {:?}", result.errors);
    assert!(called.load(Ordering::SeqCst), "Executor should have been called");
}

#[tokio::test]
async fn test_query_with_where_filter() {
    let registry = CubeRegistry::from_cubes(vec![test_cube()]);
    let dialect: Arc<dyn SqlDialect> = Arc::new(StarRocksDialect::new());
    let executor: QueryExecutor = Arc::new(move |sql, bindings| {
        assert!(sql.contains("`amount` > ?"), "SQL should have amount filter, got: {sql}");
        let has_1000 = bindings.iter().any(|b| matches!(b, SqlValue::Float(f) if *f == 1000.0));
        assert!(has_1000, "Bindings should contain 1000.0, got: {:?}", bindings);
        Box::pin(async { Ok(vec![]) })
    });

    let schema = build_schema(registry, dialect, executor, SchemaConfig::default()).unwrap();

    let result = schema.execute(r#"
        {
            TestTrades(
                network: eth
                where: { Trade: { Amount: { gt: 1000.0 } } }
                limit: { count: 10 }
            ) {
                Trade { Amount }
            }
        }
    "#).await;

    assert!(result.errors.is_empty(), "Query errors: {:?}", result.errors);
}
