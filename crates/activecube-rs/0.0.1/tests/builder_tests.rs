use activecube_rs::*;

#[test]
fn test_cube_builder() {
    let cube = CubeBuilder::new("TestCube")
        .schema("test_db")
        .table("{chain}_events")
        .dimension(dim_group("Block", vec![
            dim("Number", "block_num", DimType::Int),
            dim("Time", "block_time", DimType::DateTime),
        ]))
        .dimension(dim("Amount", "amount", DimType::Float))
        .metrics(&["count", "sum", "avg"])
        .selector(selector("date", "block_time", DimType::DateTime))
        .default_filter("active", "true")
        .default_limit(50)
        .max_limit(5000)
        .build();

    assert_eq!(cube.name, "TestCube");
    assert_eq!(cube.schema, "test_db");
    assert_eq!(cube.table_for_chain("eth"), "eth_events");
    assert_eq!(cube.metrics.len(), 3);
    assert_eq!(cube.selectors.len(), 1);
    assert_eq!(cube.default_filters.len(), 1);
    assert_eq!(cube.default_limit, 50);
    assert_eq!(cube.max_limit, 5000);

    let flat = cube.flat_dimensions();
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].0, "Block_Number");
    assert_eq!(flat[1].0, "Block_Time");
    assert_eq!(flat[2].0, "Amount");
}

#[test]
fn test_cube_builder_minimal() {
    let cube = CubeBuilder::new("Minimal")
        .schema("s")
        .table("t")
        .dimension(dim("Id", "id", DimType::Int))
        .build();

    assert_eq!(cube.name, "Minimal");
    assert!(cube.metrics.is_empty());
    assert!(cube.selectors.is_empty());
    assert!(cube.default_filters.is_empty());
    assert_eq!(cube.default_limit, 25);
    assert_eq!(cube.max_limit, 10000);
}
