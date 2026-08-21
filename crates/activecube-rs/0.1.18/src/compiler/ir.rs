use std::sync::Arc;

/// SQL binding value — database-agnostic representation.
#[derive(Debug, Clone)]
pub enum SqlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    /// Raw SQL expression (not parameterized). Used for `now() - INTERVAL ...` etc.
    Expression(String),
}

/// JOIN type for cross-cube relationships.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum JoinType {
    #[default]
    Left,
    Inner,
    Full,
    Cross,
}

impl JoinType {
    pub fn sql_keyword(&self) -> &'static str {
        match self {
            JoinType::Left => "LEFT JOIN",
            JoinType::Inner => "INNER JOIN",
            JoinType::Full => "FULL OUTER JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// Custom query builder that bypasses the standard SQL compilation pipeline.
/// Implementors produce SQL directly from a `QueryIR` for cubes that need
/// window functions, CTEs, or multi-step subqueries.
#[derive(Clone)]
pub struct QueryBuilderFn(pub Arc<dyn Fn(&QueryIR) -> CompileResult + Send + Sync>);

impl std::fmt::Debug for QueryBuilderFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("QueryBuilderFn(...)")
    }
}

/// Intermediate representation of a compiled GraphQL cube query.
#[derive(Debug, Clone)]
pub struct QueryIR {
    pub cube: String,
    pub schema: String,
    pub table: String,
    pub selects: Vec<SelectExpr>,
    pub filters: FilterNode,
    pub having: FilterNode,
    pub group_by: Vec<String>,
    pub order_by: Vec<OrderExpr>,
    pub limit: u32,
    pub offset: u32,
    /// ClickHouse `LIMIT n BY col1, col2` — per-group row limit without aggregation.
    pub limit_by: Option<LimitByExpr>,
    /// When true, append FINAL after FROM for ReplacingMergeTree tables.
    pub use_final: bool,
    /// JOIN expressions to other cubes, resolved at query time.
    pub joins: Vec<JoinExpr>,
    /// Custom query builder that overrides standard SQL compilation.
    pub custom_query_builder: Option<QueryBuilderFn>,
    /// Expanded subquery SQL for FROM clause. When present, the compiler
    /// generates `FROM ({subquery}) AS _t` instead of `FROM schema.table`.
    pub from_subquery: Option<String>,
    /// Columns to always include in GROUP BY when aggregation is triggered.
    /// Ensures ALIAS columns (e.g. dictGet) have their dependency columns available.
    pub required_group_by: Vec<String>,
}

/// A resolved JOIN to another table, appended to the outer query.
#[derive(Debug, Clone)]
pub struct JoinExpr {
    pub schema: String,
    pub table: String,
    /// SQL alias for this join, e.g. "_j0", "_j1"
    pub alias: String,
    /// (main_table_col, joined_table_col) ON conditions
    pub conditions: Vec<(String, String)>,
    /// Fields requested from the joined table
    pub selects: Vec<SelectExpr>,
    /// Non-aggregate columns for GROUP BY (mode B only)
    pub group_by: Vec<String>,
    /// Append FINAL for ReplacingMergeTree targets (mode A)
    pub use_final: bool,
    /// true = target is AggregatingMergeTree, use subquery JOIN (mode B)
    pub is_aggregate: bool,
    /// Target cube name for result mapping
    pub target_cube: String,
    /// GraphQL field name for result nesting, e.g. "joinBuyToken"
    pub join_field: String,
    /// JOIN type — defaults to Left for backward compatibility.
    pub join_type: JoinType,
}

/// Bitquery-style dimension aggregation type.
/// `PostBalance(maximum: Block_Slot)` → `argMax(post_balance, block_slot)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DimAggType {
    ArgMax,
    ArgMin,
}

#[derive(Debug, Clone)]
pub enum SelectExpr {
    Column {
        column: String,
        alias: Option<String>,
    },
    Aggregate {
        function: String,
        column: String,
        alias: String,
        condition: Option<String>,
    },
    /// Dimension-level aggregation: `argMax(value_column, compare_column)`.
    /// Used for Bitquery patterns like `PostBalance(maximum: Block_Slot)`.
    DimAggregate {
        agg_type: DimAggType,
        value_column: String,
        compare_column: String,
        alias: String,
        condition: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum FilterNode {
    And(Vec<FilterNode>),
    Or(Vec<FilterNode>),
    Condition {
        column: String,
        op: CompareOp,
        value: SqlValue,
    },
    /// Array-level includes filter: "exists an element in the parallel arrays
    /// satisfying all conditions". Compiles to `arrayExists(lambda, arrays)`.
    ArrayIncludes {
        /// ClickHouse column names of the parallel arrays participating in the lambda.
        array_columns: Vec<String>,
        /// Each inner Vec is one `includes` object (conditions AND-ed within).
        /// Multiple inner Vecs are AND-ed as separate `arrayExists` calls.
        element_conditions: Vec<Vec<FilterNode>>,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Like,
    NotLike,
    In,
    NotIn,
    Includes,
    NotIncludes,
    StartsWith,
    EndsWith,
    Ilike,
    NotIlike,
    IlikeIncludes,
    NotIlikeIncludes,
    IlikeStartsWith,
    IsNull,
    IsNotNull,
}

impl CompareOp {
    pub fn sql_op(&self) -> &'static str {
        match self {
            CompareOp::Eq => "=",
            CompareOp::Ne => "!=",
            CompareOp::Gt => ">",
            CompareOp::Ge => ">=",
            CompareOp::Lt => "<",
            CompareOp::Le => "<=",
            CompareOp::Like => "LIKE",
            CompareOp::NotLike => "NOT LIKE",
            CompareOp::In => "IN",
            CompareOp::NotIn => "NOT IN",
            CompareOp::Includes => "LIKE",
            CompareOp::NotIncludes => "NOT LIKE",
            CompareOp::StartsWith => "LIKE",
            CompareOp::EndsWith => "LIKE",
            CompareOp::Ilike => "ilike",
            CompareOp::NotIlike => "NOT ilike",
            CompareOp::IlikeIncludes => "ilike",
            CompareOp::NotIlikeIncludes => "NOT ilike",
            CompareOp::IlikeStartsWith => "ilike",
            CompareOp::IsNull => "IS NULL",
            CompareOp::IsNotNull => "IS NOT NULL",
        }
    }

    pub fn is_unary(&self) -> bool {
        matches!(self, CompareOp::IsNull | CompareOp::IsNotNull)
    }
}

#[derive(Debug, Clone)]
pub struct OrderExpr {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone)]
pub struct LimitByExpr {
    pub count: u32,
    pub offset: u32,
    pub columns: Vec<String>,
}

impl FilterNode {
    pub fn is_empty(&self) -> bool {
        matches!(self, FilterNode::Empty)
    }
}

/// Returns `true` when a column expression is a SQL aggregate function call.
const AGGREGATE_FUNCTIONS: &[&str] = &[
    "count", "sum", "avg", "min", "max", "any",
    "uniq", "uniqexact", "uniqcombined", "uniqhll12",
    "argmax", "argmin",
    "quantile", "quantiles", "quantileexact", "quantiletiming",
    "median",
    "grouparray", "groupuniqarray", "groupbitand", "groupbitor", "groupbitxor",
    "topk", "entropy", "varpop", "varsamp", "stddevpop", "stddevsamp",
    "covarsamp", "covarpop", "corr",
];

fn is_aggregate_func_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.ends_with("merge") || lower.ends_with("mergestate") {
        return true;
    }
    let base = lower.strip_suffix("if").unwrap_or(&lower);
    AGGREGATE_FUNCTIONS.contains(&base)
}

/// Correctly distinguishes aggregates (count, sum, argMaxMerge, …) from
/// plain SQL functions (toDate, toString, if, …).
/// Only checks the **outermost** function call.
pub fn is_aggregate_expr(column: &str) -> bool {
    let Some(paren_pos) = column.find('(') else {
        return false;
    };
    let func_name = column[..paren_pos].trim();
    is_aggregate_func_name(func_name)
}

/// Returns true if the expression contains **any** aggregate function call
/// at any nesting depth. Used to prevent expressions like
/// `ifNotFinite(argMax(...), 0)` from being added to GROUP BY.
pub fn contains_aggregate_expr(column: &str) -> bool {
    if !column.contains('(') {
        return false;
    }
    if is_aggregate_expr(column) {
        return true;
    }
    for (i, _) in column.match_indices('(') {
        let before = &column[..i];
        let func_name = before.rsplit(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");
        if !func_name.is_empty() && is_aggregate_func_name(func_name) {
            return true;
        }
    }
    false
}

/// Result of SQL compilation, including alias remapping for HAVING support.
pub struct CompileResult {
    pub sql: String,
    pub bindings: Vec<SqlValue>,
    /// Alias → original column name. Used to remap ClickHouse JSON keys
    /// back to the column names that resolvers expect.
    pub alias_remap: Vec<(String, String)>,
}
