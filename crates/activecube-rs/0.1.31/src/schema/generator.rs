use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_graphql::dynamic::*;
use async_graphql::Value;

use crate::compiler;
use crate::compiler::ir::{DimAggType, FilterNode, SqlValue, JoinExpr, SelectExpr, is_aggregate_expr};
use crate::cube::definition::{ChainGroup, CubeDefinition, DimType, DimensionNode, MetricDef};
use crate::cube::registry::CubeRegistry;
use crate::response::RowMap;
use crate::schema::filter_types;
use crate::sql::dialect::SqlDialect;
use crate::stats::{QueryStats, StatsCallback};

/// Canonical key naming for metric SQL aliases. Used by parser and resolver.
pub fn metric_key(alias: &str) -> String { format!("__{alias}") }

/// Canonical key naming for dimension aggregate / time interval SQL aliases.
pub fn dim_agg_key(alias: &str) -> String { format!("__da_{alias}") }

/// Async function type that executes a compiled SQL query and returns rows.
/// The service layer provides this — the library never touches a database directly.
pub type QueryExecutor = Arc<
    dyn Fn(String, Vec<SqlValue>) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<RowMap>, String>> + Send>,
    > + Send + Sync,
>;

/// A custom argument on the chain wrapper field (e.g. dataset, aggregates).
/// The library registers it on the GraphQL wrapper type and stores the resolved
/// value in `ChainContext.extra` for child cube resolvers to read.
#[derive(Debug, Clone)]
pub struct WrapperArg {
    /// GraphQL argument name (e.g. "dataset").
    pub name: String,
    /// Name of a type already registered in the schema (e.g. "Dataset").
    pub type_ref: String,
    /// Description shown in schema introspection.
    pub description: String,
    /// Enum values, used by builder schema generation. None for non-enum types.
    pub values: Option<Vec<String>>,
}

/// Describes one top-level chain wrapper type (e.g. EVM, Solana, Trading).
#[derive(Debug, Clone)]
pub struct ChainGroupConfig {
    /// Wrapper type name in the schema, e.g. "EVM", "Solana", "Trading".
    pub name: String,
    /// Which ChainGroup enum value this config represents.
    pub group: ChainGroup,
    /// Available networks. EVM: ["eth","bsc"]; Solana: ["sol"]; Trading: all chains.
    pub networks: Vec<String>,
    /// If true the wrapper type takes a `network` argument (EVM style).
    /// If false the chain is implicit (Solana style) or cross-chain (Trading).
    pub has_network_arg: bool,
    /// Additional arguments on the wrapper field, passed through to child resolvers
    /// via `ChainContext.extra`.
    pub extra_args: Vec<WrapperArg>,
    /// Custom name for the network enum type registered in the schema.
    /// Defaults to `"{name}Network"` (e.g. `EVMNetwork`) when `None`.
    /// Set to e.g. `Some("evm_network".into())` for Bitquery-compatible naming.
    pub network_enum_name: Option<String>,
    /// Upstream-compatible aliases for network names. Keys are alias values
    /// accepted in the GraphQL `network` argument; values are the canonical
    /// network names used internally (must appear in `networks`).
    ///
    /// Example: `{"matic": "polygon"}` lets Bitquery's legacy `network: matic`
    /// queries transparently resolve to `polygon`. Alias keys are registered
    /// as additional enum items so GraphQL validation accepts them.
    pub network_aliases: HashMap<String, String>,
}

/// Callback to transform the resolved table name based on `ChainContext`.
/// Called after `resolve_table` picks the base table, before SQL compilation.
/// Arguments: `(table_name, chain_context) -> transformed_table_name`.
pub type TableNameTransform = Arc<dyn Fn(&str, &ChainContext) -> String + Send + Sync>;

/// Configuration for supported networks (chains) and optional stats collection.
pub struct SchemaConfig {
    pub chain_groups: Vec<ChainGroupConfig>,
    pub root_query_name: String,
    /// Optional callback invoked after each cube query with execution metadata.
    /// Used by application layer for billing, observability, etc.
    pub stats_callback: Option<StatsCallback>,
    /// Optional hook to transform table names based on chain context (e.g. dataset suffix).
    pub table_name_transform: Option<TableNameTransform>,
    /// Additional global enum types to register in the schema (e.g. Dataset, Aggregates).
    pub extra_types: Vec<Enum>,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            chain_groups: vec![
                ChainGroupConfig {
                    name: "EVM".to_string(),
                    group: ChainGroup::Evm,
                    networks: vec!["eth".into(), "bsc".into()],
                    has_network_arg: true,
                    extra_args: vec![],
                    network_enum_name: None,
                    network_aliases: HashMap::new(),
                },
                ChainGroupConfig {
                    name: "Solana".to_string(),
                    group: ChainGroup::Solana,
                    networks: vec!["sol".into()],
                    has_network_arg: false,
                    extra_args: vec![],
                    network_enum_name: None,
                    network_aliases: HashMap::new(),
                },
                ChainGroupConfig {
                    name: "Trading".to_string(),
                    group: ChainGroup::Trading,
                    networks: vec!["sol".into(), "eth".into(), "bsc".into()],
                    has_network_arg: false,
                    extra_args: vec![],
                    network_enum_name: None,
                    network_aliases: HashMap::new(),
                },
            ],
            root_query_name: "ChainStream".to_string(),
            stats_callback: None,
            table_name_transform: None,
            extra_types: vec![],
        }
    }
}

/// Stored in the parent value of chain wrapper types so cube resolvers can
/// retrieve the active chain without a `network` argument on themselves.
/// Service-layer extra_args values are available in the `extra` map.
pub struct ChainContext {
    pub network: String,
    pub extra: HashMap<String, String>,
}

/// Returns the type-name prefix for a ChainGroup (e.g. "EVM_", "Solana_").
fn chain_group_prefix(group: &ChainGroup, configs: &[ChainGroupConfig]) -> String {
    for cfg in configs {
        if cfg.group == *group {
            return format!("{}_", cfg.name);
        }
    }
    String::new()
}

/// Build a complete async-graphql dynamic schema from registry + dialect + executor.
///
/// Schema shape (Bitquery-aligned):
/// ```graphql
/// type Query {
///   EVM(network: EVMNetwork!): EVM!
///   Solana: Solana!
///   Trading: Trading!
///   _cubeMetadata: String!
/// }
/// ```
pub fn build_schema(
    registry: CubeRegistry,
    dialect: Arc<dyn SqlDialect>,
    executor: QueryExecutor,
    config: SchemaConfig,
) -> Result<Schema, SchemaError> {
    let mut builder = Schema::build("Query", None, None);

    // --- shared input / enum types -------------------------------------------
    builder = builder.register(filter_types::build_limit_input());
    // LimitByInput is now generated per-cube in build_cube_types() with typed enum for `by`
    builder = builder.register(
        Enum::new("OrderDirection")
            .description("Sort direction")
            .item(EnumItem::new("ASC").description("Ascending"))
            .item(EnumItem::new("DESC").description("Descending")),
    );
    for input in filter_types::build_filter_primitives() {
        builder = builder.register(input);
    }
    builder = builder.register(
        Scalar::new("DateTime")
            .description("ISO 8601 date-time string (e.g. \"2026-04-01T00:00:00Z\")"),
    );
    builder = builder.register(
        Scalar::new("ISO8601DateTime")
            .description("ISO 8601 date-time string (alias for DateTime, V1 compatibility)"),
    );
    builder = builder.register(
        Enum::new("TimeUnit")
            .description("Time unit for interval bucketing")
            .item(EnumItem::new("seconds"))
            .item(EnumItem::new("minutes"))
            .item(EnumItem::new("hours"))
            .item(EnumItem::new("days"))
            .item(EnumItem::new("weeks"))
            .item(EnumItem::new("months")),
    );
    builder = builder.register(
        InputObject::new("TimeIntervalInput")
            .description("Time bucketing interval for DateTime dimensions")
            .field(InputValue::new("in", TypeRef::named_nn("TimeUnit")).description("Time unit"))
            .field(InputValue::new("count", TypeRef::named_nn(TypeRef::INT)).description("Number of time units")),
    );
    builder = builder.register(
        Scalar::new("StringOrNumber")
            .description("Accepts both string and numeric values (coerced to string for comparison)"),
    );
    builder = builder.register(
        InputObject::new("DimSelectWhere")
            .description("Post-aggregation filter for dimension values (HAVING clause)")
            .field(InputValue::new("gt", TypeRef::named("StringOrNumber")).description("Greater than"))
            .field(InputValue::new("ge", TypeRef::named("StringOrNumber")).description("Greater than or equal"))
            .field(InputValue::new("lt", TypeRef::named("StringOrNumber")).description("Less than"))
            .field(InputValue::new("le", TypeRef::named("StringOrNumber")).description("Less than or equal"))
            .field(InputValue::new("eq", TypeRef::named("StringOrNumber")).description("Equal to"))
            .field(InputValue::new("ne", TypeRef::named("StringOrNumber")).description("Not equal to")),
    );

    for extra_enum in config.extra_types {
        builder = builder.register(extra_enum);
    }

    // --- per-group network enums ---------------------------------------------
    for grp in &config.chain_groups {
        if grp.has_network_arg {
            let enum_name = grp.network_enum_name.clone()
                .unwrap_or_else(|| format!("{}Network", grp.name));
            let mut net_enum = Enum::new(&enum_name)
                .description(format!("{} network selector", grp.name));
            for net in &grp.networks {
                net_enum = net_enum.item(EnumItem::new(net));
            }
            // Register alias keys as additional enum items so GraphQL validation
            // accepts upstream-compatible values (e.g. `matic` → `polygon`).
            for alias in grp.network_aliases.keys() {
                if !grp.networks.contains(alias) {
                    net_enum = net_enum.item(
                        EnumItem::new(alias)
                            .description(format!(
                                "Alias for `{}` (upstream compatibility)",
                                grp.network_aliases.get(alias).cloned().unwrap_or_default()
                            )),
                    );
                }
            }
            builder = builder.register(net_enum);
        }
    }

    // --- register cube types (shared or per-chain when overrides exist) -------
    let mut registered_cubes: HashSet<String> = HashSet::new();
    for cube in registry.cubes() {
        if registered_cubes.contains(&cube.name) { continue; }
        registered_cubes.insert(cube.name.clone());

        if cube.has_chain_overrides() {
            // Base types (unprefixed) for join references from other cubes
            let base_types = build_cube_types(cube, "", None);
            for obj in base_types.objects { builder = builder.register(obj); }
            for inp in base_types.inputs { builder = builder.register(inp); }
            for en in base_types.enums { builder = builder.register(en); }
            for un in base_types.unions { builder = builder.register(un); }

            // Per-chain prefixed types for wrapper fields
            for grp in &cube.chain_groups {
                let prefix = chain_group_prefix(grp, &config.chain_groups);
                let dims = cube.dimensions_for_chain(grp);
                let types = build_cube_types(cube, &prefix, Some(&dims));
                for obj in types.objects { builder = builder.register(obj); }
                for inp in types.inputs { builder = builder.register(inp); }
                for en in types.enums { builder = builder.register(en); }
                for un in types.unions { builder = builder.register(un); }
            }
        } else {
            let types = build_cube_types(cube, "", None);
            for obj in types.objects { builder = builder.register(obj); }
            for inp in types.inputs { builder = builder.register(inp); }
            for en in types.enums { builder = builder.register(en); }
            for un in types.unions { builder = builder.register(un); }
        }
    }

    // --- build chain wrapper types + Query fields ----------------------------
    let mut query = Object::new("Query");

    for grp in &config.chain_groups {
        let wrapper_type_name = grp.name.clone();

        // Build the wrapper Object (e.g. "EVM", "Solana", "Trading") and attach
        // cube fields that belong to this group.
        let mut wrapper_obj = Object::new(&wrapper_type_name);

        for cube in registry.cubes() {
            if !cube.chain_groups.contains(&grp.group) { continue; }

            let prefix = if cube.has_chain_overrides() {
                chain_group_prefix(&grp.group, &config.chain_groups)
            } else {
                String::new()
            };
            let cube_field = build_cube_field(
                cube,
                &prefix,
                dialect.clone(),
                executor.clone(),
                config.stats_callback.clone(),
            );
            wrapper_obj = wrapper_obj.field(cube_field);
        }
        builder = builder.register(wrapper_obj);

        // Wrapper resolver on Query — stores ChainContext for child resolvers.
        let default_network = grp.networks.first().cloned().unwrap_or_default();
        let has_net_arg = grp.has_network_arg;
        let net_enum_name = grp.network_enum_name.clone()
            .unwrap_or_else(|| format!("{}Network", grp.name));
        let wrapper_type_for_resolver = wrapper_type_name.clone();
        let extra_arg_names: Vec<String> = grp.extra_args.iter().map(|a| a.name.clone()).collect();
        let network_aliases = grp.network_aliases.clone();

        let mut wrapper_field = Field::new(
            &wrapper_type_name,
            TypeRef::named_nn(&wrapper_type_name),
            move |ctx| {
                let default = default_network.clone();
                let has_arg = has_net_arg;
                let arg_names = extra_arg_names.clone();
                let aliases = network_aliases.clone();
                FieldFuture::new(async move {
                    let network = if has_arg {
                        ctx.args.try_get("network")
                            .ok()
                            .and_then(|v| v.enum_name().ok().map(|s| s.to_string()))
                            .unwrap_or(default)
                    } else {
                        default
                    };
                    // Resolve alias → canonical network name (e.g. matic → polygon).
                    let network = aliases.get(&network).cloned().unwrap_or(network);
                    let mut extra = HashMap::new();
                    for arg_name in &arg_names {
                        if let Ok(val) = ctx.args.try_get(arg_name) {
                            let resolved = val.enum_name().ok().map(|s| s.to_string())
                                .or_else(|| val.boolean().ok().map(|b| b.to_string()))
                                .or_else(|| val.string().ok().map(|s| s.to_string()));
                            if let Some(v) = resolved {
                                extra.insert(arg_name.clone(), v);
                            }
                        }
                    }
                    Ok(Some(FieldValue::owned_any(ChainContext { network, extra })))
                })
            },
        );

        if grp.has_network_arg {
            wrapper_field = wrapper_field.argument(
                InputValue::new("network", TypeRef::named(&net_enum_name))
                    .description(format!("{} network to query", wrapper_type_for_resolver)),
            );
        }
        for wa in &grp.extra_args {
            wrapper_field = wrapper_field.argument(
                InputValue::new(&wa.name, TypeRef::named(&wa.type_ref))
                    .description(&wa.description),
            );
        }

        query = query.field(wrapper_field);
    }

    // --- _cubeMetadata -------------------------------------------------------
    let metadata_registry = Arc::new(registry.clone());
    let metadata_groups: Vec<(String, ChainGroup)> = config.chain_groups.iter()
        .map(|g| (g.name.clone(), g.group.clone()))
        .collect();
    let metadata_field = Field::new(
        "_cubeMetadata",
        TypeRef::named_nn(TypeRef::STRING),
        move |_ctx| {
            let reg = metadata_registry.clone();
            let groups = metadata_groups.clone();
            FieldFuture::new(async move {
                let mut group_metadata: Vec<serde_json::Value> = Vec::new();
                for (group_name, group_enum) in &groups {
                    let cubes_in_group: Vec<serde_json::Value> = reg.cubes()
                        .filter(|c| c.chain_groups.contains(group_enum))
                        .map(serialize_cube_metadata)
                        .collect();
                    group_metadata.push(serde_json::json!({
                        "group": group_name,
                        "cubes": cubes_in_group,
                    }));
                }
                let json = serde_json::to_string(&group_metadata).unwrap_or_default();
                Ok(Some(FieldValue::value(Value::from(json))))
            })
        },
    )
    .description("Internal: returns JSON metadata about all cubes grouped by chain");
    query = query.field(metadata_field);

    builder = builder.register(query);
    builder = builder.data(registry);

    if let Some(transform) = config.table_name_transform.clone() {
        builder = builder.data(transform);
    }

    builder.finish()
}

fn serialize_cube_metadata(cube: &CubeDefinition) -> serde_json::Value {
    serde_json::json!({
        "name": cube.name,
        "description": cube.description,
        "schema": cube.schema,
        "tablePattern": cube.table_pattern,
        "chainGroups": cube.chain_groups.iter().map(|g| format!("{:?}", g)).collect::<Vec<_>>(),
        "metrics": cube.metrics.iter().map(|m| {
            let mut obj = serde_json::json!({
                "name": m.name,
                "returnType": format!("{:?}", m.return_type),
            });
            if let Some(ref tmpl) = m.expression_template {
                obj["expressionTemplate"] = serde_json::Value::String(tmpl.clone());
            }
            if let Some(ref desc) = m.description {
                obj["description"] = serde_json::Value::String(desc.clone());
            }
            obj
        }).collect::<Vec<_>>(),
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
                "joinType": format!("{:?}", j.join_type),
            })
        }).collect::<Vec<_>>(),
        "tableRoutes": cube.table_routes.iter().map(|r| {
            serde_json::json!({
                "schema": r.schema,
                "tablePattern": r.table_pattern,
                "availableColumns": r.available_columns,
                "priority": r.priority,
            })
        }).collect::<Vec<_>>(),
        "defaultLimit": cube.default_limit,
        "maxLimit": cube.max_limit,
    })
}

/// Build a single cube Field that reads `network` from the parent ChainContext.
/// `type_prefix` is used for cubes with chain_overrides (e.g. "EVM_", "Solana_").
fn build_cube_field(
    cube: &CubeDefinition,
    type_prefix: &str,
    dialect: Arc<dyn SqlDialect>,
    executor: QueryExecutor,
    stats_cb: Option<StatsCallback>,
) -> Field {
    let cube_name = cube.name.clone();
    let prefixed_name = format!("{}{}", type_prefix, cube.name);
    let orderby_input_name = format!("{}OrderByInput", prefixed_name);
    let cube_description = cube.description.clone();

    let mut field = Field::new(
        &cube.name,
        TypeRef::named_nn_list_nn(format!("{}Record", prefixed_name)),
        move |ctx| {
            let cube_name = cube_name.clone();
            let dialect = dialect.clone();
            let executor = executor.clone();
            let stats_cb = stats_cb.clone();
            FieldFuture::new(async move {
                let registry = ctx.ctx.data::<CubeRegistry>()?;

                let chain_ctx = ctx.parent_value
                    .try_downcast_ref::<ChainContext>().ok();
                let network = chain_ctx
                    .map(|c| c.network.as_str())
                    .unwrap_or("sol");

                let cube_def = registry.get(&cube_name).ok_or_else(|| {
                    async_graphql::Error::new(format!("Unknown cube: {cube_name}"))
                })?;

                let metric_requests = extract_metric_requests(&ctx, cube_def);
                let quantile_requests = extract_quantile_requests(&ctx);
                let calculate_requests = extract_calculate_requests(&ctx);
                let field_aliases = extract_field_aliases(&ctx, cube_def);
                let dim_agg_requests = extract_dim_agg_requests(&ctx, cube_def);
                let time_intervals = extract_time_interval_requests(&ctx, cube_def);
                let requested = extract_requested_fields(&ctx, cube_def);
                let mut ir = compiler::parser::parse_cube_query(
                    cube_def,
                    network,
                    &ctx.args,
                    &metric_requests,
                    &quantile_requests,
                    &calculate_requests,
                    &field_aliases,
                    &dim_agg_requests,
                    &time_intervals,
                    Some(requested),
                )?;

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

                if let Ok(transform) = ctx.ctx.data::<TableNameTransform>() {
                    if let Some(cctx) = chain_ctx {
                        ir.table = transform(&ir.table, cctx);
                    }
                }

                let validated = compiler::validator::validate(ir)?;
                let result = dialect.compile(&validated);
                let sql = result.sql;
                let bindings = result.bindings;

                let rows = executor(sql.clone(), bindings).await.map_err(|e| {
                    async_graphql::Error::new(format!("Query execution failed: {e}"))
                })?;

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
        .argument(InputValue::new("where", TypeRef::named(format!("{}Filter", prefixed_name)))
            .description("Filter conditions"))
        .argument(InputValue::new("limit", TypeRef::named("LimitInput"))
            .description("Pagination control"))
        .argument(InputValue::new("limitBy", TypeRef::named(format!("{}LimitByInput", prefixed_name)))
            .description("Per-group row limit"))
        .argument(InputValue::new("orderBy", TypeRef::named(&orderby_input_name))
            .description("Sort order (Bitquery-compatible)"));

    for sel in &cube.selectors {
        let scalar_type = dim_type_to_typeref(&sel.dim_type);
        field = field.argument(InputValue::new(&sel.graphql_name, scalar_type)
            .description(format!("Shorthand filter for {}", sel.graphql_name)));
    }

    field
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
        DimensionNode::Group { graphql_name, description, children, .. } => {
            let mut obj = serde_json::json!({
                "name": graphql_name,
                "children": serialize_dims(children),
            });
            if let Some(desc) = description {
                obj["description"] = serde_json::Value::String(desc.clone());
            }
            obj
        },
        DimensionNode::Array { graphql_name, description, children } => {
            let fields: Vec<serde_json::Value> = children.iter().map(|f| {
                let type_value = match &f.field_type {
                    crate::cube::definition::ArrayFieldType::Scalar(dt) => serde_json::json!({
                        "kind": "scalar",
                        "scalarType": format!("{:?}", dt),
                    }),
                    crate::cube::definition::ArrayFieldType::Union(variants) => serde_json::json!({
                        "kind": "union",
                        "variants": variants.iter().map(|v| serde_json::json!({
                            "typeName": v.type_name,
                            "fieldName": v.field_name,
                            "sourceType": format!("{:?}", v.source_type),
                        })).collect::<Vec<_>>(),
                    }),
                    crate::cube::definition::ArrayFieldType::Group(children) => serde_json::json!({
                        "kind": "group",
                        "children": children.iter().map(|c| c.graphql_name.clone()).collect::<Vec<_>>(),
                    }),
                };
                let mut field_obj = serde_json::json!({
                    "name": f.graphql_name,
                    "column": f.column,
                    "type": type_value,
                });
                if let Some(desc) = &f.description {
                    field_obj["description"] = serde_json::Value::String(desc.clone());
                }
                field_obj
            }).collect();
            let mut obj = serde_json::json!({
                "name": graphql_name,
                "kind": "array",
                "fields": fields,
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
        if !cube.has_metric(name) {
            continue;
        }

        let args = match sub_field.arguments() {
            Ok(args) => args,
            Err(_) => continue,
        };

        let of_dimension = args
            .iter()
            .find(|(k, _)| k.as_str() == "of")
            .or_else(|| args.iter().find(|(k, _)| k.as_str() == "distinct"))
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

        let gql_alias = sub_field.alias().unwrap_or(name).to_string();
        requests.push(compiler::parser::MetricRequest {
            function: name.to_string(),
            alias: gql_alias,
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
    metrics: &[MetricDef],
) {
    for sub in field.selection_set() {
        let name = sub.name();
        if metrics.iter().any(|m| m.name == name) {
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

/// Maps `{parent_path}_{graphql_alias}` → ClickHouse column name for aliased dimension fields.
/// Used by `calculate` to resolve `$TokenSupplyUpdate_post` → column.
pub type FieldAliasMap = Vec<(String, String)>;

/// Describes a `quantile(of: ..., level: ...)` request.
pub struct QuantileRequest {
    pub alias: String,
    pub of_dimension: String,
    pub level: f64,
}

/// Describes a `calculate(expression: "...")` request.
pub struct CalculateRequest {
    pub alias: String,
    pub expression: String,
}

/// Describes a time interval bucketing request.
/// `Time(interval: {in: minutes, count: 1})` → replaces column with `toStartOfInterval(ts, ...)`
pub struct TimeIntervalRequest {
    pub field_path: String,
    pub graphql_alias: String,
    pub column: String,
    pub unit: String,
    pub count: i64,
}

/// Describes a dimension-level aggregation request extracted from the selection set.
/// `PostBalance(maximum: Block_Slot)` → DimAggRequest { agg_type: ArgMax, ... }
pub struct DimAggRequest {
    pub field_path: String,
    pub graphql_alias: String,
    pub value_column: String,
    pub agg_type: DimAggType,
    pub compare_column: String,
    pub condition_filter: Option<FilterNode>,
    pub select_where_value: Option<async_graphql::Value>,
}

fn extract_quantile_requests(
    ctx: &async_graphql::dynamic::ResolverContext,
) -> Vec<QuantileRequest> {
    let mut requests = Vec::new();
    for sub_field in ctx.ctx.field().selection_set() {
        if sub_field.name() != "quantile" { continue; }
        let args = match sub_field.arguments() {
            Ok(a) => a,
            Err(_) => continue,
        };
        let of_dim = args.iter()
            .find(|(k, _)| k.as_str() == "of")
            .and_then(|(_, v)| match v {
                async_graphql::Value::Enum(e) => Some(e.to_string()),
                async_graphql::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "*".to_string());
        let level = args.iter()
            .find(|(k, _)| k.as_str() == "level")
            .and_then(|(_, v)| match v {
                async_graphql::Value::Number(n) => n.as_f64(),
                _ => None,
            })
            .unwrap_or(0.5);
        let alias = sub_field.alias().unwrap_or("quantile").to_string();
        requests.push(QuantileRequest { alias, of_dimension: of_dim, level });
    }
    requests
}

fn extract_field_aliases(
    ctx: &async_graphql::dynamic::ResolverContext,
    cube: &CubeDefinition,
) -> FieldAliasMap {
    let flat = cube.flat_dimensions();
    let mut aliases = Vec::new();
    collect_field_aliases(&ctx.ctx.field(), "", &flat, &cube.metrics, &mut aliases);
    aliases
}

fn collect_field_aliases(
    field: &async_graphql::SelectionField<'_>,
    prefix: &str,
    flat: &[(String, crate::cube::definition::Dimension)],
    metrics: &[MetricDef],
    out: &mut FieldAliasMap,
) {
    for sub in field.selection_set() {
        let name = sub.name();
        if metrics.iter().any(|m| m.name == name) || name == "calculate" || name == "quantile" {
            continue;
        }
        let path = if prefix.is_empty() { name.to_string() } else { format!("{prefix}_{name}") };
        let has_children = sub.selection_set().next().is_some();
        if has_children {
            collect_field_aliases(&sub, &path, flat, metrics, out);
        } else if let Some(alias) = sub.alias() {
            if let Some((_, dim)) = flat.iter().find(|(p, _)| p == &path) {
                let alias_path = if prefix.is_empty() {
                    alias.to_string()
                } else {
                    format!("{prefix}_{alias}")
                };
                out.push((alias_path, dim.column.clone()));
            }
        }
    }
}

fn extract_calculate_requests(
    ctx: &async_graphql::dynamic::ResolverContext,
) -> Vec<CalculateRequest> {
    let mut requests = Vec::new();
    for sub_field in ctx.ctx.field().selection_set() {
        if sub_field.name() != "calculate" { continue; }
        let args = match sub_field.arguments() {
            Ok(a) => a,
            Err(_) => continue,
        };
        let expression = args.iter()
            .find(|(k, _)| k.as_str() == "expression")
            .and_then(|(_, v)| match v {
                async_graphql::Value::String(s) => Some(s.clone()),
                _ => None,
            });
        if let Some(expr) = expression {
            let alias = sub_field.alias().unwrap_or("calculate").to_string();
            requests.push(CalculateRequest { alias, expression: expr });
        }
    }
    requests
}

fn extract_dim_agg_requests(
    ctx: &async_graphql::dynamic::ResolverContext,
    cube: &CubeDefinition,
) -> Vec<DimAggRequest> {
    let flat = cube.flat_dimensions();
    let mut requests = Vec::new();
    collect_dim_agg_paths(&ctx.ctx.field(), "", &flat, &cube.metrics, cube, &mut requests);
    requests
}

fn collect_dim_agg_paths(
    field: &async_graphql::SelectionField<'_>,
    prefix: &str,
    flat: &[(String, crate::cube::definition::Dimension)],
    metrics: &[MetricDef],
    cube: &CubeDefinition,
    out: &mut Vec<DimAggRequest>,
) {
    for sub in field.selection_set() {
        let name = sub.name();
        if metrics.iter().any(|m| m.name == name) {
            continue;
        }
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}_{name}")
        };
        let has_children = sub.selection_set().next().is_some();
        if has_children {
            collect_dim_agg_paths(&sub, &path, flat, metrics, cube, out);
        } else {
            let args = match sub.arguments() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let max_val = args.iter().find(|(k, _)| k.as_str() == "maximum");
            let min_val = args.iter().find(|(k, _)| k.as_str() == "minimum");

            let (agg_type, compare_path) = if let Some((_, v)) = max_val {
                let cp = match v {
                    async_graphql::Value::Enum(e) => e.to_string(),
                    async_graphql::Value::String(s) => s.clone(),
                    _ => continue,
                };
                (DimAggType::ArgMax, cp)
            } else if let Some((_, v)) = min_val {
                let cp = match v {
                    async_graphql::Value::Enum(e) => e.to_string(),
                    async_graphql::Value::String(s) => s.clone(),
                    _ => continue,
                };
                (DimAggType::ArgMin, cp)
            } else {
                continue;
            };

            let value_column = flat.iter()
                .find(|(p, _)| p == &path)
                .map(|(_, dim)| dim.column.clone());
            let compare_column = flat.iter()
                .find(|(p, _)| p == &compare_path)
                .map(|(_, dim)| dim.column.clone());

            if let (Some(vc), Some(cc)) = (value_column, compare_column) {
                let condition_filter = args.iter()
                    .find(|(k, _)| k.as_str() == "if")
                    .and_then(|(_, v)| {
                        compiler::filter::parse_filter_from_value(v, &cube.dimensions).ok()
                            .and_then(|f| if f.is_empty() { None } else { Some(f) })
                    });
                let select_where_value = args.iter()
                    .find(|(k, _)| k.as_str() == "selectWhere")
                    .map(|(_, v)| v.clone());

                let gql_alias = sub.alias()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| path.clone());
                out.push(DimAggRequest {
                    field_path: path,
                    graphql_alias: gql_alias,
                    value_column: vc,
                    agg_type,
                    compare_column: cc,
                    condition_filter,
                    select_where_value,
                });
            }
        }
    }
}

fn extract_time_interval_requests(
    ctx: &async_graphql::dynamic::ResolverContext,
    cube: &CubeDefinition,
) -> Vec<TimeIntervalRequest> {
    let flat = cube.flat_dimensions();
    let mut requests = Vec::new();
    collect_time_interval_paths(&ctx.ctx.field(), "", &flat, &cube.metrics, &mut requests);
    requests
}

fn collect_time_interval_paths(
    field: &async_graphql::SelectionField<'_>,
    prefix: &str,
    flat: &[(String, crate::cube::definition::Dimension)],
    metrics: &[MetricDef],
    out: &mut Vec<TimeIntervalRequest>,
) {
    for sub in field.selection_set() {
        let name = sub.name();
        if metrics.iter().any(|m| m.name == name) { continue; }
        let path = if prefix.is_empty() { name.to_string() } else { format!("{prefix}_{name}") };
        let has_children = sub.selection_set().next().is_some();
        if has_children {
            collect_time_interval_paths(&sub, &path, flat, metrics, out);
        } else {
            let args = match sub.arguments() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let interval_val = args.iter().find(|(k, _)| k.as_str() == "interval");
            if let Some((_, async_graphql::Value::Object(obj))) = interval_val {
                let unit = obj.get("in")
                    .and_then(|v| match v {
                        async_graphql::Value::Enum(e) => Some(e.to_string()),
                        async_graphql::Value::String(s) => Some(s.clone()),
                        _ => None,
                    });
                let count = obj.get("count")
                    .and_then(|v| match v {
                        async_graphql::Value::Number(n) => n.as_i64(),
                        _ => None,
                    });
                if let (Some(unit), Some(count)) = (unit, count) {
                    if let Some((_, dim)) = flat.iter().find(|(p, _)| p == &path) {
                        let gql_alias = sub.alias().unwrap_or(name).to_string();
                        out.push(TimeIntervalRequest {
                            field_path: path,
                            graphql_alias: gql_alias,
                            column: dim.column.clone(),
                            unit,
                            count,
                        });
                    }
                }
            }
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
    unions: Vec<Union>,
}

/// Build GraphQL types for a cube.
/// `type_prefix`: optional prefix for type names (e.g. "EVM_" or "Solana_").
/// `dimensions_override`: if provided, use these dimensions instead of cube.dimensions.
fn build_cube_types(
    cube: &CubeDefinition,
    type_prefix: &str,
    dimensions_override: Option<&[DimensionNode]>,
) -> CubeTypes {
    let prefixed_name = format!("{}{}", type_prefix, cube.name);
    let record_name = format!("{}Record", prefixed_name);
    let filter_name = format!("{}Filter", prefixed_name);
    let compare_enum_name = format!("{}CompareFields", prefixed_name);

    let effective_dims = dimensions_override.unwrap_or(&cube.dimensions);
    let mut flat_out = Vec::new();
    for node in effective_dims {
        crate::cube::definition::collect_leaves_pub(node, "", &mut flat_out);
    }

    let mut compare_enum = Enum::new(&compare_enum_name)
        .description(format!("Fields available for dimension aggregation (maximum/minimum) and ordering in {}", cube.name));
    for (path, _) in &flat_out {
        compare_enum = compare_enum.item(EnumItem::new(path));
    }

    let mut record_fields: Vec<Field> = Vec::new();
    let mut filter_fields: Vec<InputValue> = Vec::new();
    let mut extra_objects: Vec<Object> = Vec::new();
    let mut extra_inputs: Vec<InputObject> = Vec::new();
    let mut extra_enums: Vec<Enum> = Vec::new();
    let mut extra_unions: Vec<Union> = Vec::new();
    let mut registered_enum_names: HashSet<String> = HashSet::new();

    filter_fields.push(InputValue::new("any", TypeRef::named_list(&filter_name))
        .description("OR combinator — matches if any sub-filter matches"));

    {
        let mut collector = DimCollector {
            cube_name: &prefixed_name,
            compare_enum_name: &compare_enum_name,
            filter_name: &filter_name,
            record_fields: &mut record_fields,
            filter_fields: &mut filter_fields,
            extra_objects: &mut extra_objects,
            extra_inputs: &mut extra_inputs,
            extra_enums: &mut extra_enums,
            extra_unions: &mut extra_unions,
            registered_enum_names: &mut registered_enum_names,
        };
        for node in effective_dims {
            collect_dimension_types(node, "", &mut collector);
        }
    }

    let mut metric_enums: Vec<Enum> = Vec::new();
    let builtin_descs: std::collections::HashMap<&str, &str> = [
        ("count", "Count of rows or distinct values"),
        ("sum", "Sum of values"),
        ("avg", "Average of values"),
        ("min", "Minimum value"),
        ("max", "Maximum value"),
        ("uniq", "Count of unique (distinct) values"),
    ].into_iter().collect();

    for metric in &cube.metrics {
        let metric_name = &metric.name;
        let select_where_name = format!("{}_{}_SelectWhere", prefixed_name, metric_name);

        if metric.supports_where {
            extra_inputs.push(
                InputObject::new(&select_where_name)
                    .description(format!("Post-aggregation filter for {} (HAVING clause)", metric_name))
                    .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)).description("Greater than"))
                    .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)).description("Greater than or equal to"))
                    .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)).description("Less than"))
                    .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)).description("Less than or equal to"))
                    .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)).description("Equal to"))
                    .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)).description("Not equal to")),
            );
        }

        let of_enum_name = format!("{}_{}_Of", prefixed_name, metric_name);
        let mut of_enum = Enum::new(&of_enum_name)
            .description(format!("Dimension to apply {} aggregation on", metric_name));
        for (path, _) in &flat_out { of_enum = of_enum.item(EnumItem::new(path)); }
        metric_enums.push(of_enum);

        let metric_name_clone = metric_name.clone();
        let return_type_ref = dim_type_to_typeref(&metric.return_type);
        let metric_desc = metric.description.as_deref()
            .or_else(|| builtin_descs.get(metric_name.as_str()).copied())
            .unwrap_or("Aggregate metric");

        let mut metric_field = Field::new(metric_name, return_type_ref, move |ctx| {
            let default_name = metric_name_clone.clone();
            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                let alias = ctx.ctx.field().alias().unwrap_or(&default_name);
                let key = metric_key(alias);
                let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                Ok(Some(FieldValue::value(json_to_gql_value(val))))
            })
        })
        .description(metric_desc)
        .argument(InputValue::new("of", TypeRef::named(&of_enum_name))
            .description("Dimension to aggregate on (default: all rows)"));

        if metric_name == "count" {
            metric_field = metric_field
                .argument(InputValue::new("distinct", TypeRef::named(&of_enum_name))
                    .description("Count distinct values of this dimension (alias for of, maps to uniqExact)"));
        }

        if metric.supports_where {
            metric_field = metric_field
                .argument(InputValue::new("selectWhere", TypeRef::named(&select_where_name))
                    .description("Post-aggregation filter (HAVING)"))
                .argument(InputValue::new("if", TypeRef::named(&filter_name))
                    .description("Conditional filter for this metric"));
        }

        record_fields.push(metric_field);
    }

    // `quantile(of: ..., level: ...)` — percentile computation
    {
        let of_enum_name = format!("{}_quantile_Of", prefixed_name);
        let mut of_enum = Enum::new(&of_enum_name)
            .description(format!("Dimension to apply quantile on for {}", cube.name));
        for (path, _) in &flat_out { of_enum = of_enum.item(EnumItem::new(path)); }
        metric_enums.push(of_enum);

        let of_enum_for_closure = of_enum_name.clone();
        let quantile_field = Field::new("quantile", TypeRef::named(TypeRef::FLOAT), |ctx| {
            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                let alias = ctx.ctx.field().alias().unwrap_or("quantile");
                let key = metric_key(alias);
                let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                Ok(Some(FieldValue::value(json_to_gql_value(val))))
            })
        })
        .description("Compute a quantile (percentile) of a dimension")
        .argument(InputValue::new("of", TypeRef::named_nn(&of_enum_for_closure))
            .description("Dimension to compute quantile on"))
        .argument(InputValue::new("level", TypeRef::named_nn(TypeRef::FLOAT))
            .description("Quantile level (0 to 1, e.g. 0.95 for 95th percentile)"));
        record_fields.push(quantile_field);
    }

    // `calculate(expression: "...")` — runtime expression computation
    {
        let calculate_field = Field::new("calculate", TypeRef::named(TypeRef::FLOAT), |ctx| {
            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                let alias = ctx.ctx.field().alias().unwrap_or("calculate");
                let key = metric_key(alias);
                let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
                Ok(Some(FieldValue::value(json_to_gql_value(val))))
            })
        })
        .description("Compute an expression from other metric values. Use $field_name to reference metrics.")
        .argument(InputValue::new("expression", TypeRef::named_nn(TypeRef::STRING))
            .description("SQL expression with $variable references (e.g. \"$sell_volume - $buy_volume\")"));
        record_fields.push(calculate_field);
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

    let orderby_input_name = format!("{}OrderByInput", prefixed_name);
    let orderby_input = InputObject::new(&orderby_input_name)
        .description(format!("Sort order for {} (Bitquery-compatible)", cube.name))
        .field(InputValue::new("descending", TypeRef::named(&compare_enum_name))
            .description("Sort descending by this field"))
        .field(InputValue::new("ascending", TypeRef::named(&compare_enum_name))
            .description("Sort ascending by this field"))
        .field(InputValue::new("descendingByField", TypeRef::named(TypeRef::STRING))
            .description("Sort descending by computed/aggregated field name"))
        .field(InputValue::new("ascendingByField", TypeRef::named(TypeRef::STRING))
            .description("Sort ascending by computed/aggregated field name"));

    let limitby_input_name = format!("{}LimitByInput", prefixed_name);
    let limitby_input = InputObject::new(&limitby_input_name)
        .description(format!("Per-group row limit for {}", cube.name))
        .field(InputValue::new("by", TypeRef::named_nn(&compare_enum_name))
            .description("Dimension field to group by"))
        .field(InputValue::new("count", TypeRef::named_nn(TypeRef::INT))
            .description("Maximum rows per group"))
        .field(InputValue::new("offset", TypeRef::named(TypeRef::INT))
            .description("Rows to skip per group"));

    let mut objects = vec![record]; objects.extend(extra_objects);
    let mut inputs = vec![filter, orderby_input, limitby_input]; inputs.extend(extra_inputs);
    let mut enums = vec![compare_enum]; enums.extend(metric_enums); enums.extend(extra_enums);

    CubeTypes { objects, inputs, enums, unions: extra_unions }
}

struct DimCollector<'a> {
    cube_name: &'a str,
    compare_enum_name: &'a str,
    filter_name: &'a str,
    record_fields: &'a mut Vec<Field>,
    filter_fields: &'a mut Vec<InputValue>,
    extra_objects: &'a mut Vec<Object>,
    extra_inputs: &'a mut Vec<InputObject>,
    extra_enums: &'a mut Vec<Enum>,
    extra_unions: &'a mut Vec<Union>,
    registered_enum_names: &'a mut HashSet<String>,
}

fn collect_dimension_types(node: &DimensionNode, prefix: &str, c: &mut DimCollector<'_>) {
    match node {
        DimensionNode::Leaf(dim) => {
            let col = dim.column.clone();
            let is_datetime = dim.dim_type == DimType::DateTime;
            let compare_enum = c.compare_enum_name.to_string();
            let cube_filter = c.filter_name.to_string();
            let dim_path = if prefix.is_empty() {
                dim.graphql_name.clone()
            } else {
                format!("{prefix}_{}", dim.graphql_name)
            };
            let mut leaf_field = Field::new(
                &dim.graphql_name, dim_type_to_typeref(&dim.dim_type),
                move |ctx| {
                    let col = col.clone();
                    let dim_path = dim_path.clone();
                    FieldFuture::new(async move {
                        let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                        let has_interval = ctx.args.try_get("interval").is_ok();
                        let has_max = ctx.args.try_get("maximum").is_ok();
                        let has_min = ctx.args.try_get("minimum").is_ok();
                        let key = if has_interval || has_max || has_min {
                            let name = ctx.ctx.field().alias()
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| dim_path.clone());
                            dim_agg_key(&name)
                        } else {
                            col.clone()
                        };
                        let val = row.get(&key).cloned().unwrap_or(serde_json::Value::Null);
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
            leaf_field = leaf_field
                .argument(InputValue::new("maximum", TypeRef::named(&compare_enum))
                    .description("Return value from row where compare field is maximum (argMax)"))
                .argument(InputValue::new("minimum", TypeRef::named(&compare_enum))
                    .description("Return value from row where compare field is minimum (argMin)"))
                .argument(InputValue::new("if", TypeRef::named(&cube_filter))
                    .description("Conditional filter for aggregation"))
                .argument(InputValue::new("selectWhere", TypeRef::named("DimSelectWhere"))
                    .description("Post-aggregation value filter (HAVING)"));
            if is_datetime {
                leaf_field = leaf_field
                    .argument(InputValue::new("interval", TypeRef::named("TimeIntervalInput"))
                        .description("Time bucketing interval (e.g. {in: minutes, count: 1})"));
            }
            // Update resolver key: if interval is present, read from the interval expression key
            // This is handled by the resolver checking ctx.args for "interval"
            c.record_fields.push(leaf_field);
            if let DimType::Enum(enum_name, values) = &dim.dim_type {
                if c.registered_enum_names.insert(enum_name.clone()) {
                    let mut gql_enum = Enum::new(enum_name);
                    for v in values { gql_enum = gql_enum.item(EnumItem::new(v)); }
                    c.extra_enums.push(gql_enum);
                    c.extra_inputs.push(filter_types::build_enum_filter(enum_name));
                }
            }
            c.filter_fields.push(InputValue::new(&dim.graphql_name, TypeRef::named(dim_type_to_filter_name(&dim.dim_type))));
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
                compare_enum_name: c.compare_enum_name,
                filter_name: c.filter_name,
                record_fields: &mut child_record_fields,
                filter_fields: &mut child_filter_fields,
                extra_objects: c.extra_objects,
                extra_inputs: c.extra_inputs,
                extra_enums: c.extra_enums,
                extra_unions: c.extra_unions,
                registered_enum_names: c.registered_enum_names,
            };
            for child in children {
                collect_dimension_types(child, &new_prefix, &mut child_collector);
            }

            let nested_filter_desc = format!("Filter conditions for {}", graphql_name);
            let mut nested_filter = InputObject::new(&nested_filter_name)
                .description(nested_filter_desc);
            for f in child_filter_fields { nested_filter = nested_filter.field(f); }

            {
                let mut nested_record = Object::new(&nested_record_name);
                for f in child_record_fields { nested_record = nested_record.field(f); }
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
                c.extra_objects.push(nested_record);
            }

            c.filter_fields.push(InputValue::new(graphql_name, TypeRef::named(&nested_filter_name)));
            c.extra_inputs.push(nested_filter);
        }
        DimensionNode::Array { graphql_name, description, children } => {
            let full_path = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{prefix}_{graphql_name}")
            };
            let element_type_name = format!("{}_{full_path}_Element", c.cube_name);
            let includes_filter_name = format!("{}_{full_path}_IncludesFilter", c.cube_name);

            let mut element_obj = Object::new(&element_type_name);
            let mut includes_filter = InputObject::new(&includes_filter_name)
                .description(format!("Element-level filter for {} (used with includes)", graphql_name));

            let mut union_registrations: Vec<(String, Union, Vec<Object>)> = Vec::new();

            for child in children {
                match &child.field_type {
                    crate::cube::definition::ArrayFieldType::Scalar(dt) => {
                        let col_name = child.column.clone();
                        let is_datetime = *dt == DimType::DateTime;
                        let mut field = Field::new(
                            &child.graphql_name,
                            dim_type_to_typeref(dt),
                            move |ctx| {
                                let col = col_name.clone();
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
                        if let Some(desc) = &child.description {
                            field = field.description(desc);
                        }
                        element_obj = element_obj.field(field);
                        includes_filter = includes_filter.field(
                            InputValue::new(&child.graphql_name, TypeRef::named(dim_type_to_filter_name(dt)))
                        );
                    }
                    crate::cube::definition::ArrayFieldType::Union(variants) => {
                        let union_name = format!("{}_{full_path}_{}_Union", c.cube_name, child.graphql_name);

                        let mut gql_union = Union::new(&union_name);
                        let mut variant_objects = Vec::new();

                        for v in variants {
                            let variant_obj_name = v.type_name.clone();
                            let field_name = v.field_name.clone();
                            let source_type = v.source_type.clone();
                            let type_ref = dim_type_to_typeref(&source_type);

                            let val_col = child.column.clone();
                            let variant_obj = Object::new(&variant_obj_name)
                                .field(Field::new(
                                    &field_name,
                                    type_ref,
                                    move |ctx| {
                                        let col = val_col.clone();
                                        FieldFuture::new(async move {
                                            let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                                            let val = row.get(&col).cloned().unwrap_or(serde_json::Value::Null);
                                            Ok(Some(FieldValue::value(json_to_gql_value(val))))
                                        })
                                    },
                                ));
                            gql_union = gql_union.possible_type(&variant_obj_name);
                            variant_objects.push(variant_obj);
                        }

                        let col_name = child.column.clone();
                        let type_col = children.iter()
                            .find(|f| f.graphql_name == "Type")
                            .map(|f| f.column.clone())
                            .unwrap_or_default();
                        let variants_clone: Vec<crate::cube::definition::UnionVariant> = variants.clone();

                        let mut field = Field::new(
                            &child.graphql_name,
                            TypeRef::named(&union_name),
                            move |ctx| {
                                let col = col_name.clone();
                                let tcol = type_col.clone();
                                let vars = variants_clone.clone();
                                FieldFuture::new(async move {
                                    let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                                    let raw_val = row.get(&col).cloned().unwrap_or(serde_json::Value::Null);
                                    if raw_val.is_null() || raw_val.as_str() == Some("") {
                                        return Ok(None);
                                    }
                                    let type_str = row.get(&tcol)
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let resolved_type = resolve_union_typename(type_str, &vars);
                                    let mut elem = RowMap::new();
                                    elem.insert(col.clone(), raw_val);
                                    Ok(Some(FieldValue::owned_any(elem).with_type(resolved_type)))
                                })
                            },
                        );
                        if let Some(desc) = &child.description {
                            field = field.description(desc);
                        }
                        element_obj = element_obj.field(field);

                        let union_filter_name = format!("{}_{full_path}_{}_UnionFilter", c.cube_name, child.graphql_name);
                        let mut union_filter_obj = InputObject::new(&union_filter_name)
                            .description(format!("Filter on {} union values by variant type", child.graphql_name));
                        for v in variants {
                            let key = capitalize_first(&v.field_name);
                            let ft = dim_type_to_filter_name(&v.source_type);
                            union_filter_obj = union_filter_obj.field(
                                InputValue::new(&key, TypeRef::named(ft))
                            );
                        }
                        c.extra_inputs.push(union_filter_obj);
                        includes_filter = includes_filter.field(
                            InputValue::new(&child.graphql_name, TypeRef::named(&union_filter_name))
                        );

                        union_registrations.push((union_name, gql_union, variant_objects));
                    }
                    crate::cube::definition::ArrayFieldType::Group(group_children) => {
                        let group_obj_name = format!("{}_{full_path}_{}_Object", c.cube_name, child.graphql_name);
                        let mut group_obj = Object::new(&group_obj_name);

                        let child_cols: Vec<(String, String)> = group_children.iter()
                            .map(|gc| (gc.graphql_name.clone(), gc.column.clone()))
                            .collect();

                        for gc in group_children {
                            if let crate::cube::definition::ArrayFieldType::Scalar(dt) = &gc.field_type {
                                let gc_col = gc.column.clone();
                                let gc_dt_is_datetime = *dt == DimType::DateTime;
                                let mut gc_field = Field::new(
                                    &gc.graphql_name,
                                    dim_type_to_typeref(dt),
                                    move |ctx| {
                                        let col = gc_col.clone();
                                        FieldFuture::new(async move {
                                            let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                                            let val = row.get(&col).cloned().unwrap_or(serde_json::Value::Null);
                                            let gql_val = if gc_dt_is_datetime {
                                                json_to_gql_datetime(val)
                                            } else {
                                                json_to_gql_value(val)
                                            };
                                            Ok(Some(FieldValue::value(gql_val)))
                                        })
                                    },
                                );
                                if let Some(desc) = &gc.description {
                                    gc_field = gc_field.description(desc);
                                }
                                group_obj = group_obj.field(gc_field);
                            }
                        }
                        c.extra_objects.push(group_obj);

                        let child_cols_for_resolver = child_cols.clone();
                        let group_obj_name_clone = group_obj_name.clone();
                        let _ = &group_obj_name_clone;
                        let field = Field::new(
                            &child.graphql_name,
                            TypeRef::named(&group_obj_name),
                            move |ctx| {
                                let cols = child_cols_for_resolver.clone();
                                FieldFuture::new(async move {
                                    let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                                    let mut sub = RowMap::new();
                                    for (_, col) in &cols {
                                        if let Some(v) = row.get(col) {
                                            sub.insert(col.clone(), v.clone());
                                        }
                                    }
                                    Ok(Some(FieldValue::owned_any(sub)))
                                })
                            },
                        );
                        element_obj = element_obj.field(field);
                    }
                }
            }

            // Register union types and variant objects
            for (_, union_type, variant_objs) in union_registrations {
                c.extra_objects.extend(variant_objs);
                // Unions need to be registered differently — store in extra_objects
                // by wrapping in a dummy. Actually, we need to register unions via builder.
                // For now, we'll store them in a new field. Let's use the enums vec hack:
                // Actually, async-graphql dynamic schema registers unions the same way.
                // We need to pass them up. Let's add a unions vec to DimCollector.
                // For now, store union as Object (it won't work). We need to refactor.
                // Let me add unions to CubeTypes.
                c.extra_unions.push(union_type);
            }

            // Build the list resolver: zip parallel array columns into element objects.
            // For Group fields, flatten group children columns into the list.
            let child_columns: Vec<(String, String)> = children.iter()
                .flat_map(|f| {
                    if let crate::cube::definition::ArrayFieldType::Group(group_children) = &f.field_type {
                        group_children.iter()
                            .map(|gc| (gc.column.clone(), gc.column.clone()))
                            .collect::<Vec<_>>()
                    } else {
                        vec![(f.graphql_name.clone(), f.column.clone())]
                    }
                })
                .collect();
            let element_type_name_clone = element_type_name.clone();

            let mut array_field = Field::new(
                graphql_name,
                TypeRef::named_nn_list_nn(&element_type_name),
                move |ctx| {
                    let cols = child_columns.clone();
                    let _etype = element_type_name_clone.clone();
                    FieldFuture::new(async move {
                        let row = ctx.parent_value.try_downcast_ref::<RowMap>()?;
                        let arrays: Vec<(&str, Vec<serde_json::Value>)> = cols.iter()
                            .map(|(gql_name, col)| {
                                let arr = row.get(col)
                                    .and_then(|v| v.as_array())
                                    .cloned()
                                    .unwrap_or_default();
                                (gql_name.as_str(), arr)
                            })
                            .collect();

                        let len = arrays.first().map(|(_, a)| a.len()).unwrap_or(0);
                        let mut elements = Vec::with_capacity(len);
                        for i in 0..len {
                            let mut elem = RowMap::new();
                            for (gql_name, arr) in &arrays {
                                let val = arr.get(i).cloned().unwrap_or(serde_json::Value::Null);
                                elem.insert(gql_name.to_string(), val);
                            }
                            // Also store by column name for Union resolvers
                            for ((_gql_name, col), (_, arr)) in cols.iter().zip(arrays.iter()) {
                                let val = arr.get(i).cloned().unwrap_or(serde_json::Value::Null);
                                elem.insert(col.clone(), val);
                            }
                            elements.push(FieldValue::owned_any(elem));
                        }
                        Ok(Some(FieldValue::list(elements)))
                    })
                },
            );
            if let Some(desc) = description {
                array_field = array_field.description(desc);
            }
            c.record_fields.push(array_field);

            // Wrap the element-level IncludesFilter in an ArrayFilter so the
            // parser can find the `includes` key (compiler/filter.rs expects
            // `{ includes: [{ Name: { is: "..." } }] }`).
            let wrapper_filter_name = format!("{}_{full_path}_ArrayFilter", c.cube_name);
            let wrapper_filter = InputObject::new(&wrapper_filter_name)
                .description(format!("Array filter for {} — use `includes` to match elements", graphql_name))
                .field(InputValue::new("includes", TypeRef::named_list(&includes_filter_name))
                    .description("Match rows where at least one array element satisfies all conditions"));
            c.extra_inputs.push(wrapper_filter);

            c.filter_fields.push(InputValue::new(
                graphql_name,
                TypeRef::named(&wrapper_filter_name),
            ));

            c.extra_objects.push(element_obj);
            c.extra_inputs.push(includes_filter);
        }
    }
}

/// Resolve the GraphQL Union __typename from a discriminator column value.
///
/// Walks variants in order; the first whose `source_type_names` contains
/// `type_str` wins.  If none match, the last variant is used as fallback
/// (conventionally the "catch-all" variant such as JSON).
fn resolve_union_typename(type_str: &str, variants: &[crate::cube::definition::UnionVariant]) -> String {
    for v in variants {
        if v.source_type_names.iter().any(|s| s == type_str) {
            return v.type_name.clone();
        }
    }
    variants.last().map(|v| v.type_name.clone()).unwrap_or_default()
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn dim_type_to_typeref(dt: &DimType) -> TypeRef {
    match dt {
        DimType::String | DimType::Decimal | DimType::BigInteger | DimType::Date => TypeRef::named(TypeRef::STRING),
        DimType::DateTime => TypeRef::named("DateTime"),
        DimType::Int => TypeRef::named(TypeRef::INT),
        DimType::Float => TypeRef::named(TypeRef::FLOAT),
        DimType::Bool => TypeRef::named(TypeRef::BOOLEAN),
        DimType::Enum(name, _) => TypeRef::named(name.as_str()),
        DimType::StringArray => TypeRef::named_list(TypeRef::STRING),
        DimType::IntArray => TypeRef::named_list(TypeRef::INT),
    }
}

fn dim_type_to_filter_name(dt: &DimType) -> String {
    match dt {
        DimType::String => "StringFilter".to_string(),
        DimType::Int => "IntFilter".to_string(),
        DimType::Float => "FloatFilter".to_string(),
        DimType::Decimal => "DecimalFilter".to_string(),
        DimType::BigInteger => "BigIntFilter".to_string(),
        DimType::Date => "DateFilter".to_string(),
        DimType::DateTime => "DateTimeFilter".to_string(),
        DimType::Bool => "Boolean".to_string(),
        DimType::Enum(name, _) => format!("{name}Filter"),
        DimType::StringArray | DimType::IntArray => "StringFilter".to_string(),
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
        serde_json::Value::Array(arr) => Value::List(arr.into_iter().map(json_to_gql_value).collect()),
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

    let is_aggregate = target_flat.iter().any(|(_, dim)| is_aggregate_expr(&dim.column));

    let group_by = if is_aggregate {
        let mut gb: Vec<String> = jd.conditions.iter().map(|(_, r)| r.clone()).collect();
        for sel in &selects {
            if let SelectExpr::Column { column, .. } = sel {
                if !is_aggregate_expr(column) && !gb.contains(column) {
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
        join_type: jd.join_type.clone(),
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
