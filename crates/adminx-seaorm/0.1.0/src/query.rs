// adminx-seaorm/src/query.rs
//
// Dynamic, schema-agnostic SQL builders. We drive `sea_query` with runtime
// table/column names and read rows back as `serde_json::Value`, reproducing the
// document-oriented ergonomics of the Mongo backend on SQL.

use adminx_core::storage::{FilterClause, FilterOp};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use sea_orm::sea_query::{Alias, Expr, Order, Query, SelectStatement, SimpleExpr};
use sea_orm::Value as SeaValue;
use serde_json::Value;

/// Bind a range-filter value. ISO date/datetime text binds as a UTC timestamp
/// (so it compares against `timestamp`/`timestamptz` columns); anything else
/// falls back to a scalar (e.g. numeric ranges).
fn range_expr(v: &str) -> SimpleExpr {
    let ndt = NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(v, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        });
    match ndt {
        Some(ndt) => Expr::val(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)).into(),
        None => Expr::val(filter_value(v)).into(),
    }
}

/// Coerce a filter's text value into a bound SQL value: `true`/`false` → bool,
/// integer text → i64, everything else stays text.
fn filter_value(v: &str) -> SeaValue {
    match v {
        "true" => true.into(),
        "false" => false.into(),
        _ => match v.parse::<i64>() {
            Ok(i) => i.into(),
            Err(_) => v.to_owned().into(),
        },
    }
}

/// Apply column filters as `AND` conditions on a select statement.
fn apply_filters(select: &mut SelectStatement, filters: &[FilterClause]) {
    for f in filters {
        let col = Expr::col(Alias::new(&f.field));
        match f.op {
            FilterOp::Eq => {
                select.and_where(col.eq(filter_value(&f.value)));
            }
            FilterOp::Contains => {
                select.and_where(col.like(format!("%{}%", f.value)));
            }
            FilterOp::Gte => {
                select.and_where(col.gte(range_expr(&f.value)));
            }
            FilterOp::Lte => {
                select.and_where(col.lte(range_expr(&f.value)));
            }
        }
    }
}

/// Convert a JSON scalar into a bound SQL value. Objects/arrays are stored as
/// their JSON text form; nulls bind as a typeless SQL NULL.
pub fn json_to_sea_value(v: &Value) -> SeaValue {
    match v {
        Value::Null => SeaValue::String(None),
        Value::Bool(b) => (*b).into(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into()
            } else if let Some(u) = n.as_u64() {
                (u as i64).into()
            } else if let Some(f) = n.as_f64() {
                f.into()
            } else {
                n.to_string().into()
            }
        }
        Value::String(s) => s.clone().into(),
        other => other.to_string().into(),
    }
}

/// Coerce a path id into a comparison value: numeric when it parses, else text.
pub fn id_to_sea_value(id: &str) -> SeaValue {
    if let Ok(i) = id.parse::<i64>() {
        i.into()
    } else {
        id.to_owned().into()
    }
}

/// Wrap a bound value as a `SimpleExpr` for insert value lists.
pub fn value_expr(v: SeaValue) -> SimpleExpr {
    Expr::val(v).into()
}

pub fn build_list_select(
    table: &str,
    per_page: u64,
    offset: u64,
    sort_by: &Option<String>,
    sort_desc: bool,
    filters: &[FilterClause],
) -> SelectStatement {
    let mut select = Query::select();
    select
        .expr(Expr::cust("*"))
        .from(Alias::new(table))
        .limit(per_page)
        .offset(offset);
    apply_filters(&mut select, filters);

    if let Some(col) = sort_by {
        let order = if sort_desc { Order::Desc } else { Order::Asc };
        select.order_by(Alias::new(col), order);
    }

    select.to_owned()
}

pub fn build_count_select(table: &str, filters: &[FilterClause]) -> SelectStatement {
    let mut select = Query::select();
    select
        .expr(Expr::cust("COUNT(*) AS count"))
        .from(Alias::new(table));
    apply_filters(&mut select, filters);
    select.to_owned()
}

pub fn build_get_select(table: &str, pk: &str, id: &str) -> SelectStatement {
    Query::select()
        .expr(Expr::cust("*"))
        .from(Alias::new(table))
        .and_where(Expr::col(Alias::new(pk)).eq(id_to_sea_value(id)))
        .limit(1)
        .to_owned()
}

/// `SELECT * FROM table WHERE column = value LIMIT 1` with the value bound as
/// text (used for auth lookups by email).
pub fn build_find_select(table: &str, column: &str, value: &str) -> SelectStatement {
    Query::select()
        .expr(Expr::cust("*"))
        .from(Alias::new(table))
        .and_where(Expr::col(Alias::new(column)).eq(value.to_owned()))
        .limit(1)
        .to_owned()
}
