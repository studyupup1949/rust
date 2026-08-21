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

/// Declares a LEFT JOIN relationship from this cube to another cube.
#[derive(Debug, Clone)]
pub struct JoinDef {
    /// GraphQL field name on the source record, e.g. "joinTransfers"
    pub field_name: String,
    /// Target cube name as registered in the CubeRegistry, e.g. "Transfers"
    pub target_cube: String,
    /// (local_column, remote_column) pairs for the ON clause.
    pub conditions: Vec<(String, String)>,
    pub description: Option<String>,
}

pub fn join_def(field_name: &str, target_cube: &str, conditions: &[(&str, &str)]) -> JoinDef {
    JoinDef {
        field_name: field_name.to_string(),
        target_cube: target_cube.to_string(),
        conditions: conditions.iter().map(|(l, r)| (l.to_string(), r.to_string())).collect(),
        description: None,
    }
}

pub fn join_def_desc(field_name: &str, target_cube: &str, conditions: &[(&str, &str)], desc: &str) -> JoinDef {
    JoinDef {
        field_name: field_name.to_string(),
        target_cube: target_cube.to_string(),
        conditions: conditions.iter().map(|(l, r)| (l.to_string(), r.to_string())).collect(),
        description: Some(desc.to_string()),
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
    pub metrics: Vec<String>,
    pub selectors: Vec<SelectorDef>,
    pub default_filters: Vec<(String, String)>,
    pub default_limit: u32,
    pub max_limit: u32,
    /// Append FINAL to FROM clause for ReplacingMergeTree tables in ClickHouse.
    pub use_final: bool,
    /// Human-readable description of the cube's purpose, exposed via _cubeMetadata.
    pub description: String,
    /// Declarative LEFT JOIN relationships to other cubes.
    pub joins: Vec<JoinDef>,
}

impl CubeDefinition {
    pub fn table_for_chain(&self, chain: &str) -> String {
        self.table_pattern.replace("{chain}", chain)
    }

    pub fn qualified_table(&self, chain: &str) -> String {
        format!("{}.{}", self.schema, self.table_for_chain(chain))
    }

    pub fn flat_dimensions(&self) -> Vec<(String, Dimension)> {
        let mut out = Vec::new();
        for node in &self.dimensions {
            collect_leaves(node, "", &mut out);
        }
        out
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

    pub fn metric(mut self, name: &str) -> Self {
        self.def.metrics.push(name.to_string());
        self
    }

    pub fn metrics(mut self, names: &[&str]) -> Self {
        self.def.metrics.extend(names.iter().map(|s| s.to_string()));
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
