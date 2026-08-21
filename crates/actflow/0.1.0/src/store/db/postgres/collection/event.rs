use std::{sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use flume::{Receiver, Sender};
use sea_query::{
    Alias as SeaAlias, ColumnDef, Expr as SeaExpr, Func as SeaFunc, Iden, Index, Order as SeaOrder, PostgresQueryBuilder, Query as SeaQuery, Table,
};
use sea_query_binder::SqlxBinder;
use sqlx::{Error as DbError, Row, postgres::PgRow};
use tokio::{runtime::Runtime, time::MissedTickBehavior};

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
pub struct EventCollection {
    conn: DbConnection,

    batch_tx: Sender<data::Event>,
    batch_rx: Receiver<data::Event>,

    runtime: Arc<Runtime>,
}

#[derive(Iden)]
#[iden = "events"]
enum CollectionIden {
    Table,

    Id,
    Pid,
    Nid,
    Name,
    Message,
    Timestamp,
}

impl DbCollection for EventCollection {
    type Item = data::Event;

    fn exists(
        &self,
        _: &str,
    ) -> Result<bool> {
        unreachable!()
    }

    fn find(
        &self,
        _: &str,
    ) -> Result<Self::Item> {
        unreachable!()
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
            .columns([CollectionIden::Id, CollectionIden::Pid, CollectionIden::Nid, CollectionIden::Name, CollectionIden::Message, CollectionIden::Timestamp])
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
        self.batch_tx.send(data.clone()).map_err(map_db_err)?;
        Ok(true)
    }

    fn update(
        &self,
        data: &Self::Item,
    ) -> Result<bool> {
        let model = data.clone();
        let (sql, sql_values) = SeaQuery::update()
            .table(CollectionIden::Table)
            .values([
                (CollectionIden::Pid, model.pid.into()),
                (CollectionIden::Nid, model.nid.into()),
                (CollectionIden::Name, model.name.into()),
                (CollectionIden::Message, model.message.into()),
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

impl DbRow for data::Event {
    fn id(&self) -> &str {
        &self.id
    }

    fn from_row(row: &PgRow) -> std::result::Result<Self, DbError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get("id"),
            pid: row.get("pid"),
            nid: row.get("nid"),
            name: row.get("name"),
            message: row.get("message"),
            timestamp: row.get("timestamp"),
        })
    }
}

impl DbInit for EventCollection {
    fn init(&self) {
        let sql = [
            Table::create()
                .table(CollectionIden::Table)
                .if_not_exists()
                .col(ColumnDef::new(CollectionIden::Id).string().not_null().primary_key())
                .col(ColumnDef::new(CollectionIden::Pid).string().not_null())
                .col(ColumnDef::new(CollectionIden::Nid).string().default(""))
                .col(ColumnDef::new(CollectionIden::Name).string().default(""))
                .col(ColumnDef::new(CollectionIden::Message).text().default(""))
                .col(ColumnDef::new(CollectionIden::Timestamp).big_integer().default(0))
                .build(PostgresQueryBuilder),
            Index::create().name("idx_logs_pid").if_not_exists().table(CollectionIden::Table).col(CollectionIden::Pid).build(PostgresQueryBuilder),
            Index::create().name("idx_logs_nid").if_not_exists().table(CollectionIden::Table).col(CollectionIden::Nid).build(PostgresQueryBuilder),
        ];
        self.conn.batch_execute(&sql).unwrap();
    }
}

impl EventCollection {
    /// Batch size for inserting log records
    const BATCH_SIZE: usize = 1000;
    /// Timeout for flushing the batch
    const TIMEOUT: Duration = Duration::from_secs(3);

    pub fn new(conn: &DbConnection, runtime: Arc<Runtime>) -> Self {
        let (batch_tx, batch_rx) = flume::bounded(1024);
        let log_collection = Self {
            conn: conn.clone(),
            batch_tx,
            batch_rx,
            runtime,
        };
        log_collection.watch_batch();
        log_collection
    }

    fn watch_batch(&self) {
        let rx = self.batch_rx.clone();
        let conn = self.conn.clone();
        self.runtime.spawn(async move {
            let mut batch = Vec::with_capacity(Self::BATCH_SIZE);
            let mut interval = tokio::time::interval(Self::TIMEOUT);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut latest_inerst = 0;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        if now - latest_inerst >= Self::TIMEOUT.as_secs() && batch.len() > 0 {
                            let items = std::mem::take(&mut batch);
                            Self::batch_events(conn.clone(), items);

                            latest_inerst = now;
                        }
                    }
                    Ok(event) = rx.recv_async() => {
                        if batch.len() >= Self::BATCH_SIZE {
                            let items = std::mem::take(&mut batch);
                            Self::batch_events(conn.clone(), items);

                            latest_inerst = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        }

                        batch.push(event);
                    }
                }
            }
        });
    }

    fn batch_events(
        conn: DbConnection,
        items: Vec<data::Event>,
    ) {
        let mut binding = SeaQuery::insert();
        let stmt = binding.into_table(CollectionIden::Table).columns([
            CollectionIden::Id,
            CollectionIden::Pid,
            CollectionIden::Nid,
            CollectionIden::Name,
            CollectionIden::Message,
            CollectionIden::Timestamp,
        ]);

        for item in items {
            if let Err(e) = stmt.values(vec![
                item.id.into(),
                item.pid.into(),
                item.nid.into(),
                item.name.into(),
                item.message.into(),
                item.timestamp.into(),
            ]) {
                eprintln!("Failed to stmt log record: {:?}", e);
            }
        }

        let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

        if let Err(e) = conn.execute(sql.as_str(), values) {
            eprintln!("Failed to insert log records: {:?}", e);
        }
    }
}
