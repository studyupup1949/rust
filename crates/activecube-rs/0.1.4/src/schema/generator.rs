use std::collections::HashSet;
use std::sync::Arc;
use async_graphql::dynamic::*;
use async_graphql::Value;

use crate::compiler;
use crate::compiler::ir::{SqlValue, JoinExpr, SelectExpr};
use crate::cube::definition::{CubeDefinition, DimType, DimensionNode};
use crate::cube::registry::CubeRegistry;
use crate::response::RowMap;
use crate::schema::filter_types;
use crate::sql::dialect::SqlDialect;
use crate::stats::{QueryStats, StatsCallback};

/// Async function type that executes a compiled SQL query and returns rows.
/// The service layer provides this — the library never touches a database directly.
pub type QueryExecutor = Arc<
    dyn Fn(String, Vec<SqlValue>) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<RowMap>, String>> + Send>,
    > + Send + Sync,
>;

/// Configuration for supported networks (chains) and optional stats collection.
pub struct SchemaConfig {
    pub networks: Vec<String>,
    pub root_query_name: String,
    /// Optional callback invoked after each cube query with execution metadata.
    /// Used by application layer for billing, observability, etc.
    pub stats_callback: Option<StatsCallback>,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            networks: vec!["sol", "eth", "bsc"]
                .into_iter().map(String::from).collect(),
            root_query_name: "ChainStream".to_string(),
            stats_callback: None,
        }
    }
}

/// Build a complete async-graphql dynamic schema from registry + dialect + executor.
pub fn build_schema(
    registry: CubeRegistry,
    dialect: Arc<dyn SqlDialect>,
    executor: QueryExecutor,
    config: SchemaConfig,
) -> Result<Schema, SchemaError> {
    let mut builder = Schema::build("Query", None, None);

    let mut network_enum = Enum::new("Network")
        .description("Blockchain network to query");
    for net in &config.networks {
        network_enum = network_enum.item(EnumItem::new(net));
    }
    builder = builder.register(network_enum);
    builder = builder.register(filter_types::build_limit_input());

    builder = builder.register(
        InputObject::new("LimitByInput")
            .description("Limit results per group (similar to ClickHouse LIMIT BY)")
            .field(InputValue::new("by", TypeRef::named_nn(TypeRef::STRING))
                .description("Comma-separated dimension names to group by"))
            .field(InputValue::new("count", TypeRef::named_nn(TypeRef::INT))
                .description("Maximum rows per group"))
            .field(InputValue::new("offset", TypeRef::named(TypeRef::INT))
                .description("Rows to skip per group")),
    );

    builder = builder.register(
        Enum::new("OrderDirection")
            .description("Sort direction")
            .item(EnumItem::new("ASC").description("Ascending"))
            .item(EnumItem::new("DESC").description("Descending")),
    );

    for input in filter_types::build_filter_primitives() {
        builder = builder.register(input);
    }

    // Cubes are top-level Query fields, each with a required `network` argument.
    // Query pattern: `query { DEXTrades(network: sol, limit: ...) { ... } }`
    let mut query = Object::new("Query");

    for cube in registry.cubes() {
        let types = build_cube_types(cube);
        for obj in types.objects { builder = builder.register(obj); }
        for inp in types.inputs { builder = builder.register(inp); }
        for en in types.enums { builder = builder.register(en); }

        let cube_name = cube.name.clone();
        let dialect_clone = dialect.clone();
        let executor_clone = executor.clone();
        let stats_cb = config.stats_callback.clone();

        let orderby_list_input_name = format!("{}OrderByInput", cube.name);

        let cube_description = cube.description.clone();
        let mut field = Field::new(
            &cube.name,
            TypeRef::named_nn_list_nn(format!("{}Record", cube.name)),
            move |ctx| {
                let cube_name = cube_name.clone();
                let dialect = dialect_clone.clone();
                let executor = executor_clone.clone();
                let stats_cb = stats_cb.clone();
                FieldFuture::new(async move {
                    let registry = ctx.ctx.data::<CubeRegistry>()?;
                    let network_val = ctx.args.try_get("network")?;
                    let network = network_val.enum_name()
                        .map_err(|_| async_graphql::Error::new("network must be a Network enum value"))?;

                    let cube_def = registry.get(&cube_name).ok_or_else(|| {
                        async_graphql::Error::new(format!("Unknown cube: {cube_name}"))
                    })?;

                    let metric_requests = extract_metric_requests(&ctx, cube_def);
                    let requested = extract_requested_fields(&ctx, cube_def);
                    let mut ir = compiler::parser::parse_cube_query(
                        cube_def,
                        network,
                        &ctx.args,
                        &metric_requests,
                        Some(requested),
                    )?;

                    // Resolve join fields from the selection set
                    let mut join_idx = 0usize;
                    for sub_field in ctx.ctx.field().selection_set() {
                        let fname = sub_field.name().to_string();
                        let join_def = cube_def.joins.iter().find(|j| j.field_name == fname);
                        if let Some(jd) = join_def {
                            if let Some(target_cube) = registry.get(&jd.target_cube) {
                                let join_expr = build_join_expr(
                                    jd, target_cube, &sub_field, network, join_idx,
                                );
                                ir.joins.push(join_expr);
                                join_idx += 1;
                            }
                        }
                    }

                    let validated = compiler::validator::validate(ir)?;
                    let result = dialect.compile(&validated);
                    let sql = result.sql;
                    let bindings = result.bindings;

                    let rows = executor(sql.clone(), bindings).await.map_err(|e| {
                        async_graphql::Error::new(format!("Query execution failed: {e}"))
                    })?;

                    // Remap aliased columns back to original names for resolvers
                    let rows = if result.alias_remap.is_empty() {
                        rows
                    } else {
                        rows.into_iter().map(|mut row| {
                            for (alias, original) in &result.alias_remap {
                                if let Some(val) = row.shift_remove(alias) {
                                    row.entry(original.clone()).or_insert(val);
                                }
                            }
                            row
                        }).collect()
                    };

                    // Restructure join data: extract _jN.xxx columns into nested objects
                    let rows: Vec<RowMap> = if validated.joins.is_empty() {
                        rows
                    } else {
                        rows.into_iter().map(|mut row| {
                            for join in &validated.joins {
                                let prefix = format!("{}.", join.alias);
                                let mut sub_row = RowMap::new();
                                let keys: Vec<String> = row.keys()
                                    .filter(|k| k.starts_with(&prefix))
                                    .cloned()
                                    .collect();
                                for key in keys {
                                    if let Some(val) = row.shift_remove(&key) {
                                        sub_row.insert(key[prefix.len()..].to_string(), val);
                                    }
                                }
                                let obj: serde_json::Map<String, serde_json::Value> =
                                    sub_row.into_iter().collect();
                                row.insert(
                                    join.join_field.clone(),
                                    serde_json::Value::Object(obj),
                                );
                            }
                            row
                        }).collect()
                    };

                    let effective_cb = ctx.ctx.data::<StatsCallback>().ok().cloned()
                        .or_else(|| stats_cb.clone());
                    if let Some(cb) = effective_cb {
                        let stats = QueryStats::from_ir(&validated, rows.len(), &sql);
                        cb(stats);
                    }

                    let values: Vec<FieldValue> = rows.into_iter().map(FieldValue::owned_any).collect();
                    Ok(Some(FieldValue::list(values)))
                })
            },
        );
        if !cube_description.is_empty() {
            field = field.description(&cube_description);
        }
        field = field
            .argument(InputValue::new("network", TypeRef::named_nn("Network"))
                .description("Blockchain network to query"))
            .argument(InputValue::new("where", TypeRef::named(format!("{}Filter", cube.name)))
                .description("Filter conditions"))
            .argument(InputValue::new("limit", TypeRef::named("LimitInput"))
                .description("Pagination control"))
            .argument(InputValue::new("limitBy", TypeRef::named("LimitByInput"))
                .description("Per-group row limit"))
            .argument(InputValue::new("orderBy", TypeRef::named(format!("{}OrderBy", cube.name)))
                .description("Sort order (single column)"))
            .argument(InputValue::new("orderByList", TypeRef::named_list(&orderby_list_input_name))
                .description("Sort order (multiple columns)"));

        for sel in &cube.selectors {
            let filter_type = dim_type_to_filter_name(&sel.dim_type);
            field = field.argument(InputValue::new(&sel.graphql_name, TypeRef::named(filter_type))
                .description(format!("Shorthand filter for {}", sel.graphql_name)));
        }

        query = query.field(field);
    }

    let metadata_registry = Arc::new(registry.clone());
    let metadata_field = Field::new(
        "_cubeMetadata",
        TypeRef::named_nn(TypeRef::STRING),
        move |_ctx| {
            let reg = metadata_registry.clone();
            FieldFuture::new(async move {
                    let metadata: Vec<serde_json::Value> = reg.cubes().map(|cube| {
                    serde_json::json!({
                        "name": cube.name,
                        "description": cube.description,
                        "schema": cube.schema,
                        "tablePattern": cube.table_pattern,
                        "metrics": cube.metrics,
                        "selectors": cube.selectors.iter().map(|s| {
                            serde_json::json!({
                                "name": s.graphql_name,
                                "column": s.column,
                                "type": format!("{:?}", s.dim_type),
                            })
                        }).collect::<Vec<_>>(),
                        "dimensions": serialize_dims(&cube.dimensions),
                        "joins": cube.joins.iter().map(|j| {
                            serde_json::json!({
                                "field": j.field_name,
                                "target": j.target_cube,
                            })
                        }).collect::<Vec<_>>(),
                        "defaultLimit": cube.default_limit,
                        "maxLimit": cube.max_limit,
                    })
                }).collect();
                let json = serde_json::to_string(&metadata).unwrap_or_default();
                Ok(Some(FieldValue::value(Value::from(json))))
            })
        },
    )
    .description("Internal: returns JSON metadata about all cubes");
    query = query.field(metadata_field);

    builder = builder.register(query);
    builder = builder.data(registry);

    builder.finish()
}

fn serialize_dims(dims: &[DimensionNode]) -> serde_json::Value {
    serde_json::Value::Array(dims.iter().map(|d| match d {
        DimensionNode::Leaf(dim) => {
            let mut obj = serde_json::json!({
                "name": dim.graphql_name,
                "column": dim.column,
                "type": format!("{:?}", dim.dim_type),
            });
            if let Some(desc) = &dim.description {
                obj["description"] = serde_json::Value::String(desc.clone());
            }
            obj
        },
        DimensionNode::Group { graphql_name, description, children } => {
            let mut obj = serde_json::json!({
                "name": graphql_name,
                "children": serialize_dims(children),
            });
            if let Some(desc) = description {
                obj["description"] = serde_json::Value::String(desc.clone());
            }
            obj
        },
    }).collect())
}

/// Extract metric requests from the GraphQL selection set by inspecting
/// child fields. If a user selects `count(of: "Trade_Buy_Amount")`, we find
/// the "count" field in the selection set and extract its `of` argument.
fn extract_metric_requests(
    ctx: &async_graphql::dynamic::ResolverContext,
    cube: &CubeDefinition,
) -> Vec<compiler::parser::MetricRequest> {
    let mut requests = Vec::new();

    for sub_field in ctx.ctx.field().selection_set() {
        let name = sub_field.name();
        if !cube.metrics.contains(&name.to_string()) {
            continue;
        }

        let args = match sub_field.arguments() {
            Ok(args) => args,
            Err(_) => continue,
        };

        let of_dimension = args
            .iter()
            .find(|(k, _)| k.as_str() == "of")
            .and_then(|(_, v)| match v {
                async_graphql::Value::Enum(e) => Some(e.to_string()),
                async_graphql::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "*".to_string());

        let select_where_value = args
            .iter()
            .find(|(k, _)| k.as_str() == "selectWhere")
            .map(|(_, v)| v.clone());

        let condition_filter = args
            .iter()
            .find(|(k, _)| k.as_str() == "if")
            .and_then(|(_, v)| {
                compiler::filter::parse_filter_from_value(v, &cube.dimensions).ok()
                    .and_then(|f| if f.is_empty() { None } else { Some(f) })
            });

        requests.push(compiler::parser::MetricRequest {
            function: name.to_string(),
            of_dimension,
            select_where_value,
            condition_filter,
        });
    }

    requests
}

fn extract_requested_fields(
    ctx: &async_graphql::dynamic::ResolverContext,
    cube: &CubeDefinition,
) -> HashSet<String> {
    let mut fields = HashSet::new();
    collect_selection_paths(&ctx.ctx.field(), "", &mut fields, &cube.metrics);
    fields
}

fn collect_selection_paths(
    field: &async_graphql::SelectionField<'_>,
    prefix: &str,
    out: &mut HashSet<String>,
    metrics: &[String],
) {
    for sub in field.selection_set() {
        let name = sub.name();
        if metrics.iter().any(|m| m == name) {
            continue;
        }
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}_{name}")
        };
        let has_children = sub.selection_set().next().is_some();
        if has_children {
            collect_selection_paths(&sub, &path, out, metrics);
        } else {
            out.insert(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Per-Cube GraphQL type generation
// ---------------------------------------------------------------------------

struct CubeTypes {
    objects: Vec<Object>,
    inputs: Vec<InputObject>,
    enums: Vec<Enum>,
}

fn build_cube_types(cube: &CubeDefinition) -> CubeTypes {
    let record_name = format!("{}Record", cube.name);
    let filter_name = format!("{}Filter", cube.name);
    let orderby_name = format!("{}OrderBy", cube.name);

    let mut record_fields: Vec<Field> = Vec::new();
    let mut filter_fields: Vec<InputValue> = Vec::new();
    let mut orderby_items: Vec<String> = Vec::new();
    let mut extra_objects: Vec<Object> = Vec::new();
    let mut extra_inputs: Vec<InputObject> = Vec::new();

    filter_fields.push(InputValue::new("any", TypeRef::named_list(&filter_name))
        .description("OR combinator — matches if any sub-filter matches"));

    {
        let mut collector = DimCollector {
            cube_name: &cube.name,
            record_fields: &mut record_fields,
            filter_fields: &mut filter_fields,
            orderby_items: &mut orderby_items,
            extra_objects: &mut extra_objects,
            extra_inputs: &mut extra_inputs,
        };
        for node in &cube.dimensions {
            collect_dimension_types(node, "", &mut collector);
        }
    }

    let flat_dims = cube.flat_dimensions();
    let mut metric_enums: Vec<Enum> = Vec::new();
    let metric_descriptions: std::collections::HashMap<&str, &str> = [
        ("count", "Count of rows or distinct values"),
        ("sum", "Sum of values"),
        ("avg", "Average of values"),
        ("min", "Minimum value"),
        ("max", "Maximum value"),
        ("uniq", "Count of unique (distinct) values"),
    ].into_iter().collect();

    for metric in &cube.metrics {
        let select_where_name = format!("{}_{}_SelectWhere", cube.name, metric);
        extra_inputs.push(
            InputObject::new(&select_where_name)
                .description(format!("Post-aggregation filter for {} (HAVING clause)", metric))
                .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)).description("Greater than"))
                .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)).description("Greater than or equal to"))
                .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)).description("Less than"))
                .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)).description("Less than or equal to"))
                .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)).description("Equal to")),
        );

        let of_enum_name = format!("{}_{}_Of", cube.name, metric);
        let mut of_enum = Enum::new(&of_enum_name)
            .description(format!("Dimension to apply {} aggregation on", metric));
        for (path, _) in &flat_dims { of_enum = of_enum.item(EnumItem::new(path)); }
        metric_enums.push(of_enum);

        let metric_clone = metric.clone();
        let metric_desc = metric_descriptions.get(metric.as_str())
            .copied()
            .unwrap_or("Aggregate metric");
        let metric_field = Field::new(metric, TypeRef::named(TypeRef::FLOAT), move |ctx| {
            let metric_key = metric_clone.clone();
            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                let key = format!("__{metric_key}");
                let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                Ok(Some(FieldValue::value(json_to_gql_value(val))))
            })
        })
        .description(metric_desc)
        .argument(InputValue::new("of", TypeRef::named(&of_enum_name))
            .description("Dimension to aggregate on (default: all rows)"))
        .argument(InputValue::new("selectWhere", TypeRef::named(&select_where_name))
            .description("Post-aggregation filter (HAVING)"))
        .argument(InputValue::new("if", TypeRef::named(&filter_name))
            .description("Conditional filter for this metric"));

        record_fields.push(metric_field);
    }

    // Add join fields: joinXxx returns the target cube's Record type
    for jd in &cube.joins {
        let target_record_name = format!("{}Record", jd.target_cube);
        let field_name_owned = jd.field_name.clone();
        let mut join_field = Field::new(
            &jd.field_name,
            TypeRef::named(&target_record_name),
            move |ctx| {
                let field_name = field_name_owned.clone();
                FieldFuture::new(async move {
                    let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                    if let Some(serde_json::Value::Object(obj)) = row.get(&field_name) {
                        let sub_row: RowMap = obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        Ok(Some(FieldValue::owned_any(sub_row)))
                    } else {
                        Ok(Some(FieldValue::value(Value::Null)))
                    }
                })
            },
        );
        if let Some(desc) = &jd.description {
            join_field = join_field.description(desc);
        }
        record_fields.push(join_field);
    }

    let mut record = Object::new(&record_name);
    for f in record_fields { record = record.field(f); }

    let mut filter = InputObject::new(&filter_name)
        .description(format!("Filter conditions for {} query", cube.name));
    for f in filter_fields { filter = filter.field(f); }

    let mut orderby = Enum::new(&orderby_name)
        .description(format!("Sort order for {} results (single column)", cube.name));
    for item in &orderby_items { orderby = orderby.item(EnumItem::new(item)); }

    // Multi-column orderBy: {Cube}OrderBy_Field enum + {Cube}OrderByInput
    let field_enum_name = format!("{}_Field", orderby_name);
    let orderby_input_name = format!("{}OrderByInput", cube.name);
    let mut field_enum = Enum::new(&field_enum_name)
        .description(format!("Available fields for {} multi-column sort", cube.name));
    let flat_dims = cube.flat_dimensions();
    for (path, _) in &flat_dims {
        field_enum = field_enum.item(EnumItem::new(path));
    }
    let orderby_input = InputObject::new(&orderby_input_name)
        .description(format!("Multi-column sort input for {}", cube.name))
        .field(InputValue::new("field", TypeRef::named_nn(&field_enum_name))
            .description("Field to sort by"))
        .field(InputValue::new("direction", TypeRef::named("OrderDirection"))
            .description("Sort direction (ASC or DESC)"));

    let mut objects = vec![record]; objects.extend(extra_objects);
    let mut inputs = vec![filter, orderby_input]; inputs.extend(extra_inputs);
    let mut enums = vec![orderby, field_enum]; enums.extend(metric_enums);

    CubeTypes { objects, inputs, enums }
}

struct DimCollector<'a> {
    cube_name: &'a str,
    record_fields: &'a mut Vec<Field>,
    filter_fields: &'a mut Vec<InputValue>,
    orderby_items: &'a mut Vec<String>,
    extra_objects: &'a mut Vec<Object>,
    extra_inputs: &'a mut Vec<InputObject>,
}

fn collect_dimension_types(node: &DimensionNode, prefix: &str, c: &mut DimCollector<'_>) {
    match node {
        DimensionNode::Leaf(dim) => {
            let col = dim.column.clone();
            let is_datetime = dim.dim_type == DimType::DateTime;
            let mut leaf_field = Field::new(
                &dim.graphql_name, dim_type_to_typeref(&dim.dim_type),
                move |ctx| {
                    let col = col.clone();
                    FieldFuture::new(async move {
                        let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                        let val = row.get(&col).cloned().unwrap_or(serde_json::Value::Null);
                        let gql_val = if is_datetime {
                            json_to_gql_datetime(val)
                        } else {
                            json_to_gql_value(val)
                        };
                        Ok(Some(FieldValue::value(gql_val)))
                    })
                },
            );
            if let Some(desc) = &dim.description {
                leaf_field = leaf_field.description(desc);
            }
            c.record_fields.push(leaf_field);
            c.filter_fields.push(InputValue::new(&dim.graphql_name, TypeRef::named(dim_type_to_filter_name(&dim.dim_type))));

            let path = if prefix.is_empty() { dim.graphql_name.clone() } else { format!("{}_{}", prefix, dim.graphql_name) };
            c.orderby_items.push(format!("{path}_ASC"));
            c.orderby_items.push(format!("{path}_DESC"));
        }
        DimensionNode::Group { graphql_name, description, children } => {
            let full_path = if prefix.is_empty() { graphql_name.clone() } else { format!("{prefix}_{graphql_name}") };
            let nested_record_name = format!("{}_{full_path}_Record", c.cube_name);
            let nested_filter_name = format!("{}_{full_path}_Filter", c.cube_name);

            let mut child_record_fields: Vec<Field> = Vec::new();
            let mut child_filter_fields: Vec<InputValue> = Vec::new();
            let new_prefix = if prefix.is_empty() { graphql_name.clone() } else { format!("{prefix}_{graphql_name}") };

            let mut child_collector = DimCollector {
                cube_name: c.cube_name,
                record_fields: &mut child_record_fields,
                filter_fields: &mut child_filter_fields,
                orderby_items: c.orderby_items,
                extra_objects: c.extra_objects,
                extra_inputs: c.extra_inputs,
            };
            for child in children {
                collect_dimension_types(child, &new_prefix, &mut child_collector);
            }

            let mut nested_record = Object::new(&nested_record_name);
            for f in child_record_fields { nested_record = nested_record.field(f); }

            let nested_filter_desc = format!("Filter conditions for {}", graphql_name);
            let mut nested_filter = InputObject::new(&nested_filter_name)
                .description(nested_filter_desc);
            for f in child_filter_fields { nested_filter = nested_filter.field(f); }

            let mut group_field = Field::new(graphql_name, TypeRef::named_nn(&nested_record_name), |ctx| {
                FieldFuture::new(async move {
                    let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                    Ok(Some(FieldValue::owned_any(row.clone())))
                })
            });
            if let Some(desc) = description {
                group_field = group_field.description(desc);
            }
            c.record_fields.push(group_field);
            c.filter_fields.push(InputValue::new(graphql_name, TypeRef::named(&nested_filter_name)));
            c.extra_objects.push(nested_record);
            c.extra_inputs.push(nested_filter);
        }
    }
}

fn dim_type_to_typeref(dt: &DimType) -> TypeRef {
    match dt {
        DimType::String | DimType::DateTime => TypeRef::named(TypeRef::STRING),
        DimType::Int => TypeRef::named(TypeRef::INT),
        DimType::Float => TypeRef::named(TypeRef::FLOAT),
        DimType::Bool => TypeRef::named(TypeRef::BOOLEAN),
    }
}

fn dim_type_to_filter_name(dt: &DimType) -> &'static str {
    match dt {
        DimType::String => "StringFilter",
        DimType::Int => "IntFilter",
        DimType::Float => "FloatFilter",
        DimType::DateTime => "DateTimeFilter",
        DimType::Bool => "BoolFilter",
    }
}

pub fn json_to_gql_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Value::from(i) }
            else if let Some(f) = n.as_f64() { Value::from(f) }
            else { Value::from(n.to_string()) }
        }
        serde_json::Value::String(s) => Value::from(s),
        _ => Value::from(v.to_string()),
    }
}

/// Build a JoinExpr from a JoinDef and target cube definition.
/// Inspects the sub-selection of the join field to determine which columns to SELECT.
fn build_join_expr(
    jd: &crate::cube::definition::JoinDef,
    target_cube: &CubeDefinition,
    sub_field: &async_graphql::SelectionField<'_>,
    network: &str,
    join_idx: usize,
) -> JoinExpr {
    let target_flat = target_cube.flat_dimensions();
    let target_table = target_cube.table_for_chain(network);

    let mut requested_paths = HashSet::new();
    collect_selection_paths(sub_field, "", &mut requested_paths, &target_cube.metrics);

    let mut selects: Vec<SelectExpr> = target_flat.iter()
        .filter(|(path, _)| requested_paths.contains(path))
        .map(|(_, dim)| SelectExpr::Column {
            column: dim.column.clone(),
            alias: None,
        })
        .collect();

    if selects.is_empty() {
        selects = target_flat.iter()
            .map(|(_, dim)| SelectExpr::Column { column: dim.column.clone(), alias: None })
            .collect();
    }

    let is_aggregate = target_flat.iter().any(|(_, dim)| dim.column.contains('('));

    let group_by = if is_aggregate {
        let mut gb: Vec<String> = jd.conditions.iter().map(|(_, r)| r.clone()).collect();
        for sel in &selects {
            if let SelectExpr::Column { column, .. } = sel {
                if !column.contains('(') && !gb.contains(column) {
                    gb.push(column.clone());
                }
            }
        }
        gb
    } else {
        vec![]
    };

    JoinExpr {
        schema: target_cube.schema.clone(),
        table: target_table,
        alias: format!("_j{}", join_idx),
        conditions: jd.conditions.clone(),
        selects,
        group_by,
        use_final: target_cube.use_final,
        is_aggregate,
        target_cube: jd.target_cube.clone(),
        join_field: sub_field.name().to_string(),
    }
}

/// Convert a ClickHouse DateTime value to ISO 8601 format.
/// `"2026-03-27 19:06:41.000"` -> `"2026-03-27T19:06:41.000Z"`
fn json_to_gql_datetime(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::String(s) => {
            let iso = if s.contains('T') {
                if s.ends_with('Z') || s.contains('+') { s } else { format!("{s}Z") }
            } else {
                let replaced = s.replacen(' ', "T", 1);
                if replaced.ends_with('Z') { replaced } else { format!("{replaced}Z") }
            };
            Value::from(iso)
        }
        other => json_to_gql_value(other),
    }
}
