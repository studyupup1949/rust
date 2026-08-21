use crate::compiler::ir::{JoinType, QueryBuilderFn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimType {
    String,
    Int,
    Float,
    DateTime,
    Bool,
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
                    || requested_columns.iter().all(|c| r.available_columns.contains(c))
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

    /// Check if a metric name exists in this cube's metrics.
    pub fn has_metric(&self, name: &str) -> bool {
        self.metrics.iter().any(|m| m.name == name)
    }

    /// Find a metric definition by name.
    pub fn find_metric(&self, name: &str) -> Option<&MetricDef> {
        self.metrics.iter().find(|m| m.name == name)
    }
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
    }
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
