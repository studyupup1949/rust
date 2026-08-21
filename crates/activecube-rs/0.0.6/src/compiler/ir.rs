/// SQL binding value — database-agnostic representation.
#[derive(Debug, Clone)]
pub enum SqlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
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
    /// When true, append FINAL after FROM for ReplacingMergeTree tables.
    pub use_final: bool,
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
    In,
    NotIn,
    Includes,
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
            CompareOp::In => "IN",
            CompareOp::NotIn => "NOT IN",
            CompareOp::Includes => "LIKE",
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

impl FilterNode {
    pub fn is_empty(&self) -> bool {
        matches!(self, FilterNode::Empty)
    }
}
