use std::cmp;
#[cfg(feature = "seaorm")]
use std::collections::BTreeMap;
use std::hash::Hash;

use actix_cloud::utils;
use anyhow::Result;
use derivative::Derivative;
#[cfg(feature = "seaorm")]
use sea_orm::{
    ExprTrait as _, FromQueryResult, Order, QueryOrder as _,
    entity::prelude::async_trait::async_trait,
    prelude::*,
    sea_query::{ColumnRef, LikeExpr, SimpleExpr},
};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_with::{DisplayFromStr, serde_as};
use validator::{Validate, ValidationError};

use crate::HyUuid;
#[cfg(feature = "entity")]
use crate::entity::DefaultColumnTrait;
#[cfg(feature = "seaorm")]
use crate::utils::StringUtil as _;

#[derive(Serialize, Debug)]
pub struct PageData<T> {
    pub total: u64,
    pub data: Vec<T>,
}

impl<T> PageData<T> {
    pub fn new(data: (Vec<T>, u64)) -> Self {
        Self {
            total: data.1,
            data: data.0,
        }
    }
}

#[serde_as]
#[serde_inline_default]
#[derive(Derivative, Deserialize, Validate, Clone, Debug)]
#[derivative(Default(new = "true"))]
pub struct PaginationParam {
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    #[derivative(Default(value = "1"))]
    #[serde_as(as = "DisplayFromStr")]
    pub page: u64,

    #[validate(range(min = 1, max = 100))]
    #[serde_inline_default(10)]
    #[derivative(Default(value = "10"))]
    #[serde_as(as = "DisplayFromStr")]
    pub size: u64,
}

impl PaginationParam {
    pub fn left(&self) -> u64 {
        self.page.saturating_sub(1).saturating_mul(self.size)
    }

    pub fn right(&self) -> u64 {
        self.page.saturating_mul(self.size)
    }

    pub fn round(p: u64) -> usize {
        cmp::min(p, usize::MAX as u64) as usize
    }

    pub fn split<T>(&self, data: Vec<T>) -> PageData<T> {
        let cnt = data.len();
        PageData::new((
            data.into_iter()
                .skip(Self::round(self.left()))
                .take(Self::round(self.size))
                .collect(),
            cnt as u64,
        ))
    }
}

#[cfg(feature = "seaorm")]
#[async_trait]
pub trait SelectPage<E, C>
where
    E: EntityTrait,
    C: ConnectionTrait,
{
    async fn select_page(self, db: &C, p: Option<PaginationParam>) -> Result<(Vec<E::Model>, u64)>;
}

#[cfg(feature = "seaorm")]
#[async_trait]
impl<M, E, C> SelectPage<E, C> for Select<E>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
    C: ConnectionTrait,
{
    async fn select_page(self, db: &C, p: Option<PaginationParam>) -> Result<(Vec<E::Model>, u64)> {
        if let Some(page) = p {
            let q = self.paginate(db, page.size);
            Ok((q.fetch_page(page.page - 1).await?, q.num_items().await?))
        } else {
            let res = self.all(db).await?;
            let cnt = res.len() as u64;
            Ok((res, cnt))
        }
    }
}

#[cfg(feature = "seaorm")]
pub trait IntoExpr {
    fn like_expr<T>(&self, col: T) -> SimpleExpr
    where
        T: ColumnTrait;
}

#[cfg(feature = "seaorm")]
impl IntoExpr for &String {
    fn like_expr<T>(&self, col: T) -> SimpleExpr
    where
        T: ColumnTrait,
    {
        sea_orm::prelude::Expr::col((col.entity_name(), col))
            .like(LikeExpr::new(format!("%{}%", self.like_escape())).escape('\\'))
    }
}

#[cfg(feature = "seaorm")]
pub struct Condition {
    pub page: Option<PaginationParam>,
    pub cond: sea_orm::Condition,
    pub order: Vec<(SimpleExpr, Order)>,
}

#[cfg(feature = "seaorm")]
impl Default for Condition {
    fn default() -> Self {
        Self {
            cond: sea_orm::Condition::all(),
            page: None,
            order: Vec::new(),
        }
    }
}

#[cfg(feature = "seaorm")]
impl From<sea_orm::Condition> for Condition {
    fn from(cond: sea_orm::Condition) -> Self {
        Self::new(cond)
    }
}

#[cfg(feature = "seaorm")]
impl Condition {
    pub fn new(cond: sea_orm::Condition) -> Self {
        Self {
            cond,
            page: None,
            order: Vec::new(),
        }
    }

    pub fn new_all() -> Self {
        Self::new(Self::all())
    }

    pub fn new_any() -> Self {
        Self::new(Self::any())
    }

    pub fn add_sort(mut self, col: SimpleExpr, sort: Order) -> Self {
        self.order.push((col, sort));
        self
    }

    pub fn add_page(mut self, page: PaginationParam) -> Self {
        self.page = Some(page);
        self
    }

    pub fn parse_sort(mut self, str: String, col: Vec<ColumnRef>) -> Self {
        let mut allow: BTreeMap<String, ColumnRef> = col
            .into_iter()
            .map(|x| (x.column().unwrap().to_string(), x))
            .collect();
        for i in str.split(',') {
            let order = if i.starts_with("-") {
                Order::Desc
            } else {
                Order::Asc
            };
            let name = i
                .strip_prefix('+')
                .or_else(|| i.strip_prefix('-'))
                .unwrap_or(i);
            if let Some(x) = allow.remove(name) {
                self = self.add_sort(SimpleExpr::Column(x), order);
            }
        }
        self
    }

    pub fn any() -> sea_orm::Condition {
        sea_orm::Condition::any()
    }

    pub fn all() -> sea_orm::Condition {
        sea_orm::Condition::all()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<C>(mut self, condition: C) -> Self
    where
        C: Into<sea_orm::Condition>,
    {
        self.cond = self.cond.add(condition.into());
        self
    }

    pub fn add_option<C>(mut self, condition: Option<C>) -> Self
    where
        C: Into<sea_orm::Condition>,
    {
        self.cond = self.cond.add_option(condition);
        self
    }

    pub fn build<E>(self, mut q: Select<E>) -> (Select<E>, Option<PaginationParam>)
    where
        E: EntityTrait,
    {
        for i in self.order {
            q = q.order_by(i.0, i.1);
        }
        (q.filter(self.cond), self.page)
    }

    pub async fn select_page<M, E, C>(self, q: Select<E>, db: &C) -> Result<(Vec<E::Model>, u64)>
    where
        E: EntityTrait<Model = M>,
        M: FromQueryResult + Sized + Send + Sync,
        C: ConnectionTrait,
    {
        let (q, page) = self.build(q);
        q.select_page(db, page).await
    }

    #[cfg(feature = "entity")]
    pub fn add_time<T>(mut self, p: &TimeParam) -> Self
    where
        T: DefaultColumnTrait,
    {
        self = self.add_option(p.created_start.map(|x| T::get_created_at().gte(x)));
        self = self.add_option(p.created_end.map(|x| T::get_created_at().lte(x)));
        self = self.add_option(p.updated_start.map(|x| T::get_updated_at().gte(x)));
        self = self.add_option(p.updated_end.map(|x| T::get_updated_at().lte(x)));
        self
    }
}

/// # Errors
/// Will return `Err` when `x` has duplicate items.
pub fn unique_validator<T: Eq + Hash>(x: &Vec<T>) -> Result<(), ValidationError> {
    if utils::is_unique(x) {
        Ok(())
    } else {
        Err(ValidationError::new("not unique"))
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct IDsReq {
    #[validate(custom(function = "unique_validator"))]
    pub id: Vec<HyUuid>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct OptionIDsReq {
    #[validate(custom(function = "unique_validator"))]
    pub id: Option<Vec<HyUuid>>,
}

#[serde_as]
#[derive(Debug, Deserialize, Validate)]
pub struct TimeParam {
    #[validate(range(min = 0))]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub created_start: Option<i64>,
    #[validate(range(min = 0))]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub created_end: Option<i64>,

    #[validate(range(min = 0))]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub updated_start: Option<i64>,
    #[validate(range(min = 0))]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub updated_end: Option<i64>,
}
