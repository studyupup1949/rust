use serde::{Deserialize, Serialize};

/// Logical operator
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LogicalOperator {
    #[default]
    And,
    Or,
}

/// Comparison operator
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ComparisonOperator {
    // for string or array
    Contains,
    NotContains,
    StartWith,
    EndWith,
    Is,
    IsNot,
    Empty,
    NotEmpty,
    In,
    NotIn,
    AllOf,
    // for number
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Null,
    NotNull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubCondition {
    pub key: String,
    pub comparison_operator: ComparisonOperator,
    pub value: Option<ConditionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubVariableCondition {
    pub logical_operator: LogicalOperator,

    #[serde(default)]
    pub conditions: Vec<SubCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub variable_selector: String,
    pub comparison_operator: ComparisonOperator,
    pub value: Option<ConditionValue>,
    pub sub_variable_condition: Option<SubVariableCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub case_id: String,
    pub logical_operator: LogicalOperator,
    pub conditions: Vec<Condition>,
}
