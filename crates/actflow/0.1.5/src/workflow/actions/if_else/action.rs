use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Result,
    common::Vars,
    runtime::Context,
    workflow::{
        actions::{Action, ActionOutput, ActionType},
        consts::{IF_ELSE_RESULT, IF_ELSE_SELECTED},
        node::NodeId,
        template,
    },
};

use super::models::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IfElseAction {
    cases: Vec<Case>,
}

impl IfElseAction {
    /// Evaluate a single comparison
    fn evaluate_comparison(
        &self,
        actual: &Option<Value>,
        operator: ComparisonOperator,
        expected: &Option<ConditionValue>,
    ) -> bool {
        match operator {
            ComparisonOperator::Null => actual.is_none() || matches!(actual, Some(Value::Null)),
            ComparisonOperator::NotNull => actual.is_some() && !matches!(actual, Some(Value::Null)),
            ComparisonOperator::Empty => match actual {
                None => true,
                Some(Value::Null) => true,
                Some(Value::String(s)) => s.is_empty(),
                Some(Value::Array(arr)) => arr.is_empty(),
                Some(Value::Object(obj)) => obj.is_empty(),
                _ => false,
            },
            ComparisonOperator::NotEmpty => match actual {
                None => false,
                Some(Value::Null) => false,
                Some(Value::String(s)) => !s.is_empty(),
                Some(Value::Array(arr)) => !arr.is_empty(),
                Some(Value::Object(obj)) => !obj.is_empty(),
                _ => true,
            },
            _ => {
                let Some(actual_val) = actual else {
                    return false;
                };
                self.evaluate_with_value(actual_val, operator, expected)
            }
        }
    }

    /// Evaluate comparison operators that require a value
    fn evaluate_with_value(
        &self,
        actual: &Value,
        operator: ComparisonOperator,
        expected: &Option<ConditionValue>,
    ) -> bool {
        let expected = match expected {
            Some(v) => v,
            None => return false,
        };

        match operator {
            ComparisonOperator::Contains => self.eval_contains(actual, expected),
            ComparisonOperator::NotContains => !self.eval_contains(actual, expected),
            ComparisonOperator::StartWith => self.eval_starts_with(actual, expected),
            ComparisonOperator::EndWith => self.eval_ends_with(actual, expected),
            ComparisonOperator::Is => self.eval_is(actual, expected),
            ComparisonOperator::IsNot => !self.eval_is(actual, expected),
            ComparisonOperator::In => self.eval_in(actual, expected),
            ComparisonOperator::NotIn => !self.eval_in(actual, expected),
            ComparisonOperator::AllOf => self.eval_all_of(actual, expected),
            ComparisonOperator::Eq => self.eval_eq(actual, expected),
            ComparisonOperator::Ne => !self.eval_eq(actual, expected),
            ComparisonOperator::Gt => self.eval_cmp(actual, expected, |a, b| a > b),
            ComparisonOperator::Lt => self.eval_cmp(actual, expected, |a, b| a < b),
            ComparisonOperator::Ge => self.eval_cmp(actual, expected, |a, b| a >= b),
            ComparisonOperator::Le => self.eval_cmp(actual, expected, |a, b| a <= b),
            _ => false,
        }
    }

    fn eval_contains(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::String(s), ConditionValue::Str(e)) => s.contains(e),
            (Value::Array(arr), ConditionValue::Str(e)) => arr.iter().any(|v| v.as_str() == Some(e.as_str())),
            _ => false,
        }
    }

    fn eval_starts_with(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::String(s), ConditionValue::Str(e)) => s.starts_with(e),
            _ => false,
        }
    }

    fn eval_ends_with(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::String(s), ConditionValue::Str(e)) => s.ends_with(e),
            _ => false,
        }
    }

    fn eval_is(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::String(s), ConditionValue::Str(e)) => s == e,
            (Value::Bool(b), ConditionValue::Str(e)) => (*b && e == "true") || (!*b && e == "false"),
            _ => false,
        }
    }

    fn eval_in(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match expected {
            ConditionValue::List(list) => match actual {
                Value::String(s) => list.contains(s),
                Value::Number(n) => list.contains(&n.to_string()),
                _ => false,
            },
            ConditionValue::Str(s) => match actual {
                Value::String(a) => s.contains(a.as_str()),
                _ => false,
            },
        }
    }

    fn eval_all_of(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::Array(arr), ConditionValue::List(list)) => list.iter().all(|e| arr.iter().any(|v| v.as_str() == Some(e.as_str()))),
            _ => false,
        }
    }

    fn eval_eq(
        &self,
        actual: &Value,
        expected: &ConditionValue,
    ) -> bool {
        match (actual, expected) {
            (Value::Number(n), ConditionValue::Str(s)) => {
                if let Ok(e) = s.parse::<f64>() {
                    n.as_f64() == Some(e)
                } else {
                    false
                }
            }
            (Value::String(a), ConditionValue::Str(e)) => a == e,
            _ => false,
        }
    }

    fn eval_cmp<F>(
        &self,
        actual: &Value,
        expected: &ConditionValue,
        cmp: F,
    ) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (actual, expected) {
            (Value::Number(n), ConditionValue::Str(s)) => {
                if let (Some(a), Ok(e)) = (n.as_f64(), s.parse::<f64>()) {
                    cmp(a, e)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Process conditions for a single case
    fn process_conditions(
        &self,
        ctx: &Context,
        conditions: &[Condition],
        logical_operator: LogicalOperator,
    ) -> bool {
        let mut results = vec![];

        for condition in conditions {
            let actual_value = template::resolve_template_to_values(ctx, &condition.variable_selector).ok().and_then(|v| v.into_iter().next());
            let result = self.evaluate_comparison(&actual_value, condition.comparison_operator, &condition.value);

            results.push(result);
        }

        let final_result = match logical_operator {
            LogicalOperator::And => results.iter().all(|r| r.clone()),
            LogicalOperator::Or => results.iter().any(|r| r.clone()),
        };

        final_result
    }
}

#[async_trait]
#[typetag::serde]
impl Action for IfElseAction {
    fn create(params: serde_json::Value) -> Result<Self> {
        jsonschema::validate(&params, &Self::schema())?;
        let action = serde_json::from_value::<Self>(params)?;
        Ok(action)
    }

    fn schema() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "type": "object",
            "properties": {
                "cases": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "case_id": { "type": "string" },
                            "logical_operator": { "type": "string", "enum": ["and", "or"] },
                            "conditions": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "variable_selector": { "type": "string" },
                                        "comparison_operator": { "type": "string" },
                                        "value": {},
                                        "sub_variable_condition": {
                                            "type": "object",
                                            "properties": {
                                                "logical_operator": { "type": "string", "enum": ["and", "or"] },
                                                "conditions": {
                                                    "type": "array",
                                                    "items": {
                                                        "type": "object",
                                                        "properties": {
                                                            "key": { "type": "string" },
                                                            "comparison_operator": { "type": "string" },
                                                            "value": {}
                                                        },
                                                        "required": ["key", "comparison_operator"]
                                                    }
                                                }
                                            },
                                            "required": ["logical_operator"]
                                        }
                                    },
                                    "required": ["variable_selector", "comparison_operator"]
                                }
                            }
                        },
                        "required": ["case_id", "logical_operator", "conditions"]
                    }
                }
            },
            "required": ["cases"]
        }"#,
        )
        .unwrap()
    }

    fn action_type(&self) -> ActionType {
        ActionType::IfElse
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        _nid: NodeId,
    ) -> Result<ActionOutput> {
        let mut selected_case_id = "false".to_string();
        let mut final_result = false;

        for case in &self.cases {
            let case_result = self.process_conditions(&ctx, &case.conditions, case.logical_operator);

            // Short-circuit: break if a case passes
            if case_result {
                selected_case_id = case.case_id.clone();
                final_result = true;
                break;
            }
        }

        let outputs = Vars::new().with(IF_ELSE_RESULT, final_result).with(IF_ELSE_SELECTED, &selected_case_id);

        Ok(ActionOutput::success(outputs))
    }
}
