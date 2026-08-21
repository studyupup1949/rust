// adminx-seaorm/src/lib.rs
//
// SeaORM storage backend for adminx-core. Implements `Storage` against a single
// `DatabaseConnection` that transparently backs PostgreSQL or MySQL (selected
// by the connection URL scheme).

mod query;

use adminx_core::storage::{
    set_storage, CreateOutcome, ListPage, QueryOptions, Storage, StorageError,
};
use async_trait::async_trait;
use sea_orm::sea_query::{Alias, Expr, Query};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, ExecResult,
    FromQueryResult, JsonValue, StatementBuilder,
};
use serde_json::{Map, Value};

use query::{
    build_count_select, build_find_select, build_get_select, build_list_select, id_to_sea_value,
    json_to_sea_value, value_expr,
};

/// SeaORM-backed storage. Clone-cheap (the pool is `Arc` internally).
pub struct SeaOrmStorage {
    conn: DatabaseConnection,
}

impl SeaOrmStorage {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    fn backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    /// Run a raw SQL statement (schema DDL, seeding). Handy for demos/tests that
    /// need to create tables before serving.
    pub async fn execute_sql(&self, sql: &str) -> Result<(), sea_orm::DbErr> {
        let backend = self.backend();
        self.conn
            .execute(sea_orm::Statement::from_string(backend, sql.to_owned()))
            .await?;
        Ok(())
    }
}

/// Open a connection pool from a database URL and return a ready storage.
/// e.g. `postgres://user:pass@host/db` or `mysql://user:pass@host/db`.
pub async fn connect(database_url: &str) -> Result<SeaOrmStorage, sea_orm::DbErr> {
    let mut opts = ConnectOptions::new(database_url.to_owned());
    opts.sqlx_logging(false);
    let conn = Database::connect(opts).await?;
    tracing::info!(
        "✅ adminx-seaorm connected ({:?})",
        conn.get_database_backend()
    );
    Ok(SeaOrmStorage::new(conn))
}

/// Convenience: connect and register as the global adminx storage backend.
pub async fn init(database_url: &str) -> Result<(), sea_orm::DbErr> {
    let storage = connect(database_url).await?;
    set_storage(Box::new(storage));
    Ok(())
}

/// Connect and run a batch of **SQL** statements (seeding / migrations), in
/// order, returning the total rows affected. Self-contained — no `set_storage`
/// needed. Used by `adminx seed` and by app startup code.
pub async fn seed(database_url: &str, statements: &[&str]) -> Result<u64, StorageError> {
    let store = connect(database_url)
        .await
        .map_err(|e| StorageError::Backend(e.to_string()))?;
    let mut total = 0u64;
    for stmt in statements {
        total += store.execute_raw(stmt).await?;
    }
    Ok(total)
}

fn last_insert_id(backend: DbBackend, res: &ExecResult) -> Option<String> {
    match backend {
        DbBackend::MySql | DbBackend::Sqlite => Some(res.last_insert_id().to_string()),
        _ => None,
    }
}

fn map_err(e: sea_orm::DbErr) -> StorageError {
    StorageError::Backend(e.to_string())
}

#[async_trait]
impl Storage for SeaOrmStorage {
    async fn list(&self, table: &str, opts: &QueryOptions) -> Result<ListPage, StorageError> {
        let backend = self.backend();

        // Read the aggregate as a scalar. `JsonValue::find_by_statement` drops
        // unmapped aggregate columns (returns `{}`) on some backends, so go
        // through the raw `QueryResult` and pull the `count` column as an i64.
        let count_stmt =
            StatementBuilder::build(&build_count_select(table, &opts.filters), &backend);
        let total: u64 = match self.conn.query_one(count_stmt).await.map_err(map_err)? {
            Some(qr) => qr.try_get::<i64>("", "count").map_err(map_err)?.max(0) as u64,
            None => 0,
        };

        let select = build_list_select(
            table,
            opts.per_page,
            opts.offset(),
            &opts.sort_by,
            opts.sort_desc,
            &opts.filters,
        );
        let stmt = StatementBuilder::build(&select, &backend);
        let rows = JsonValue::find_by_statement(stmt)
            .all(&self.conn)
            .await
            .map_err(map_err)?;

        Ok(ListPage { rows, total })
    }

    async fn get(&self, table: &str, pk: &str, id: &str) -> Result<Option<Value>, StorageError> {
        let backend = self.backend();
        let stmt = StatementBuilder::build(&build_get_select(table, pk, id), &backend);
        JsonValue::find_by_statement(stmt)
            .one(&self.conn)
            .await
            .map_err(map_err)
    }

    async fn find_one_by(
        &self,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<Option<Value>, StorageError> {
        let backend = self.backend();
        let stmt = StatementBuilder::build(&build_find_select(table, column, value), &backend);
        JsonValue::find_by_statement(stmt)
            .one(&self.conn)
            .await
            .map_err(map_err)
    }

    async fn create(
        &self,
        table: &str,
        data: Map<String, Value>,
    ) -> Result<CreateOutcome, StorageError> {
        let mut insert = Query::insert();
        insert.into_table(Alias::new(table));
        insert.columns(data.keys().map(Alias::new));
        let values = data.values().map(|v| value_expr(json_to_sea_value(v)));
        insert
            .values(values)
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        let backend = self.backend();
        let stmt = StatementBuilder::build(&insert, &backend);
        let res = self.conn.execute(stmt).await.map_err(map_err)?;

        Ok(CreateOutcome {
            last_insert_id: last_insert_id(backend, &res),
        })
    }

    async fn update(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        data: Map<String, Value>,
    ) -> Result<u64, StorageError> {
        let mut update = Query::update();
        update.table(Alias::new(table));
        for (col, val) in &data {
            update.value(Alias::new(col), json_to_sea_value(val));
        }
        update.and_where(Expr::col(Alias::new(pk)).eq(id_to_sea_value(id)));

        let backend = self.backend();
        let stmt = StatementBuilder::build(&update, &backend);
        let res = self.conn.execute(stmt).await.map_err(map_err)?;
        Ok(res.rows_affected())
    }

    async fn delete(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        soft: bool,
    ) -> Result<u64, StorageError> {
        let backend = self.backend();
        let stmt = if soft {
            let mut update = Query::update();
            update
                .table(Alias::new(table))
                .value(Alias::new("deleted"), true)
                .and_where(Expr::col(Alias::new(pk)).eq(id_to_sea_value(id)));
            StatementBuilder::build(&update, &backend)
        } else {
            let mut delete = Query::delete();
            delete
                .from_table(Alias::new(table))
                .and_where(Expr::col(Alias::new(pk)).eq(id_to_sea_value(id)));
            StatementBuilder::build(&delete, &backend)
        };

        let res = self.conn.execute(stmt).await.map_err(map_err)?;
        Ok(res.rows_affected())
    }

    async fn execute_raw(&self, statement: &str) -> Result<u64, StorageError> {
        let backend = self.backend();
        let res = self
            .conn
            .execute(sea_orm::Statement::from_string(backend, statement.to_owned()))
            .await
            .map_err(map_err)?;
        Ok(res.rows_affected())
    }

    async fn health(&self) -> bool {
        self.conn.ping().await.is_ok()
    }
}
