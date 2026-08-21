use async_graphql::dynamic::{ObjectAccessor, ValueAccessor};

use crate::compiler::ir::{CompareOp, FilterNode, SqlValue};
use crate::cube::definition::{DimType, DimensionNode};

pub fn parse_where(
    accessor: &ObjectAccessor,
    dimensions: &[DimensionNode],
) -> Result<FilterNode, async_graphql::Error> {
    let mut conditions = Vec::new();

    if let Ok(any_val) = accessor.try_get("any") {
        if let Ok(list) = any_val.list() {
            let mut or_children = Vec::new();
            for item in list.iter() {
                if let Ok(obj) = item.object() {
                    let child = parse_where(&obj, dimensions)?;
                    if !child.is_empty() {
                        or_children.push(child);
                    }
                }
            }
            if !or_children.is_empty() {
                conditions.push(FilterNode::Or(or_children));
            }
        }
    }

    for node in dimensions {
        match node {
            DimensionNode::Leaf(dim) => {
                if let Ok(filter_val) = accessor.try_get(&dim.graphql_name) {
                    if let Ok(filter_obj) = filter_val.object() {
                        let leaf_conditions =
                            parse_leaf_filter(&filter_obj, &dim.column, &dim.dim_type)?;
                        conditions.extend(leaf_conditions);
                    }
                }
            }
            DimensionNode::Group { graphql_name, children } => {
                if let Ok(group_val) = accessor.try_get(graphql_name) {
                    if let Ok(group_obj) = group_val.object() {
                        let child_filter = parse_where(&group_obj, children)?;
                        if !child_filter.is_empty() {
                            conditions.push(child_filter);
                        }
                    }
                }
            }
        }
    }

    Ok(match conditions.len() {
        0 => FilterNode::Empty,
        1 => conditions.into_iter().next().unwrap(),
        _ => FilterNode::And(conditions),
    })
}

/// Public entry point for selector-based filters. Same logic as dimension leaf filters
/// but callable from the parser for cube-level selector arguments.
pub fn parse_leaf_filter_for_selector(
    obj: &ObjectAccessor,
    column: &str,
    dim_type: &DimType,
) -> Result<Vec<FilterNode>, async_graphql::Error> {
    parse_leaf_filter(obj, column, dim_type)
}

fn parse_leaf_filter(
    obj: &ObjectAccessor,
    column: &str,
    dim_type: &DimType,
) -> Result<Vec<FilterNode>, async_graphql::Error> {
    let mut conditions = Vec::new();

    let ops: &[(&str, CompareOp)] = match dim_type {
        DimType::Int | DimType::Float => &[
            ("eq", CompareOp::Eq), ("ne", CompareOp::Ne),
            ("gt", CompareOp::Gt), ("ge", CompareOp::Ge),
            ("lt", CompareOp::Lt), ("le", CompareOp::Le),
        ],
        DimType::String => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("like", CompareOp::Like), ("includes", CompareOp::Includes),
        ],
        DimType::DateTime => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::Bool => &[("eq", CompareOp::Eq)],
    };

    for (key, op) in ops {
        if let Ok(val) = obj.try_get(key) {
            let sql_val = accessor_to_sql(&val, dim_type)?;
            conditions.push(FilterNode::Condition {
                column: column.to_string(),
                op: op.clone(),
                value: sql_val,
            });
        }
    }

    // isNull: true → IS NULL, isNull: false → IS NOT NULL
    if let Ok(val) = obj.try_get("isNull") {
        if let Ok(b) = val.boolean() {
            conditions.push(FilterNode::Condition {
                column: column.to_string(),
                op: if b { CompareOp::IsNull } else { CompareOp::IsNotNull },
                value: SqlValue::Bool(b),
            });
        }
    }

    for (key, op) in &[("in", CompareOp::In), ("notIn", CompareOp::NotIn)] {
        if let Ok(val) = obj.try_get(key) {
            if let Ok(list) = val.list() {
                let mut values = Vec::new();
                for item in list.iter() {
                    match dim_type {
                        DimType::String | DimType::DateTime => {
                            if let Ok(s) = item.string() {
                                values.push(s.to_string());
                            }
                        }
                        DimType::Int => {
                            if let Ok(n) = item.i64() {
                                values.push(n.to_string());
                            }
                        }
                        DimType::Float => {
                            if let Ok(f) = item.f64() {
                                values.push(f.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                if !values.is_empty() {
                    conditions.push(FilterNode::Condition {
                        column: column.to_string(),
                        op: op.clone(),
                        value: SqlValue::String(values.join(",")),
                    });
                }
            }
        }
    }

    Ok(conditions)
}

/// Parse a filter from a raw `async_graphql::Value` (used when ObjectAccessor is unavailable,
/// e.g. from selection-set arguments on metric fields).
pub fn parse_filter_from_value(
    val: &async_graphql::Value,
    dimensions: &[DimensionNode],
) -> Result<FilterNode, async_graphql::Error> {
    let obj = match val {
        async_graphql::Value::Object(map) => map,
        _ => return Ok(FilterNode::Empty),
    };

    let mut conditions = Vec::new();

    if let Some(async_graphql::Value::List(items)) = obj.get("any") {
        let mut or_children = Vec::new();
        for item in items {
            let child = parse_filter_from_value(item, dimensions)?;
            if !child.is_empty() {
                or_children.push(child);
            }
        }
        if !or_children.is_empty() {
            conditions.push(FilterNode::Or(or_children));
        }
    }

    for node in dimensions {
        match node {
            DimensionNode::Leaf(dim) => {
                if let Some(async_graphql::Value::Object(filter_map)) = obj.get(dim.graphql_name.as_str()) {
                    let leaf = parse_leaf_filter_from_value(filter_map, &dim.column, &dim.dim_type)?;
                    conditions.extend(leaf);
                }
            }
            DimensionNode::Group { graphql_name, children } => {
                if let Some(group_val) = obj.get(graphql_name.as_str()) {
                    let child_filter = parse_filter_from_value(group_val, children)?;
                    if !child_filter.is_empty() {
                        conditions.push(child_filter);
                    }
                }
            }
        }
    }

    Ok(match conditions.len() {
        0 => FilterNode::Empty,
        1 => conditions.into_iter().next().unwrap(),
        _ => FilterNode::And(conditions),
    })
}

fn parse_leaf_filter_from_value(
    obj: &indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
    column: &str,
    dim_type: &DimType,
) -> Result<Vec<FilterNode>, async_graphql::Error> {
    let mut conditions = Vec::new();

    let ops: &[(&str, CompareOp)] = match dim_type {
        DimType::Int | DimType::Float => &[
            ("eq", CompareOp::Eq), ("ne", CompareOp::Ne),
            ("gt", CompareOp::Gt), ("ge", CompareOp::Ge),
            ("lt", CompareOp::Lt), ("le", CompareOp::Le),
        ],
        DimType::String => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("like", CompareOp::Like), ("includes", CompareOp::Includes),
        ],
        DimType::DateTime => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::Bool => &[("eq", CompareOp::Eq)],
    };

    for (key, op) in ops {
        if let Some(val) = obj.get(*key) {
            if let Some(sql_val) = value_to_sql(val, dim_type) {
                conditions.push(FilterNode::Condition {
                    column: column.to_string(),
                    op: op.clone(),
                    value: sql_val,
                });
            }
        }
    }

    if let Some(async_graphql::Value::Boolean(b)) = obj.get("isNull") {
        conditions.push(FilterNode::Condition {
            column: column.to_string(),
            op: if *b { CompareOp::IsNull } else { CompareOp::IsNotNull },
            value: SqlValue::Bool(*b),
        });
    }

    for (key, op) in &[("in", CompareOp::In), ("notIn", CompareOp::NotIn)] {
        if let Some(async_graphql::Value::List(list)) = obj.get(*key) {
            let values: Vec<String> = list.iter().filter_map(|item| match (dim_type, item) {
                (DimType::String | DimType::DateTime, async_graphql::Value::String(s)) => Some(s.clone()),
                (DimType::Int, async_graphql::Value::Number(n)) => n.as_i64().map(|i| i.to_string()),
                (DimType::Float, async_graphql::Value::Number(n)) => n.as_f64().map(|f| f.to_string()),
                _ => None,
            }).collect();
            if !values.is_empty() {
                conditions.push(FilterNode::Condition {
                    column: column.to_string(),
                    op: op.clone(),
                    value: SqlValue::String(values.join(",")),
                });
            }
        }
    }

    Ok(conditions)
}

fn value_to_sql(val: &async_graphql::Value, dim_type: &DimType) -> Option<SqlValue> {
    match (dim_type, val) {
        (DimType::Int, async_graphql::Value::Number(n)) => n.as_i64().map(SqlValue::Int),
        (DimType::Float, async_graphql::Value::Number(n)) => n.as_f64().map(SqlValue::Float),
        (DimType::Bool, async_graphql::Value::Boolean(b)) => Some(SqlValue::Bool(*b)),
        (DimType::String | DimType::DateTime, async_graphql::Value::String(s)) => {
            Some(SqlValue::String(s.clone()))
        }
        (DimType::Int, async_graphql::Value::String(s)) => s.parse::<i64>().ok().map(SqlValue::Int),
        (DimType::Float, async_graphql::Value::String(s)) => s.parse::<f64>().ok().map(SqlValue::Float),
        _ => None,
    }
}

fn accessor_to_sql(
    val: &ValueAccessor,
    dim_type: &DimType,
) -> Result<SqlValue, async_graphql::Error> {
    match dim_type {
        DimType::Int => Ok(SqlValue::Int(val.i64()?)),
        DimType::Float => Ok(SqlValue::Float(val.f64()?)),
        DimType::Bool => Ok(SqlValue::Bool(val.boolean()?)),
        DimType::String | DimType::DateTime => Ok(SqlValue::String(val.string()?.to_string())),
    }
}

