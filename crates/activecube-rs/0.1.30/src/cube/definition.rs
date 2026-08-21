use std::collections::HashMap;
use crate::compiler::ir::{JoinType, QueryBuilderFn};

/// Per-column filter value transformation applied before SQL compilation.
/// Generic string manipulation for upstream-compat prefixes or unit conversions.
#[derive(Debug, Clone)]
pub enum FilterValueTransform {
    /// Strip a fixed prefix from matching column's filter value.
    /// Non-matching values (no prefix) are left unchanged.
    StripPrefix { column: String, prefix: String },
    /// Multiply integer filter values by a constant factor.
    /// Used for unit conversions (e.g. user-facing minutes → DB-stored seconds).
    /// Applies to `Int` and numeric `String` SqlValue variants; non-numeric
    /// values are left unchanged.
    MultiplyBy { column: String, factor: i64 },
}

/// Which top-level chain wrapper(s) a cube appears under.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChainGroup {
    /// EVM chains (eth, bsc, ...) — wrapper carries a `network` argument.
    Evm,
    /// Solana — implicit `sol` chain, no network argument needed.
    Solana,
    /// Cross-chain aggregated cubes (OHLC, TokenStats, ...).
    Trading,
}

/// Per-chain dimension override. Replaces a dimension node at a given path
/// when generating schema types for a specific ChainGroup.
#[derive(Debug, Clone)]
pub struct DimensionOverride {
    /// Dot-separated path to the dimension to replace, e.g. "Transfer.Sender".
    pub path: String,
    /// Replacement dimension node.
    pub node: DimensionNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimType {
    String,
    Int,
    Float,
    /// High-precision decimal — GraphQL filter/output uses String to preserve precision
    Decimal,
    /// Arbitrary-precision integer — GraphQL output/filter uses String to preserve precision.
    /// Maps to Bitquery's `OLAP_BigInteger` reference type.
    BigInteger,
    /// Date string (YYYY-MM-DD) with range operators (since/till/after/before)
    Date,
    DateTime,
    Bool,
    /// Enumeration type with a named GraphQL enum and allowed values.
    /// Filter accepts bare enum values (e.g. `{is: buy}`) instead of strings.
    /// Stored as String in ClickHouse — values are lowercased before comparison.
    Enum(std::string::String, Vec<std::string::String>),
    /// Flat array of strings — ClickHouse `Array(String)`, GraphQL `[String]`.
    /// Column is selected as-is; JSON array is returned directly.
    StringArray,
    /// Flat array of integers — ClickHouse `Array(UInt32)` etc., GraphQL `[Int]`.
    IntArray,
}

#[derive(Debug, Clone)]
pub struct Dimension {
    pub graphql_name: String,
    pub column: String,
    pub dim_type: DimType,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DimensionNode {
    Leaf(Dimension),
    Group {
        graphql_name: String,
        description: Option<String>,
        children: Vec<DimensionNode>,
    },
    /// Array dimension: maps to parallel ClickHouse Array columns.
    /// Each element is a structured object with fields from aligned arrays.
    Array {
        graphql_name: String,
        description: Option<String>,
        children: Vec<ArrayFieldDef>,
    },
}

/// One field inside an Array dimension, backed by a ClickHouse Array column.
#[derive(Debug, Clone)]
pub struct ArrayFieldDef {
    pub graphql_name: String,
    /// Result lookup key (also used as SQL alias when `expression` is set).
    pub column: String,
    pub field_type: ArrayFieldType,
    pub description: Option<String>,
    /// Optional SQL expression. When set, the SELECT becomes `expression AS column`
    /// (e.g. `arrayFilter(...) AS topics_hash`). The resolver uses `column` for lookup.
    pub expression: Option<String>,
}

/// Type of an array field: scalar, polymorphic Union, or nested object group.
#[derive(Debug, Clone)]
pub enum ArrayFieldType {
    Scalar(DimType),
    Union(Vec<UnionVariant>),
    /// Nested object group within an array element (e.g. Accounts.Token { Mint, Owner }).
    /// Each child is backed by its own parallel ClickHouse array.
    Group(Vec<ArrayFieldDef>),
}

/// One variant of a GraphQL Union type.
#[derive(Debug, Clone)]
pub struct UnionVariant {
    /// GraphQL type name, e.g. "Solana_ABI_Integer_Value_Arg"
    pub type_name: String,
    /// GraphQL field name inside the variant, e.g. "integer"
    pub field_name: String,
    /// Scalar type of the value, e.g. DimType::Int
    pub source_type: DimType,
    /// Source type strings that resolve to this variant (e.g. ["u8", "u16", "u32"]).
    /// Empty means this is the fallback variant (matched when no other variant matches).
    pub source_type_names: Vec<String>,
}

/// A named selector defines a filterable field on a Cube.
/// Each selector maps a GraphQL argument name to a column + type,
/// enabling `eq`, `gt`, `in`, `any` etc.
#[derive(Debug, Clone)]
pub struct SelectorDef {
    pub graphql_name: String,
    pub column: String,
    pub dim_type: DimType,
}

pub fn selector(graphql_name: &str, column: &str, dim_type: DimType) -> SelectorDef {
    SelectorDef {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        dim_type,
    }
}

/// Metric definition — standard SQL aggregate or custom expression.
#[derive(Debug, Clone)]
pub struct MetricDef {
    pub name: String,
    /// If None, uses the standard SQL function (COUNT/SUM/AVG/...).
    /// If Some, uses this SQL template with `{column}` as placeholder.
    /// Example: `"sumIf({column}, direction='in') - sumIf({column}, direction='out')"`
    pub expression_template: Option<String>,
    pub return_type: DimType,
    pub description: Option<String>,
    /// Whether this metric supports conditional aggregation (countIf/sumIf).
    pub supports_where: bool,
}

impl MetricDef {
    pub fn standard(name: &str) -> Self {
        Self {
            name: name.to_string(),
            expression_template: None,
            return_type: DimType::Float,
            description: None,
            supports_where: true,
        }
    }

    pub fn custom(name: &str, expression: &str) -> Self {
        Self {
            name: name.to_string(),
            expression_template: Some(expression.to_string()),
            return_type: DimType::Float,
            description: None,
            supports_where: false,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_return_type(mut self, rt: DimType) -> Self {
        self.return_type = rt;
        self
    }
}

/// Helper to create a list of standard metrics from names.
pub fn standard_metrics(names: &[&str]) -> Vec<MetricDef> {
    names.iter().map(|n| MetricDef::standard(n)).collect()
}

/// Multi-table routing: a single Cube can map to different physical tables
/// depending on which columns the query requests.
#[derive(Debug, Clone)]
pub struct TableRoute {
    pub schema: String,
    pub table_pattern: String,
    /// Columns available in this table. If empty, this route serves all queries.
    pub available_columns: Vec<String>,
    /// Lower priority = preferred. The primary table (schema/table_pattern) has implicit priority 0.
    pub priority: u32,
}

/// Declares a JOIN relationship from this cube to another cube.
#[derive(Debug, Clone)]
pub struct JoinDef {
    /// GraphQL field name on the source record, e.g. "joinTransfers"
    pub field_name: String,
    /// Target cube name as registered in the CubeRegistry, e.g. "Transfers"
    pub target_cube: String,
    /// (local_column, remote_column) pairs for the ON clause.
    pub conditions: Vec<(String, String)>,
    pub description: Option<String>,
    /// JOIN type — defaults to Left.
    pub join_type: JoinType,
}

pub fn join_def(field_name: &str, target_cube: &str, conditions: &[(&str, &str)]) -> JoinDef {
    JoinDef {
        field_name: field_name.to_string(),
        target_cube: target_cube.to_string(),
        conditions: conditions.iter().map(|(l, r)| (l.to_string(), r.to_string())).collect(),
        description: None,
        join_type: JoinType::Left,
    }
}

pub fn join_def_desc(field_name: &str, target_cube: &str, conditions: &[(&str, &str)], desc: &str) -> JoinDef {
    JoinDef {
        field_name: field_name.to_string(),
        target_cube: target_cube.to_string(),
        conditions: conditions.iter().map(|(l, r)| (l.to_string(), r.to_string())).collect(),
        description: Some(desc.to_string()),
        join_type: JoinType::Left,
    }
}

pub fn join_def_typed(
    field_name: &str, target_cube: &str,
    conditions: &[(&str, &str)],
    join_type: JoinType,
) -> JoinDef {
    JoinDef {
        field_name: field_name.to_string(),
        target_cube: target_cube.to_string(),
        conditions: conditions.iter().map(|(l, r)| (l.to_string(), r.to_string())).collect(),
        description: None,
        join_type,
    }
}

#[derive(Debug, Clone)]
pub struct CubeDefinition {
    pub name: String,
    pub schema: String,
    /// Table name pattern. Use `{chain}` as placeholder for chain-prefixed tables
    /// (e.g. `{chain}_trades` → `sol_trades`). For tables without chain prefix
    /// (e.g. `dex_pool_liquidities`), use the literal table name and set
    /// `chain_column` instead.
    pub table_pattern: String,
    /// If set, the table doesn't use a `{chain}` prefix in its name. Instead,
    /// the chain is filtered via `WHERE <chain_column> = ?`. Example:
    /// `dex_pool_liquidities` has a `chain` column rather than `sol_dex_pool_liquidities`.
    pub chain_column: Option<String>,
    pub dimensions: Vec<DimensionNode>,
    pub metrics: Vec<MetricDef>,
    pub selectors: Vec<SelectorDef>,
    pub default_filters: Vec<(String, String)>,
    pub default_limit: u32,
    pub max_limit: u32,
    /// Append FINAL to FROM clause for ReplacingMergeTree tables in ClickHouse.
    pub use_final: bool,
    /// Human-readable description of the cube's purpose, exposed via _cubeMetadata.
    pub description: String,
    /// Declarative JOIN relationships to other cubes.
    pub joins: Vec<JoinDef>,
    /// Alternative tables that can serve subsets of this cube's columns.
    /// When non-empty, `resolve_table` picks the best match by requested columns.
    pub table_routes: Vec<TableRoute>,
    /// Custom query builder that bypasses the standard IR → SQL compilation.
    /// Used for cubes requiring window functions, CTEs, or multi-step subqueries.
    pub custom_query_builder: Option<QueryBuilderFn>,
    /// SQL subquery used as the FROM source instead of `schema.table`.
    /// Supports `{schema}` and `{chain}` placeholders expanded at query time.
    /// When set, the compiler generates `FROM ({expanded}) AS _t`.
    pub from_subquery: Option<String>,
    /// Columns to always include in GROUP BY when aggregation is triggered.
    /// Required for AggregatingMergeTree tables where ALIAS columns (dictGet)
    /// reference key columns that may not appear in the user's SELECT.
    pub required_group_by: Vec<String>,
    /// Which chain wrapper(s) this cube appears under. Empty = legacy flat mode.
    pub chain_groups: Vec<ChainGroup>,
    /// Per-chain dimension overrides. When a cube appears under multiple
    /// ChainGroups and certain fields need different types per chain,
    /// these overrides replace matching dimension paths during type generation.
    pub chain_overrides: HashMap<ChainGroup, Vec<DimensionOverride>>,
    /// Columns whose filter values should be lowercased before SQL compilation.
    /// Used for Bitquery compatibility where e.g. `"Solana"` must map to `"solana"`.
    pub lowercase_filter_columns: Vec<String>,
    /// Per-column filter value transformations applied before SQL compilation.
    /// Generic string manipulation (e.g. strip upstream-compat prefix) that is
    /// not tied to any particular upstream system.
    pub filter_value_transforms: Vec<FilterValueTransform>,
    /// Dimension names that should appear in `aggregateFunctions` instead of `fields`
    /// in the builder schema JSON. Matches Bitquery's convention where measure-like
    /// leaf dimensions (e.g. Price, Side) are classified as aggregate functions.
    pub aggregate_only_fields: Vec<String>,
}

impl CubeDefinition {
    pub fn table_for_chain(&self, chain: &str) -> String {
        self.table_pattern.replace("{chain}", chain)
    }

    pub fn qualified_table(&self, chain: &str) -> String {
        format!("{}.{}", self.schema, self.table_for_chain(chain))
    }

    /// Pick the optimal (schema, table) for a given chain and set of requested columns.
    /// Falls back to the primary schema/table_pattern when no route matches.
    pub fn resolve_table(&self, chain: &str, requested_columns: &[String]) -> (String, String) {
        if self.table_routes.is_empty() {
            return (self.schema.clone(), self.table_for_chain(chain));
        }

        let mut candidates: Vec<&TableRoute> = self.table_routes.iter()
            .filter(|r| {
                r.available_columns.is_empty()
                    || (!requested_columns.is_empty()
                        && requested_columns.iter().all(|c| r.available_columns.contains(c)))
            })
            .collect();

        candidates.sort_by_key(|r| r.priority);

        if let Some(best) = candidates.first() {
            (best.schema.clone(), best.table_pattern.replace("{chain}", chain))
        } else {
            (self.schema.clone(), self.table_for_chain(chain))
        }
    }

    pub fn flat_dimensions(&self) -> Vec<(String, Dimension)> {
        let mut out = Vec::new();
        for node in &self.dimensions {
            collect_leaves(node, "", &mut out);
        }
        out
    }

    /// Returns dimensions with chain-specific overrides applied.
    pub fn dimensions_for_chain(&self, group: &ChainGroup) -> Vec<DimensionNode> {
        match self.chain_overrides.get(group) {
            Some(overrides) if !overrides.is_empty() => apply_overrides(&self.dimensions, overrides),
            _ => self.dimensions.clone(),
        }
    }

    /// Returns true if this cube has per-chain dimension overrides.
    pub fn has_chain_overrides(&self) -> bool {
        !self.chain_overrides.is_empty()
    }

    /// Check if a metric name exists in this cube's metrics.
    pub fn has_metric(&self, name: &str) -> bool {
        self.metrics.iter().any(|m| m.name == name)
    }

    /// Find a metric definition by name.
    pub fn find_metric(&self, name: &str) -> Option<&MetricDef> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Collect all columns used by Array dimensions (parallel arrays).
    /// Returns `(graphql_path, column)` pairs for every array child field.
    /// Returns (path, result_key, optional_expression) triples for array columns.
    pub fn array_columns(&self) -> Vec<(String, String, Option<String>)> {
        let mut out = Vec::new();
        for node in &self.dimensions {
            collect_array_columns(node, "", &mut out);
        }
        out
    }
}

/// Returns the effective dimensions for a specific chain group,
/// applying any chain_overrides on top of the base dimensions.
pub fn apply_overrides(dims: &[DimensionNode], overrides: &[DimensionOverride]) -> Vec<DimensionNode> {
    if overrides.is_empty() {
        return dims.to_vec();
    }
    dims.iter().map(|node| {
        // Check if a top-level override matches this node's root path
        let node_name = match node {
            DimensionNode::Leaf(d) => &d.graphql_name,
            DimensionNode::Group { graphql_name, .. } => graphql_name,
            DimensionNode::Array { graphql_name, .. } => graphql_name,
        };
        if let Some(ov) = overrides.iter().find(|o| o.path == *node_name) {
            ov.node.clone()
        } else {
            apply_override_node(node, "", overrides)
        }
    }).collect()
}

fn apply_override_node(node: &DimensionNode, prefix: &str, overrides: &[DimensionOverride]) -> DimensionNode {
    match node {
        DimensionNode::Group { graphql_name, description, children } => {
            let path = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{}.{}", prefix, graphql_name)
            };
            // Check if any override matches children of this group
            let new_children: Vec<DimensionNode> = children.iter().map(|child| {
                let child_name = match child {
                    DimensionNode::Leaf(d) => &d.graphql_name,
                    DimensionNode::Group { graphql_name, .. } => graphql_name,
                    DimensionNode::Array { graphql_name, .. } => graphql_name,
                };
                let child_path = format!("{}.{}", path, child_name);
                if let Some(ov) = overrides.iter().find(|o| o.path == child_path) {
                    ov.node.clone()
                } else {
                    apply_override_node(child, &path, overrides)
                }
            }).collect();
            DimensionNode::Group {
                graphql_name: graphql_name.clone(),
                description: description.clone(),
                children: new_children,
            }
        }
        other => other.clone(),
    }
}

/// (path, result_key, optional_expression)
/// When `expression` is `Some`, the SQL SELECT should be `expression AS result_key`.
fn collect_array_columns(node: &DimensionNode, prefix: &str, out: &mut Vec<(String, String, Option<String>)>) {
    match node {
        DimensionNode::Leaf(_) => {}
        DimensionNode::Group { graphql_name, children, .. } => {
            let new_prefix = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{prefix}_{graphql_name}")
            };
            for child in children {
                collect_array_columns(child, &new_prefix, out);
            }
        }
        DimensionNode::Array { graphql_name, children, .. } => {
            let arr_prefix = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{prefix}_{graphql_name}")
            };
            for af in children {
                if let ArrayFieldType::Group(group_children) = &af.field_type {
                    for gc in group_children {
                        out.push((
                            format!("{}_{}_{}", arr_prefix, af.graphql_name, gc.graphql_name),
                            gc.column.clone(),
                            gc.expression.clone(),
                        ));
                    }
                    continue;
                }
                out.push((
                    format!("{}_{}", arr_prefix, af.graphql_name),
                    af.column.clone(),
                    af.expression.clone(),
                ));
            }
        }
    }
}

pub fn collect_leaves_pub(node: &DimensionNode, prefix: &str, out: &mut Vec<(String, Dimension)>) {
    collect_leaves(node, prefix, out);
}

fn collect_leaves(node: &DimensionNode, prefix: &str, out: &mut Vec<(String, Dimension)>) {
    match node {
        DimensionNode::Leaf(dim) => {
            let path = if prefix.is_empty() {
                dim.graphql_name.clone()
            } else {
                format!("{}_{}", prefix, dim.graphql_name)
            };
            out.push((path, dim.clone()));
        }
        DimensionNode::Group { graphql_name, children, .. } => {
            let new_prefix = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{prefix}_{graphql_name}")
            };
            for child in children {
                collect_leaves(child, &new_prefix, out);
            }
        }
        DimensionNode::Array { graphql_name, description, children } => {
            // Emit the array node itself as a sortable entry using a
            // representative ClickHouse Array column. ClickHouse compares
            // Array(T) values lexicographically, so `ORDER BY <col>` on a
            // representative column yields a stable array-level ordering
            // that matches Bitquery's behavior for `orderBy: { descending: Instruction_Accounts }`.
            let path = if prefix.is_empty() {
                graphql_name.clone()
            } else {
                format!("{prefix}_{graphql_name}")
            };
            if let Some(col) = array_representative_column(children) {
                out.push((
                    path,
                    Dimension {
                        graphql_name: graphql_name.clone(),
                        column: col,
                        dim_type: DimType::StringArray,
                        description: description.clone(),
                    },
                ));
            }
        }
    }
}

/// Pick a representative ClickHouse column for ordering by an entire Array dimension.
/// Prefers the first Scalar child (most semantically meaningful for sort);
/// falls back to drilling into the first Group child's first Scalar column.
fn array_representative_column(children: &[ArrayFieldDef]) -> Option<String> {
    for child in children {
        if let ArrayFieldType::Scalar(_) = &child.field_type {
            if !child.column.is_empty() {
                return Some(child.column.clone());
            }
        }
    }
    for child in children {
        if let ArrayFieldType::Group(group_children) = &child.field_type {
            for gc in group_children {
                if let ArrayFieldType::Scalar(_) = &gc.field_type {
                    if !gc.column.is_empty() {
                        return Some(gc.column.clone());
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// CubeBuilder — ergonomic builder pattern for CubeDefinition
// ---------------------------------------------------------------------------

pub struct CubeBuilder {
    def: CubeDefinition,
}

impl CubeBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            def: CubeDefinition {
                name: name.to_string(),
                schema: String::new(),
                table_pattern: String::new(),
                chain_column: None,
                dimensions: Vec::new(),
                metrics: Vec::new(),
                selectors: Vec::new(),
                default_filters: Vec::new(),
                default_limit: 25,
                max_limit: 10000,
                use_final: false,
                description: String::new(),
                joins: Vec::new(),
                table_routes: Vec::new(),
                custom_query_builder: None,
                from_subquery: None,
                required_group_by: Vec::new(),
                chain_groups: Vec::new(),
                chain_overrides: HashMap::new(),
                lowercase_filter_columns: Vec::new(),
                filter_value_transforms: Vec::new(),
                aggregate_only_fields: Vec::new(),
            },
        }
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.def.schema = schema.to_string();
        self
    }

    pub fn table(mut self, pattern: &str) -> Self {
        self.def.table_pattern = pattern.to_string();
        self
    }

    pub fn chain_column(mut self, column: &str) -> Self {
        self.def.chain_column = Some(column.to_string());
        self
    }

    pub fn dimension(mut self, node: DimensionNode) -> Self {
        self.def.dimensions.push(node);
        self
    }

    /// Add a standard metric (count, sum, avg, min, max, uniq).
    pub fn metric(mut self, name: &str) -> Self {
        self.def.metrics.push(MetricDef::standard(name));
        self
    }

    /// Add multiple standard metrics by name.
    pub fn metrics(mut self, names: &[&str]) -> Self {
        self.def.metrics.extend(names.iter().map(|s| MetricDef::standard(s)));
        self
    }

    /// Add a custom metric with an SQL expression template.
    pub fn custom_metric(mut self, def: MetricDef) -> Self {
        self.def.metrics.push(def);
        self
    }

    pub fn selector(mut self, sel: SelectorDef) -> Self {
        self.def.selectors.push(sel);
        self
    }

    pub fn default_filter(mut self, column: &str, value: &str) -> Self {
        self.def.default_filters.push((column.to_string(), value.to_string()));
        self
    }

    pub fn default_limit(mut self, limit: u32) -> Self {
        self.def.default_limit = limit;
        self
    }

    pub fn max_limit(mut self, limit: u32) -> Self {
        self.def.max_limit = limit;
        self
    }

    pub fn use_final(mut self, val: bool) -> Self {
        self.def.use_final = val;
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.def.description = desc.to_string();
        self
    }

    pub fn join(mut self, j: JoinDef) -> Self {
        self.def.joins.push(j);
        self
    }

    pub fn joins(mut self, js: Vec<JoinDef>) -> Self {
        self.def.joins.extend(js);
        self
    }

    pub fn table_route(mut self, route: TableRoute) -> Self {
        self.def.table_routes.push(route);
        self
    }

    pub fn custom_query_builder(mut self, builder: QueryBuilderFn) -> Self {
        self.def.custom_query_builder = Some(builder);
        self
    }

    pub fn from_subquery(mut self, subquery_sql: &str) -> Self {
        self.def.from_subquery = Some(subquery_sql.to_string());
        self
    }

    pub fn chain_groups(mut self, groups: Vec<ChainGroup>) -> Self {
        self.def.chain_groups = groups;
        self
    }

    pub fn build(self) -> CubeDefinition {
        self.def
    }
}

// ---------------------------------------------------------------------------
// Helper functions for concise dimension/selector construction
// ---------------------------------------------------------------------------

pub fn dim(graphql_name: &str, column: &str, dim_type: DimType) -> DimensionNode {
    DimensionNode::Leaf(Dimension {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        dim_type,
        description: None,
    })
}

pub fn dim_desc(graphql_name: &str, column: &str, dim_type: DimType, desc: &str) -> DimensionNode {
    DimensionNode::Leaf(Dimension {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        dim_type,
        description: Some(desc.to_string()),
    })
}

pub fn dim_group(graphql_name: &str, children: Vec<DimensionNode>) -> DimensionNode {
    DimensionNode::Group {
        graphql_name: graphql_name.to_string(),
        description: None,
        children,
    }
}

pub fn dim_group_desc(graphql_name: &str, desc: &str, children: Vec<DimensionNode>) -> DimensionNode {
    DimensionNode::Group {
        graphql_name: graphql_name.to_string(),
        description: Some(desc.to_string()),
        children,
    }
}

pub fn dim_array(graphql_name: &str, children: Vec<ArrayFieldDef>) -> DimensionNode {
    DimensionNode::Array {
        graphql_name: graphql_name.to_string(),
        description: None,
        children,
    }
}

pub fn dim_array_desc(graphql_name: &str, desc: &str, children: Vec<ArrayFieldDef>) -> DimensionNode {
    DimensionNode::Array {
        graphql_name: graphql_name.to_string(),
        description: Some(desc.to_string()),
        children,
    }
}

pub fn array_field(graphql_name: &str, column: &str, field_type: ArrayFieldType) -> ArrayFieldDef {
    ArrayFieldDef {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        field_type,
        description: None,
        expression: None,
    }
}

pub fn array_field_desc(graphql_name: &str, column: &str, field_type: ArrayFieldType, desc: &str) -> ArrayFieldDef {
    ArrayFieldDef {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        field_type,
        description: Some(desc.to_string()),
        expression: None,
    }
}

/// Array field backed by a SQL expression (e.g. `arrayFilter(...)`).
/// `column` is used as the SQL alias and result lookup key.
pub fn array_field_expr(graphql_name: &str, column: &str, expression: &str, field_type: ArrayFieldType) -> ArrayFieldDef {
    ArrayFieldDef {
        graphql_name: graphql_name.to_string(),
        column: column.to_string(),
        field_type,
        description: None,
        expression: Some(expression.to_string()),
    }
}

/// Create a Union variant without explicit source-type matching (fallback-only).
pub fn variant(type_name: &str, field_name: &str, source_type: DimType) -> UnionVariant {
    UnionVariant {
        type_name: type_name.to_string(),
        field_name: field_name.to_string(),
        source_type,
        source_type_names: vec![],
    }
}

/// Create a Union variant with explicit source-type string matching.
/// When the discriminator column value matches any of `source_names`,
/// this variant is selected.
pub fn variant_matching(
    type_name: &str,
    field_name: &str,
    source_type: DimType,
    source_names: &[&str],
) -> UnionVariant {
    UnionVariant {
        type_name: type_name.to_string(),
        field_name: field_name.to_string(),
        source_type,
        source_type_names: source_names.iter().map(|s| s.to_string()).collect(),
    }
}
