use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use activityforge::{Error, Result, db::DbConfig};

mod collaborator;
mod connect;
mod container;
mod factory;
mod follower;
mod inbox;
mod key;
mod migration;
mod outbox;
mod person;
mod repository;
mod team;

#[tokio::test]
async fn db_tests() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init()
        .ok();

    let username = std::env::var("POSTGRES_USER").unwrap_or("activityforge_test".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("activityforge_test".to_string());
    let db_name = std::env::var("POSTGRES_DB_NAME").unwrap_or("activityforge_test".to_string());
    let port: u16 = 5432;

    let config = DbConfig::new()
        .with_username(username)
        .with_password(password)
        .with_host("127.0.0.1")
        .with_port(port)
        .with_db_name(db_name);

    container::start_db(&config)?;

    if let Err(err) = run_tests(&config).await {
        log::error!("error running database tests: {err}");
        container::stop_db()?;
        Err(err)
    } else {
        container::stop_db()
    }
}

async fn run_tests(config: &DbConfig) -> Result<()> {
    // get a connection pool to the test database
    let db = connect::test_connection(config).await.map(Arc::new)?;

    // perform database migrations to create tables + load any fixtures
    migration::test_migration(&db).await?;

    let is_err = Arc::new(AtomicBool::new(false));

    let inbox_is_err = is_err.clone();
    let outbox_is_err = is_err.clone();
    let key_is_err = is_err.clone();
    let collaborator_is_err = is_err.clone();
    let person_is_err = is_err.clone();
    let follower_is_err = is_err.clone();
    let team_is_err = is_err.clone();
    let factory_is_err = is_err.clone();
    let repository_is_err = is_err.clone();

    let inbox_db = db.clone();
    let outbox_db = db.clone();
    let key_db = db.clone();
    let collaborator_db = db.clone();
    let person_db = db.clone();
    let follower_db = db.clone();
    let team_db = db.clone();
    let factory_db = db.clone();
    let repository_db = db.clone();

    let (inbox, outbox, key, collaborator, person, follower, team, factory, repository) = tokio::join!(
        // test Inbox record operations
        tokio::spawn(async move {
            if let Err(err) = inbox::test_inbox(&inbox_db).await {
                log::error!("tests: inbox: {err}");
                inbox_is_err.store(true, Ordering::Release);
            }
        }),
        // test Outbox record operations
        tokio::spawn(async move {
            if let Err(err) = outbox::test_outbox(&outbox_db).await {
                log::error!("tests: outbox: {err}");
                outbox_is_err.store(true, Ordering::Release);
            }
        }),
        // test Key record operations
        tokio::spawn(async move {
            if let Err(err) = key::test_key(&key_db).await {
                log::error!("tests: key: {err}");
                key_is_err.store(true, Ordering::Release);
            }
        }),
        // test Collaborator record operations
        tokio::spawn(async move {
            if let Err(err) = collaborator::test_collaborator(&collaborator_db).await {
                log::error!("tests: collaborator: {err}");
                collaborator_is_err.store(true, Ordering::Release);
            }
        }),
        // test Person record operations
        tokio::spawn(async move {
            if let Err(err) = person::test_person(&person_db).await {
                log::error!("tests: person: {err}");
                person_is_err.store(true, Ordering::Release);
            }
        }),
        // test Follower record operations
        tokio::spawn(async move {
            if let Err(err) = follower::test_follower(&follower_db).await {
                log::error!("tests: follower: {err}");
                follower_is_err.store(true, Ordering::Release);
            }
        }),
        // test Team record operations
        tokio::spawn(async move {
            if let Err(err) = team::test_team(&team_db).await {
                log::error!("tests: team: {err}");
                team_is_err.store(true, Ordering::Release);
            }
        }),
        // test Factory record operations
        tokio::spawn(async move {
            if let Err(err) = factory::test_factory(&factory_db).await {
                log::error!("tests: factory: {err}");
                factory_is_err.store(true, Ordering::Release);
            }
        }),
        // test Repository record operations
        tokio::spawn(async move {
            if let Err(err) = repository::test_repository(&repository_db).await {
                log::error!("tests: repository: {err}");
                repository_is_err.store(true, Ordering::Release);
            }
        }),
    );

    inbox?;
    outbox?;
    key?;
    collaborator?;
    person?;
    follower?;
    team?;
    factory?;
    repository?;

    if is_err.load(Ordering::Acquire) {
        Err(Error::sql("db tests failed"))
    } else {
        Ok(())
    }
}
