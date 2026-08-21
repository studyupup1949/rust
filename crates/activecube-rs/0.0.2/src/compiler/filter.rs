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

    if matches!(dim_type, DimType::String) {
        for (key, op) in &[("in", CompareOp::In), ("notIn", CompareOp::NotIn)] {
            if let Ok(val) = obj.try_get(key) {
                if let Ok(list) = val.list() {
                    let mut values = Vec::new();
                    for item in list.iter() {
                        if let Ok(s) = item.string() {
                            values.push(s.to_string());
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
    }

    Ok(conditions)
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

