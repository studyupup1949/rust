use sea_query::{
    Alias as SeaAlias, ColumnDef, Expr as SeaExpr, Func as SeaFunc, Iden, Index, Order as SeaOrder, PostgresQueryBuilder, Query as SeaQuery, Table,
};
use sea_query_binder::SqlxBinder;
use sqlx::{Error as DbError, Row, postgres::PgRow};

use crate::{
    Result,
    store::{
        DbCollection, PageData, data,
        db::postgres::{DbInit, DbRow},
        query,
    },
};

use super::{DbConnection, into_query, map_db_err};

#[derive(Debug)]
pub struct WorkflowCollection {
    conn: DbConnection,
}

#[derive(Iden)]
#[iden = "workflows"]
enum CollectionIden {
    Table,

    Id,
    Name,
    Desc,
    Data,
    CreateTime,
    UpdateTime,
}

impl DbCollection for WorkflowCollection {
    type Item = data::Workflow;

    fn exists(
        &self,
        id: &str,
    ) -> Result<bool> {
        let (sql, values) = SeaQuery::select()
            .from(CollectionIden::Table)
            .expr(SeaFunc::count(SeaExpr::col(CollectionIden::Id)))
            .and_where(SeaExpr::col(CollectionIden::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let count = self.conn.query_one(sql.as_str(), values).map(|row| row.get::<i64, usize>(0)).map_err(map_db_err)?;

        Ok(count > 0)
    }

    fn find(
        &self,
        id: &str,
    ) -> Result<Self::Item> {
        let (sql, values) = SeaQuery::select()
            .from(CollectionIden::Table)
            .columns([
                CollectionIden::Id,
                CollectionIden::Name,
                CollectionIden::Desc,
                CollectionIden::Data,
                CollectionIden::CreateTime,
                CollectionIden::UpdateTime,
            ])
            .and_where(SeaExpr::col(CollectionIden::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);
        self.conn.query_one(&sql, values).map(|row| Self::Item::from_row(&row).map_err(map_db_err)).map_err(map_db_err)?
    }

    fn query(
        &self,
        q: &query::Query,
    ) -> Result<PageData<Self::Item>> {
        let filter = into_query(q);

        let mut count_query = SeaQuery::select();
        count_query.from(CollectionIden::Table).expr(SeaFunc::count(SeaExpr::col(CollectionIden::Id)));

        let mut query = SeaQuery::select();
        query
            .columns([
                CollectionIden::Id,
                CollectionIden::Name,
                CollectionIden::Desc,
                CollectionIden::Data,
                CollectionIden::CreateTime,
                CollectionIden::UpdateTime,
            ])
            .from(CollectionIden::Table);

        if !filter.is_empty() {
            count_query.cond_where(filter.clone());
            query.cond_where(filter);
        }

        if !q.order_by().is_empty() {
            for (order, rev) in q.order_by().iter() {
                query.order_by(
                    SeaAlias::new(order),
                    if *rev {
                        SeaOrder::Desc
                    } else {
                        SeaOrder::Asc
                    },
                );
            }
        }
        let (sql, values) = query.limit(q.limit() as u64).offset(q.offset() as u64).build_sqlx(PostgresQueryBuilder);

        let (count_sql, count_values) = count_query.build_sqlx(PostgresQueryBuilder);
        let count = self.conn.query_one(count_sql.as_str(), count_values).map_err(map_db_err)?.get::<i64, usize>(0) as usize;
        let page_count = count.div_ceil(q.limit());
        let page_num = q.offset() / q.limit() + 1;
        let data = PageData {
            count,
            page_size: q.limit(),
            page_num,
            page_count,
            rows: self.conn.query(&sql, values).map_err(map_db_err)?.iter().map(|row| Self::Item::from_row(row).unwrap()).collect::<Vec<_>>(),
        };
        Ok(data)
    }

    fn create(
        &self,
        data: &Self::Item,
    ) -> Result<bool> {
        let data = data.clone();
        let (sql, sql_values) = SeaQuery::insert()
            .into_table(CollectionIden::Table)
            .columns([
                CollectionIden::Id,
                CollectionIden::Name,
                CollectionIden::Desc,
                CollectionIden::Data,
                CollectionIden::CreateTime,
                CollectionIden::UpdateTime,
            ])
            .values([data.id.into(), data.name.into(), data.desc.into(), data.data.into(), data.create_time.into(), data.update_time.into()])
            .map_err(map_db_err)?
            .build_sqlx(PostgresQueryBuilder);

        let result = self.conn.execute(sql.as_str(), sql_values).map_err(map_db_err)?;
        Ok(result.rows_affected() > 0)
    }

    fn update(
        &self,
        data: &Self::Item,
    ) -> Result<bool> {
        let model = data.clone();
        let (sql, sql_values) = SeaQuery::update()
            .table(CollectionIden::Table)
            .values([
                (CollectionIden::Name, model.name.into()),
                (CollectionIden::Desc, model.desc.into()),
                (CollectionIden::Data, model.data.into()),
                (CollectionIden::CreateTime, model.create_time.into()),
                (CollectionIden::UpdateTime, model.update_time.into()),
            ])
            .and_where(SeaExpr::col(CollectionIden::Id).eq(data.id()))
            .build_sqlx(PostgresQueryBuilder);

        let result = self.conn.execute(sql.as_str(), sql_values).map_err(map_db_err)?;
        Ok(result.rows_affected() > 0)
    }

    fn delete(
        &self,
        id: &str,
    ) -> Result<bool> {
        let (sql, values) =
            SeaQuery::delete().from_table(CollectionIden::Table).and_where(SeaExpr::col(CollectionIden::Id).eq(id)).build_sqlx(PostgresQueryBuilder);

        let result = self.conn.execute(sql.as_str(), values).map_err(map_db_err)?;
        Ok(result.rows_affected() > 0)
    }
}

impl DbRow for data::Workflow {
    fn id(&self) -> &str {
        &self.id
    }

    fn from_row(row: &PgRow) -> std::result::Result<Self, DbError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get("id"),
            name: row.get("name"),
            desc: row.get("desc"),
            data: row.get("data"),
            create_time: row.get("create_time"),
            update_time: row.get("update_time"),
        })
    }
}

impl DbInit for WorkflowCollection {
    fn init(&self) {
        let sql = [
            Table::create()
                .table(CollectionIden::Table)
                .if_not_exists()
                .col(ColumnDef::new(CollectionIden::Id).string().not_null().primary_key())
                .col(ColumnDef::new(CollectionIden::Name).string())
                .col(ColumnDef::new(CollectionIden::Desc).string())
                .col(ColumnDef::new(CollectionIden::Data).string().not_null())
                .col(ColumnDef::new(CollectionIden::CreateTime).big_integer().default(0))
                .col(ColumnDef::new(CollectionIden::UpdateTime).big_integer().default(0))
                .build(PostgresQueryBuilder),
            Index::create().name("idx_workflows_name").if_not_exists().table(CollectionIden::Table).col(CollectionIden::Name).build(PostgresQueryBuilder),
        ];

        self.conn.batch_execute(&sql).unwrap();
    }
}

impl WorkflowCollection {
    pub fn new(conn: &DbConnection) -> Self {
        Self {
            conn: conn.clone(),
        }
    }
}
