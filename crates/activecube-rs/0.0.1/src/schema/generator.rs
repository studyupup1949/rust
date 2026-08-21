use std::sync::Arc;
use async_graphql::dynamic::*;
use async_graphql::Value;

use crate::compiler;
use crate::compiler::ir::SqlValue;
use crate::cube::definition::{CubeDefinition, DimType, DimensionNode};
use crate::cube::registry::CubeRegistry;
use crate::response::RowMap;
use crate::schema::filter_types;
use crate::sql::dialect::SqlDialect;

/// Async function type that executes a compiled SQL query and returns rows.
/// The service layer provides this — the library never touches a database directly.
pub type QueryExecutor = Arc<
    dyn Fn(String, Vec<SqlValue>) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<RowMap>, String>> + Send>,
    > + Send + Sync,
>;

/// Configuration for supported networks (chains).
pub struct SchemaConfig {
    pub networks: Vec<String>,
    pub root_query_name: String,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            networks: vec![
                "sol", "eth", "bsc", "base", "polygon",
                "arbitrum", "optimism", "avalanche", "sui",
            ].into_iter().map(String::from).collect(),
            root_query_name: "ChainStream".to_string(),
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

    // Network enum
    let mut network_enum = Enum::new("Network");
    for net in &config.networks {
        network_enum = network_enum.item(EnumItem::new(net));
    }
    builder = builder.register(network_enum);
    builder = builder.register(filter_types::build_limit_input());

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

        let mut field = Field::new(
            &cube.name,
            TypeRef::named_nn_list_nn(format!("{}Record", cube.name)),
            move |ctx| {
                let cube_name = cube_name.clone();
                let dialect = dialect_clone.clone();
                let executor = executor_clone.clone();
                FieldFuture::new(async move {
                    let registry = ctx.ctx.data::<CubeRegistry>()?;
                    let network_val = ctx.args.try_get("network")?;
                    let network = network_val.enum_name()
                        .map_err(|_| async_graphql::Error::new("network must be a Network enum value"))?;

                    let cube_def = registry.get(&cube_name).ok_or_else(|| {
                        async_graphql::Error::new(format!("Unknown cube: {cube_name}"))
                    })?;

                    let metric_requests = extract_metric_requests(&ctx, cube_def);
                    let ir = compiler::parser::parse_cube_query(cube_def, network, &ctx.args, &metric_requests)?;
                    let validated = compiler::validator::validate(ir)?;
                    let (sql, bindings) = dialect.compile(&validated);

                    let rows = executor(sql, bindings).await.map_err(|e| {
                        async_graphql::Error::new(format!("Query execution failed: {e}"))
                    })?;

                    let values: Vec<FieldValue> = rows.into_iter().map(FieldValue::owned_any).collect();
                    Ok(Some(FieldValue::list(values)))
                })
            },
        )
        .argument(InputValue::new("network", TypeRef::named_nn("Network")))
        .argument(InputValue::new("where", TypeRef::named(format!("{}Filter", cube.name))))
        .argument(InputValue::new("limit", TypeRef::named("LimitInput")))
        .argument(InputValue::new("orderBy", TypeRef::named(format!("{}OrderBy", cube.name))));

        for sel in &cube.selectors {
            let filter_type = dim_type_to_filter_name(&sel.dim_type);
            field = field.argument(InputValue::new(&sel.graphql_name, TypeRef::named(filter_type)));
        }

        query = query.field(field);
    }

    builder = builder.register(query);
    builder = builder.data(registry);

    builder.finish()
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

        requests.push(compiler::parser::MetricRequest {
            function: name.to_string(),
            of_dimension,
            select_where_value,
        });
    }

    requests
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

    filter_fields.push(InputValue::new("any", TypeRef::named_list(&filter_name)));

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
    for metric in &cube.metrics {
        let select_where_name = format!("{}_{}_SelectWhere", cube.name, metric);
        extra_inputs.push(
            InputObject::new(&select_where_name)
                .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)))
                .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)))
                .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)))
                .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)))
                .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING))),
        );

        let of_enum_name = format!("{}_{}_Of", cube.name, metric);
        let mut of_enum = Enum::new(&of_enum_name);
        for (path, _) in &flat_dims { of_enum = of_enum.item(EnumItem::new(path)); }
        metric_enums.push(of_enum);

        let metric_clone = metric.clone();
        let metric_field = Field::new(metric, TypeRef::named(TypeRef::FLOAT), move |ctx| {
            let metric_key = metric_clone.clone();
            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                let key = format!("__{metric_key}");
                let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                Ok(Some(FieldValue::value(json_to_gql_value(val))))
            })
        })
        .argument(InputValue::new("of", TypeRef::named(&of_enum_name)))
        .argument(InputValue::new("selectWhere", TypeRef::named(&select_where_name)));

        record_fields.push(metric_field);
    }

    let mut record = Object::new(&record_name);
    for f in record_fields { record = record.field(f); }

    let mut filter = InputObject::new(&filter_name);
    for f in filter_fields { filter = filter.field(f); }

    let mut orderby = Enum::new(&orderby_name);
    for item in orderby_items { orderby = orderby.item(EnumItem::new(item)); }

    let mut objects = vec![record]; objects.extend(extra_objects);
    let mut inputs = vec![filter]; inputs.extend(extra_inputs);
    let mut enums = vec![orderby]; enums.extend(metric_enums);

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
            let leaf_field = Field::new(
                &dim.graphql_name, dim_type_to_typeref(&dim.dim_type),
                move |ctx| {
                    let col = col.clone();
                    FieldFuture::new(async move {
                        let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                        let val = row.get(&col).cloned().unwrap_or(serde_json::Value::Null);
                        Ok(Some(FieldValue::value(json_to_gql_value(val))))
                    })
                },
            );
            c.record_fields.push(leaf_field);
            c.filter_fields.push(InputValue::new(&dim.graphql_name, TypeRef::named(dim_type_to_filter_name(&dim.dim_type))));

            let path = if prefix.is_empty() { dim.graphql_name.clone() } else { format!("{}_{}", prefix, dim.graphql_name) };
            c.orderby_items.push(format!("{path}_ASC"));
            c.orderby_items.push(format!("{path}_DESC"));
        }
        DimensionNode::Group { graphql_name, children } => {
            let nested_record_name = format!("{}_{graphql_name}_Record", c.cube_name);
            let nested_filter_name = format!("{}_{graphql_name}_Filter", c.cube_name);

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

            let mut nested_filter = InputObject::new(&nested_filter_name);
            for f in child_filter_fields { nested_filter = nested_filter.field(f); }

            let group_field = Field::new(graphql_name, TypeRef::named_nn(&nested_record_name), |ctx| {
                FieldFuture::new(async move {
                    let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                    Ok(Some(FieldValue::owned_any(row.clone())))
                })
            });
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
