use std::sync::Arc;

use crate::cube::definition::FilterValueTransform;

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

    /// Case-insensitive filter for the specified columns.
    /// Wraps the column with `lower()` and lowercases the filter value, so
    /// `lower(column) = 'lowered_value'` is used in the WHERE clause.
    pub fn lowercase_columns(&mut self, columns: &[String]) {
        if columns.is_empty() {
            return;
        }
        match self {
            FilterNode::Condition { column, value, .. } => {
                if columns.iter().any(|c| c == column) {
                    if let SqlValue::String(s) = value {
                        *s = s.to_lowercase();
                    }
                    if !column.contains('(') {
                        *column = format!("lower(`{column}`)");
                    }
                }
            }
            FilterNode::And(children) | FilterNode::Or(children) => {
                for child in children {
                    child.lowercase_columns(columns);
                }
            }
            FilterNode::ArrayIncludes { element_conditions, .. } => {
                for group in element_conditions {
                    for child in group {
                        child.lowercase_columns(columns);
                    }
                }
            }
            FilterNode::Empty => {}
        }
    }

    /// Apply per-column filter value transforms to all matching Condition nodes.
    /// `StripPrefix` applies to String values (including `In`/`NotIn` comma-joined
    /// lists); `MultiplyBy` applies to Int values and numeric String values.
    pub fn apply_filter_value_transforms(&mut self, rules: &[FilterValueTransform]) {
        if rules.is_empty() {
            return;
        }
        match self {
            FilterNode::Condition { column, op, value } => {
                let is_list = matches!(op, CompareOp::In | CompareOp::NotIn);
                for rule in rules {
                    match rule {
                        FilterValueTransform::StripPrefix { column: rule_col, prefix } => {
                            if rule_col != column || prefix.is_empty() {
                                continue;
                            }
                            let SqlValue::String(s) = value else { continue };
                            if s.is_empty() {
                                continue;
                            }
                            if is_list {
                                let joined = s
                                    .split(',')
                                    .map(|part| part.strip_prefix(prefix.as_str()).unwrap_or(part))
                                    .collect::<Vec<_>>()
                                    .join(",");
                                *s = joined;
                            } else if let Some(stripped) = s.strip_prefix(prefix.as_str()) {
                                *s = stripped.to_string();
                            }
                        }
                        FilterValueTransform::MultiplyBy { column: rule_col, factor } => {
                            if rule_col != column || *factor == 0 {
                                continue;
                            }
                            match value {
                                SqlValue::Int(n) => {
                                    *n = n.saturating_mul(*factor);
                                }
                                SqlValue::String(s) => {
                                    if is_list {
                                        let joined = s
                                            .split(',')
                                            .map(|part| match part.trim().parse::<i64>() {
                                                Ok(n) => n.saturating_mul(*factor).to_string(),
                                                Err(_) => part.to_string(),
                                            })
                                            .collect::<Vec<_>>()
                                            .join(",");
                                        *s = joined;
                                    } else if let Ok(n) = s.parse::<i64>() {
                                        *s = n.saturating_mul(*factor).to_string();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            FilterNode::And(children) | FilterNode::Or(children) => {
                for child in children {
                    child.apply_filter_value_transforms(rules);
                }
            }
            FilterNode::ArrayIncludes { element_conditions, .. } => {
                for group in element_conditions {
                    for child in group {
                        child.apply_filter_value_transforms(rules);
                    }
                }
            }
            FilterNode::Empty => {}
        }
    }

    /// Auto-detect hex address values (`0x` + 40 hex chars) and make the comparison
    /// case-insensitive by wrapping the column with `lower()` and lowercasing the value.
    pub fn normalize_hex_addresses(&mut self) {
        match self {
            FilterNode::Condition { column, value, .. } => {
                let has_hex = match value {
                    SqlValue::String(s) => s.split(',').any(Self::is_hex_address),
                    _ => false,
                };
                if has_hex {
                    if let SqlValue::String(s) = value {
                        *s = s.to_lowercase();
                    }
                    if !column.contains('(') {
                        *column = format!("lower(`{column}`)");
                    }
                }
            }
            FilterNode::And(children) | FilterNode::Or(children) => {
                for child in children {
                    child.normalize_hex_addresses();
                }
            }
            FilterNode::ArrayIncludes { element_conditions, .. } => {
                for group in element_conditions {
                    for child in group {
                        child.normalize_hex_addresses();
                    }
                }
            }
            FilterNode::Empty => {}
        }
    }

    fn is_hex_address(s: &str) -> bool {
        let s = s.trim();
        s.len() == 42 && s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::definition::FilterValueTransform;

    fn strip(column: &str, prefix: &str) -> FilterValueTransform {
        FilterValueTransform::StripPrefix {
            column: column.to_string(),
            prefix: prefix.to_string(),
        }
    }

    fn cond_eq(column: &str, value: &str) -> FilterNode {
        FilterNode::Condition {
            column: column.to_string(),
            op: CompareOp::Eq,
            value: SqlValue::String(value.to_string()),
        }
    }

    fn cond_in(column: &str, values: &[&str]) -> FilterNode {
        FilterNode::Condition {
            column: column.to_string(),
            op: CompareOp::In,
            value: SqlValue::String(values.join(",")),
        }
    }

    fn get_str(node: &FilterNode) -> &str {
        match node {
            FilterNode::Condition { value: SqlValue::String(s), .. } => s.as_str(),
            _ => panic!("expected Condition with String value"),
        }
    }

    #[test]
    fn transforms_eq_strip_prefix() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = cond_eq("token_id", "bid:solana:ADDR");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "solana:ADDR");
    }

    #[test]
    fn transforms_in_strips_each_element() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = cond_in("token_id", &["bid:solana:A", "bid:ethereum:0xff", "solana:C"]);
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "solana:A,ethereum:0xff,solana:C");
    }

    #[test]
    fn transforms_leave_value_without_prefix_unchanged() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = cond_eq("token_id", "solana:ADDR");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "solana:ADDR");
    }

    #[test]
    fn transforms_skip_nonmatching_column() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = cond_eq("currency_id", "bid:solana:ADDR");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "bid:solana:ADDR");
    }

    #[test]
    fn transforms_skip_empty_string() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = cond_eq("token_id", "");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "");
    }

    #[test]
    fn transforms_recurse_into_and_or() {
        let rules = vec![strip("token_id", "bid:")];
        let mut node = FilterNode::And(vec![
            cond_eq("token_id", "bid:solana:A"),
            FilterNode::Or(vec![
                cond_eq("token_id", "bid:ethereum:B"),
                cond_eq("currency_id", "bid:solana:C"),
            ]),
        ]);
        node.apply_filter_value_transforms(&rules);
        if let FilterNode::And(children) = &node {
            assert_eq!(get_str(&children[0]), "solana:A");
            if let FilterNode::Or(grand) = &children[1] {
                assert_eq!(get_str(&grand[0]), "ethereum:B");
                assert_eq!(get_str(&grand[1]), "bid:solana:C");
            } else {
                panic!("expected Or");
            }
        } else {
            panic!("expected And");
        }
    }

    #[test]
    fn transforms_noop_on_empty_rules() {
        let rules: Vec<FilterValueTransform> = vec![];
        let mut node = cond_eq("token_id", "bid:solana:ADDR");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "bid:solana:ADDR");
    }

    #[test]
    fn transforms_multiple_rules_same_column() {
        let rules = vec![strip("token_id", "bid:"), strip("token_id", "solana:")];
        let mut node = cond_eq("token_id", "bid:solana:ADDR");
        node.apply_filter_value_transforms(&rules);
        assert_eq!(get_str(&node), "ADDR");
    }

    #[test]
    fn transforms_skip_nonstring_value() {
        let rules = vec![strip("id", "bid:")];
        let mut node = FilterNode::Condition {
            column: "id".to_string(),
            op: CompareOp::Eq,
            value: SqlValue::Int(42),
        };
        node.apply_filter_value_transforms(&rules);
        match node {
            FilterNode::Condition { value: SqlValue::Int(v), .. } => assert_eq!(v, 42),
            _ => panic!("expected unchanged Int"),
        }
    }
}
