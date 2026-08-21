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
pub struct ProcCollection {
    conn: DbConnection,
}

#[derive(Iden)]
#[iden = "procs"]
enum CollectionIden {
    Table,

    Id,
    Wid,
    State,
    StartTime,
    EndTime,
    Err,
    Timestamp,
}

impl DbCollection for ProcCollection {
    type Item = data::Proc;

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
                CollectionIden::Wid,
                CollectionIden::State,
                CollectionIden::StartTime,
                CollectionIden::EndTime,
                CollectionIden::Err,
                CollectionIden::Timestamp,
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
        count_query.from(CollectionIden::Table).expr(SeaFunc::count(SeaExpr::col(SeaAlias::new("id"))));

        let mut query = SeaQuery::select();
        query
            .columns([
                CollectionIden::Id,
                CollectionIden::Wid,
                CollectionIden::State,
                CollectionIden::StartTime,
                CollectionIden::EndTime,
                CollectionIden::Err,
                CollectionIden::Timestamp,
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
                CollectionIden::Wid,
                CollectionIden::State,
                CollectionIden::StartTime,
                CollectionIden::EndTime,
                CollectionIden::Err,
                CollectionIden::Timestamp,
            ])
            .values([data.id.into(), data.wid.into(), data.state.into(), data.start_time.into(), data.end_time.into(), data.err.into(), data.timestamp.into()])
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
                (CollectionIden::State, model.state.into()),
                (CollectionIden::Wid, model.wid.into()),
                (CollectionIden::StartTime, model.start_time.into()),
                (CollectionIden::EndTime, model.end_time.into()),
                (CollectionIden::Err, model.err.into()),
                (CollectionIden::Timestamp, model.timestamp.into()),
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

impl DbRow for data::Proc {
    fn id(&self) -> &str {
        &self.id
    }

    fn from_row(row: &PgRow) -> std::result::Result<Self, DbError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get("id"),
            wid: row.get("wid"),
            state: row.get("state"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            err: row.get("err"),
            timestamp: row.get("timestamp"),
        })
    }
}

impl DbInit for ProcCollection {
    fn init(&self) {
        let sql = [
            Table::create()
                .table(CollectionIden::Table)
                .if_not_exists()
                .col(ColumnDef::new(CollectionIden::Id).string().not_null().primary_key())
                .col(ColumnDef::new(CollectionIden::State).string().not_null())
                .col(ColumnDef::new(CollectionIden::Wid).string().not_null())
                .col(ColumnDef::new(CollectionIden::StartTime).big_integer().default(0))
                .col(ColumnDef::new(CollectionIden::EndTime).big_integer().default(0))
                .col(ColumnDef::new(CollectionIden::Timestamp).big_integer().default(0))
                .col(ColumnDef::new(CollectionIden::Err).string())
                .build(PostgresQueryBuilder),
            Index::create().name("idx_procs_state").if_not_exists().table(CollectionIden::Table).col(CollectionIden::State).build(PostgresQueryBuilder),
            Index::create().name("idx_procs_wid").if_not_exists().table(CollectionIden::Table).col(CollectionIden::Wid).build(PostgresQueryBuilder),
        ];
        self.conn.batch_execute(&sql).unwrap();
    }
}

impl ProcCollection {
    pub fn new(conn: &DbConnection) -> Self {
        Self {
            conn: conn.clone(),
        }
    }
}
