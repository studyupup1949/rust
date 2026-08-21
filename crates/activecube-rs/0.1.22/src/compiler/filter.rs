use async_graphql::dynamic::{ObjectAccessor, ValueAccessor};

use crate::compiler::ir::{CompareOp, FilterNode, SqlValue};
use crate::cube::definition::{ArrayFieldDef, ArrayFieldType, DimType, DimensionNode};

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
                    if matches!(dim.dim_type, DimType::Bool) {
                        if let Ok(b) = filter_val.boolean() {
                            conditions.push(FilterNode::Condition {
                                column: dim.column.clone(),
                                op: CompareOp::Eq,
                                value: SqlValue::Bool(b),
                            });
                        } else if let Ok(filter_obj) = filter_val.object() {
                            let leaf_conditions =
                                parse_leaf_filter(&filter_obj, &dim.column, &dim.dim_type)?;
                            conditions.extend(leaf_conditions);
                        }
                    } else if let Ok(filter_obj) = filter_val.object() {
                        let leaf_conditions =
                            parse_leaf_filter(&filter_obj, &dim.column, &dim.dim_type)?;
                        conditions.extend(leaf_conditions);
                    }
                }
            }
            DimensionNode::Group { graphql_name, children, .. } => {
                if let Ok(group_val) = accessor.try_get(graphql_name) {
                    if let Ok(group_obj) = group_val.object() {
                        let child_filter = parse_where(&group_obj, children)?;
                        if !child_filter.is_empty() {
                            conditions.push(child_filter);
                        }
                    }
                }
            }
            DimensionNode::Array { graphql_name, children, .. } => {
                if let Ok(arr_val) = accessor.try_get(graphql_name) {
                    if let Ok(arr_obj) = arr_val.object() {
                        let arr_filter = parse_array_includes_filter(&arr_obj, children)?;
                        if !arr_filter.is_empty() {
                            conditions.push(arr_filter);
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
        DimType::Decimal | DimType::BigInteger => &[
            ("eq", CompareOp::Eq), ("ne", CompareOp::Ne),
            ("gt", CompareOp::Gt), ("ge", CompareOp::Ge),
            ("lt", CompareOp::Lt), ("le", CompareOp::Le),
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
        ],
        DimType::String => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("like", CompareOp::Like), ("notLike", CompareOp::NotLike),
            ("includes", CompareOp::Includes), ("notIncludes", CompareOp::NotIncludes),
            ("startsWith", CompareOp::StartsWith), ("endsWith", CompareOp::EndsWith),
            ("likeCaseInsensitive", CompareOp::Ilike), ("notLikeCaseInsensitive", CompareOp::NotIlike),
            ("includesCaseInsensitive", CompareOp::IlikeIncludes),
            ("notIncludesCaseInsensitive", CompareOp::NotIlikeIncludes),
            ("startsWithCaseInsensitive", CompareOp::IlikeStartsWith),
        ],
        DimType::Date => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::DateTime => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::Bool => &[("eq", CompareOp::Eq), ("is", CompareOp::Eq)],
        DimType::Enum(_, _) => &[("is", CompareOp::Eq), ("not", CompareOp::Ne)],
    };

    for (key, op) in ops {
        if let Ok(val) = obj.try_get(key) {
            match accessor_to_sql(&val, dim_type) {
                Ok(sql_val) => {
                    conditions.push(FilterNode::Condition {
                        column: column.to_string(),
                        op: op.clone(),
                        value: sql_val,
                    });
                }
                Err(e) => {
                    let msg = e.message.to_lowercase();
                    if msg.contains("null") || msg.contains("not a string") || msg.contains("not a boolean") || msg.contains("not a float") {
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    for (key, op) in &[
        ("since_relative", CompareOp::Ge),
        ("after_relative", CompareOp::Gt),
        ("till_relative", CompareOp::Le),
    ] {
        if let Ok(val) = obj.try_get(key) {
            if let Ok(rel_obj) = val.object() {
                if let Some(expr) = parse_relative_time_accessor(&rel_obj) {
                    conditions.push(FilterNode::Condition {
                        column: column.to_string(),
                        op: op.clone(),
                        value: SqlValue::Expression(expr),
                    });
                }
            }
        }
    }

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
                        DimType::String | DimType::Date | DimType::DateTime
                        | DimType::Decimal | DimType::BigInteger => {
                            if let Ok(s) = item.string() {
                                values.push(s.to_string());
                            }
                        }
                        DimType::Int => {
                            if let Ok(i) = item.i64() {
                                values.push(i.to_string());
                            } else if let Ok(s) = item.string() {
                                if let Ok(i) = s.parse::<i64>() {
                                    values.push(i.to_string());
                                }
                            }
                        }
                        DimType::Float => {
                            if let Ok(f) = item.f64() {
                                values.push(f.to_string());
                            } else if let Ok(s) = item.string() {
                                if let Ok(f) = s.parse::<f64>() {
                                    values.push(f.to_string());
                                }
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
                if let Some(val) = obj.get(dim.graphql_name.as_str()) {
                    if matches!(dim.dim_type, DimType::Bool) {
                        if let async_graphql::Value::Boolean(b) = val {
                            conditions.push(FilterNode::Condition {
                                column: dim.column.clone(),
                                op: CompareOp::Eq,
                                value: SqlValue::Bool(*b),
                            });
                        } else if let async_graphql::Value::Object(filter_map) = val {
                            let leaf = parse_leaf_filter_from_value(filter_map, &dim.column, &dim.dim_type)?;
                            conditions.extend(leaf);
                        }
                    } else if let async_graphql::Value::Object(filter_map) = val {
                        let leaf = parse_leaf_filter_from_value(filter_map, &dim.column, &dim.dim_type)?;
                        conditions.extend(leaf);
                    }
                }
            }
            DimensionNode::Group { graphql_name, children, .. } => {
                if let Some(group_val) = obj.get(graphql_name.as_str()) {
                    let child_filter = parse_filter_from_value(group_val, children)?;
                    if !child_filter.is_empty() {
                        conditions.push(child_filter);
                    }
                }
            }
            DimensionNode::Array { graphql_name, children, .. } => {
                if let Some(async_graphql::Value::Object(arr_map)) = obj.get(graphql_name.as_str()) {
                    let arr_filter = parse_array_includes_from_value(arr_map, children)?;
                    if !arr_filter.is_empty() {
                        conditions.push(arr_filter);
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
        DimType::Decimal | DimType::BigInteger => &[
            ("eq", CompareOp::Eq), ("ne", CompareOp::Ne),
            ("gt", CompareOp::Gt), ("ge", CompareOp::Ge),
            ("lt", CompareOp::Lt), ("le", CompareOp::Le),
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
        ],
        DimType::String => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("like", CompareOp::Like), ("notLike", CompareOp::NotLike),
            ("includes", CompareOp::Includes), ("notIncludes", CompareOp::NotIncludes),
            ("startsWith", CompareOp::StartsWith), ("endsWith", CompareOp::EndsWith),
            ("likeCaseInsensitive", CompareOp::Ilike), ("notLikeCaseInsensitive", CompareOp::NotIlike),
            ("includesCaseInsensitive", CompareOp::IlikeIncludes),
            ("notIncludesCaseInsensitive", CompareOp::NotIlikeIncludes),
            ("startsWithCaseInsensitive", CompareOp::IlikeStartsWith),
        ],
        DimType::Date => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::DateTime => &[
            ("is", CompareOp::Eq), ("not", CompareOp::Ne),
            ("after", CompareOp::Gt), ("since", CompareOp::Ge),
            ("before", CompareOp::Lt), ("till", CompareOp::Le),
        ],
        DimType::Bool => &[("eq", CompareOp::Eq), ("is", CompareOp::Eq)],
        DimType::Enum(_, _) => &[("is", CompareOp::Eq), ("not", CompareOp::Ne)],
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

    for (key, op) in &[
        ("since_relative", CompareOp::Ge),
        ("after_relative", CompareOp::Gt),
        ("till_relative", CompareOp::Le),
    ] {
        if let Some(async_graphql::Value::Object(rel_map)) = obj.get(*key) {
            if let Some(expr) = parse_relative_time_value(rel_map) {
                conditions.push(FilterNode::Condition {
                    column: column.to_string(),
                    op: op.clone(),
                    value: SqlValue::Expression(expr),
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
                (DimType::String | DimType::Date | DimType::DateTime | DimType::Decimal | DimType::BigInteger, async_graphql::Value::String(s)) => Some(s.clone()),
                (DimType::String | DimType::Date | DimType::Enum(_, _), async_graphql::Value::Enum(e)) => Some(e.to_string()),
                (DimType::Enum(_, _), async_graphql::Value::String(s)) => Some(s.clone()),
                (DimType::Decimal | DimType::BigInteger, async_graphql::Value::Number(n)) => n.as_f64().map(|f| f.to_string()),
                (DimType::Int, async_graphql::Value::String(s)) => s.parse::<i64>().ok().map(|i| i.to_string()),
                (DimType::Int, async_graphql::Value::Number(n)) => n.as_i64().map(|i| i.to_string()),
                (DimType::Float, async_graphql::Value::String(s)) => s.parse::<f64>().ok().map(|f| f.to_string()),
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

fn normalize_datetime(s: &str) -> String {
    let s = s.trim_end_matches('Z');
    s.replacen('T', " ", 1)
}

/// Truncate an ISO datetime string to just the date portion (YYYY-MM-DD).
/// ClickHouse `Date` columns cannot compare against full datetime strings.
fn normalize_date(s: &str) -> String {
    if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') {
        s[..10].to_string()
    } else {
        s.to_string()
    }
}

fn value_to_sql(val: &async_graphql::Value, dim_type: &DimType) -> Option<SqlValue> {
    match (dim_type, val) {
        (DimType::Int, async_graphql::Value::Number(n)) => n.as_i64().map(SqlValue::Int),
        (DimType::Int, async_graphql::Value::String(s)) => s.parse::<i64>().ok().map(SqlValue::Int),
        (DimType::Float, async_graphql::Value::String(s)) => s.parse::<f64>().ok().map(SqlValue::Float),
        (DimType::Float, async_graphql::Value::Number(n)) => n.as_f64().map(SqlValue::Float),
        (DimType::Decimal | DimType::BigInteger, async_graphql::Value::String(s)) => Some(SqlValue::String(s.clone())),
        (DimType::Decimal | DimType::BigInteger, async_graphql::Value::Number(n)) => n.as_f64().map(|f| SqlValue::String(f.to_string())),
        (DimType::Bool, async_graphql::Value::Boolean(b)) => Some(SqlValue::Bool(*b)),
        (DimType::DateTime, async_graphql::Value::String(s)) => {
            Some(SqlValue::String(normalize_datetime(s)))
        }
        (DimType::Date, async_graphql::Value::String(s)) => {
            Some(SqlValue::String(normalize_date(s)))
        }
        (DimType::String, async_graphql::Value::String(s)) => {
            Some(SqlValue::String(s.clone()))
        }
        (DimType::String | DimType::Date | DimType::Enum(_, _), async_graphql::Value::Enum(e)) => {
            Some(SqlValue::String(e.to_string()))
        }
        (DimType::DateTime, async_graphql::Value::Enum(e)) => {
            Some(SqlValue::String(normalize_datetime(e.as_ref())))
        }
        (DimType::Enum(_, _), async_graphql::Value::String(s)) => {
            Some(SqlValue::String(s.clone()))
        }
        (DimType::String, async_graphql::Value::Number(n)) => {
            Some(SqlValue::String(n.to_string()))
        }
        _ => None,
    }
}

fn accessor_to_sql(
    val: &ValueAccessor,
    dim_type: &DimType,
) -> Result<SqlValue, async_graphql::Error> {
    match dim_type {
        DimType::Int => {
            if let Ok(i) = val.i64() {
                Ok(SqlValue::Int(i))
            } else if let Ok(s) = val.string() {
                let i = s.parse::<i64>().map_err(|_| {
                    async_graphql::Error::new(format!("Invalid integer value: {s}"))
                })?;
                Ok(SqlValue::Int(i))
            } else {
                Err(async_graphql::Error::new("Expected integer value"))
            }
        }
        DimType::Float => {
            if let Ok(f) = val.f64() {
                Ok(SqlValue::Float(f))
            } else if let Ok(s) = val.string() {
                let f = s.parse::<f64>().map_err(|_| {
                    async_graphql::Error::new(format!("Invalid float value: {s}"))
                })?;
                Ok(SqlValue::Float(f))
            } else {
                Err(async_graphql::Error::new("Expected float value"))
            }
        }
        DimType::Decimal | DimType::BigInteger => {
            if let Ok(s) = val.string() {
                Ok(SqlValue::String(s.to_string()))
            } else if let Ok(f) = val.f64() {
                Ok(SqlValue::String(f.to_string()))
            } else {
                Err(async_graphql::Error::new("Expected decimal/bigint value"))
            }
        }
        DimType::Bool => Ok(SqlValue::Bool(val.boolean()?)),
        DimType::DateTime => {
            if let Ok(s) = val.string() {
                Ok(SqlValue::String(normalize_datetime(s)))
            } else if let Ok(s) = val.enum_name() {
                Ok(SqlValue::String(normalize_datetime(s)))
            } else {
                Err(async_graphql::Error::new(
                    "DateTime filter requires a string value (e.g. \"2026-01-01T00:00:00Z\"), got null or non-string"
                ))
            }
        }
        DimType::Date => {
            if let Ok(s) = val.string() {
                Ok(SqlValue::String(normalize_date(s)))
            } else if let Ok(s) = val.enum_name() {
                Ok(SqlValue::String(normalize_date(s)))
            } else {
                Err(async_graphql::Error::new("Expected date value"))
            }
        }
        DimType::String | DimType::Enum(_, _) => {
            if let Ok(s) = val.string() {
                Ok(SqlValue::String(s.to_string()))
            } else if let Ok(s) = val.enum_name() {
                Ok(SqlValue::String(s.to_string()))
            } else {
                Err(async_graphql::Error::new("Expected string or enum value"))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Array includes filter parsing
// ---------------------------------------------------------------------------

fn array_columns_from_children(children: &[ArrayFieldDef]) -> Vec<String> {
    let mut cols = Vec::new();
    for f in children {
        if let ArrayFieldType::Group(group_children) = &f.field_type {
            for gc in group_children {
                cols.push(gc.column.clone());
            }
        } else {
            cols.push(f.column.clone());
        }
    }
    cols
}

fn dim_type_for_array_field(field: &ArrayFieldDef) -> DimType {
    match &field.field_type {
        ArrayFieldType::Scalar(dt) => dt.clone(),
        ArrayFieldType::Union(_) | ArrayFieldType::Group(_) => DimType::String,
    }
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Find the "Type" sibling column within array children (used for Union type discrimination).
fn find_type_column(children: &[ArrayFieldDef]) -> Option<String> {
    children.iter().find(|f| f.graphql_name == "Type").map(|f| f.column.clone())
}

/// Build a type discrimination condition for a Union variant.
fn build_type_discrimination(type_column: &str, source_type_names: &[String]) -> Option<FilterNode> {
    if source_type_names.is_empty() {
        return None;
    }
    if source_type_names.len() == 1 {
        Some(FilterNode::Condition {
            column: type_column.to_string(),
            op: CompareOp::Eq,
            value: SqlValue::String(source_type_names[0].clone()),
        })
    } else {
        Some(FilterNode::Condition {
            column: type_column.to_string(),
            op: CompareOp::In,
            value: SqlValue::String(source_type_names.join(",")),
        })
    }
}

fn parse_single_includes_object(
    obj: &ObjectAccessor,
    children: &[ArrayFieldDef],
) -> Result<Vec<FilterNode>, async_graphql::Error> {
    let mut conds = Vec::new();
    let type_column = find_type_column(children);
    for field in children {
        if let Ok(filter_val) = obj.try_get(&field.graphql_name) {
            if let Ok(filter_obj) = filter_val.object() {
                match &field.field_type {
                    ArrayFieldType::Union(variants) => {
                        for v in variants {
                            let key = capitalize_first(&v.field_name);
                            if let Ok(variant_val) = filter_obj.try_get(&key) {
                                if let Ok(variant_filter) = variant_val.object() {
                                    if let Some(ref tc) = type_column {
                                        if let Some(disc) = build_type_discrimination(tc, &v.source_type_names) {
                                            conds.push(disc);
                                        }
                                    }
                                    let leaf = parse_leaf_filter(&variant_filter, &field.column, &v.source_type)?;
                                    conds.extend(leaf);
                                }
                            }
                        }
                    }
                    _ => {
                        let dt = dim_type_for_array_field(field);
                        let leaf = parse_leaf_filter(&filter_obj, &field.column, &dt)?;
                        conds.extend(leaf);
                    }
                }
            }
        }
    }
    Ok(conds)
}

/// Parse `includes` from an ObjectAccessor (runtime GraphQL args).
fn parse_array_includes_filter(
    obj: &ObjectAccessor,
    children: &[ArrayFieldDef],
) -> Result<FilterNode, async_graphql::Error> {
    if let Ok(includes_val) = obj.try_get("includes") {
        let array_columns = array_columns_from_children(children);
        if let Ok(list) = includes_val.list() {
            let mut all_conditions = Vec::new();
            for item in list.iter() {
                if let Ok(item_obj) = item.object() {
                    let conds = parse_single_includes_object(&item_obj, children)?;
                    if !conds.is_empty() {
                        all_conditions.push(conds);
                    }
                }
            }
            if all_conditions.is_empty() {
                return Ok(FilterNode::Empty);
            }
            return Ok(FilterNode::ArrayIncludes { array_columns, element_conditions: all_conditions });
        }
        if let Ok(single_obj) = includes_val.object() {
            let conds = parse_single_includes_object(&single_obj, children)?;
            if conds.is_empty() {
                return Ok(FilterNode::Empty);
            }
            return Ok(FilterNode::ArrayIncludes { array_columns, element_conditions: vec![conds] });
        }
    }
    Ok(FilterNode::Empty)
}

fn parse_single_includes_from_value(
    map: &indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
    children: &[ArrayFieldDef],
) -> Result<Vec<FilterNode>, async_graphql::Error> {
    let mut conds = Vec::new();
    let type_column = find_type_column(children);
    for field in children {
        if let Some(async_graphql::Value::Object(filter_map)) = map.get(field.graphql_name.as_str()) {
            match &field.field_type {
                ArrayFieldType::Union(variants) => {
                    for v in variants {
                        let key = capitalize_first(&v.field_name);
                        if let Some(async_graphql::Value::Object(variant_map)) = filter_map.get(key.as_str()) {
                            if let Some(ref tc) = type_column {
                                if let Some(disc) = build_type_discrimination(tc, &v.source_type_names) {
                                    conds.push(disc);
                                }
                            }
                            let leaf = parse_leaf_filter_from_value(variant_map, &field.column, &v.source_type)?;
                            conds.extend(leaf);
                        }
                    }
                }
                _ => {
                    let dt = dim_type_for_array_field(field);
                    let leaf = parse_leaf_filter_from_value(filter_map, &field.column, &dt)?;
                    conds.extend(leaf);
                }
            }
        }
    }
    Ok(conds)
}

/// Parse `includes` from a raw Value map (used in parse_filter_from_value path).
fn parse_array_includes_from_value(
    obj: &indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
    children: &[ArrayFieldDef],
) -> Result<FilterNode, async_graphql::Error> {
    let array_columns = array_columns_from_children(children);

    if let Some(async_graphql::Value::List(items)) = obj.get("includes") {
        let mut all_conditions = Vec::new();
        for item in items {
            if let async_graphql::Value::Object(item_map) = item {
                let conds = parse_single_includes_from_value(item_map, children)?;
                if !conds.is_empty() {
                    all_conditions.push(conds);
                }
            }
        }
        if all_conditions.is_empty() {
            return Ok(FilterNode::Empty);
        }
        return Ok(FilterNode::ArrayIncludes { array_columns, element_conditions: all_conditions });
    }

    if let Some(async_graphql::Value::Object(single_map)) = obj.get("includes") {
        let conds = parse_single_includes_from_value(single_map, children)?;
        if conds.is_empty() {
            return Ok(FilterNode::Empty);
        }
        return Ok(FilterNode::ArrayIncludes { array_columns, element_conditions: vec![conds] });
    }

    Ok(FilterNode::Empty)
}

fn parse_relative_time_accessor(obj: &ObjectAccessor) -> Option<String> {
    if let Ok(v) = obj.try_get("minutes_ago") {
        if let Ok(n) = v.i64() { return Some(format!("now() - INTERVAL {n} MINUTE")); }
    }
    if let Ok(v) = obj.try_get("hours_ago") {
        if let Ok(n) = v.i64() { return Some(format!("now() - INTERVAL {n} HOUR")); }
    }
    if let Ok(v) = obj.try_get("days_ago") {
        if let Ok(n) = v.i64() { return Some(format!("now() - INTERVAL {n} DAY")); }
    }
    None
}

fn parse_relative_time_value(
    obj: &indexmap::IndexMap<async_graphql::Name, async_graphql::Value>,
) -> Option<String> {
    if let Some(async_graphql::Value::Number(n)) = obj.get("minutes_ago") {
        if let Some(v) = n.as_i64() { return Some(format!("now() - INTERVAL {v} MINUTE")); }
    }
    if let Some(async_graphql::Value::Number(n)) = obj.get("hours_ago") {
        if let Some(v) = n.as_i64() { return Some(format!("now() - INTERVAL {v} HOUR")); }
    }
    if let Some(async_graphql::Value::Number(n)) = obj.get("days_ago") {
        if let Some(v) = n.as_i64() { return Some(format!("now() - INTERVAL {v} DAY")); }
    }
    None
}

