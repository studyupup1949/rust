#[macro_export]
macro_rules! db_test {
    ($mod:ident) => {
        $crate::db_test!($mod, run_tests);
    };

    ($mod:ident, $fn_name:ident) => {
        ::paste::paste! {
            #[tokio::test]
            async fn [<test_ $mod>]() -> ::activityforge::Result<()> {
                $crate::log::init_logger();

                let config = $crate::db::config::test_config();

                $crate::db::container::start_db(&config)?;
                $crate::db::container::wait_for_db(&config).await?;

                let db = $crate::db::connect::test_connection(&config).await?;
                $crate::db::migration::test_migration(&db).await?;

                $fn_name(&db)
                    .await
                    .and_then(|_| $crate::db::container::stop_db())
                    .or_else(|err| $crate::db::container::stop_db().and_then(|_| Err(err)))
            }
        }
    };

    ($mod:ident => $fn_name:ident ($db:ident) $test_fn:tt) => {
        $crate::db_test!($mod, $fn_name);

        async fn $fn_name($db: &::activityforge::db::Db) -> ::activityforge::Result<()> {
            $test_fn
        }
    };
}

#[macro_export]
macro_rules! router_test {
    ($mod:ident) => {
        $crate::router_test!($mod, run_tests);
    };

    ($mod:ident, $fn_name:ident) => {
        ::paste::paste! {
            #[tokio::test]
            async fn [<test_ $mod>]() -> ::activityforge::Result<()> {
                $crate::log::init_logger();

                let db_host = std::env::var("POSTGRES_HOST").unwrap_or("127.0.0.1".to_string());
                let username = std::env::var("POSTGRES_USER").unwrap_or("activityforge_test".to_string());
                let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("activityforge_test".to_string());
                let db_name = std::env::var("POSTGRES_DB_NAME").unwrap_or("activityforge_test".to_string());
                let port: u16 = std::env::var("POSTGRES_DB_PORT")
                    .unwrap_or("5432".to_string())
                    .parse::<u16>()
                    .unwrap_or(5432);

                let config = ::activityforge::db::DbConfig::new()
                    .with_username(username)
                    .with_password(password)
                    .with_host(db_host)
                    .with_port(port)
                    .with_db_name(db_name);

                $crate::db::container::start_db(&config)?;
                $crate::db::container::wait_for_db(&config).await?;

                let db = $crate::db::connect::test_connection(&config).await?;
                $crate::db::migration::test_migration(&db).await?;

                let app_port = $crate::router::test_server_port();
                let app_host = format!("127.0.0.1:{app_port}");

                let app = ::activityforge::app::App::create(
                    config,
                    ::activityforge::db::Iri::try_from(format!("http://{app_host}")).map(|i| i.into())?,
                    ::activityforge::db::Name::try_from(stringify!($mod)).map(|n| n.into())?,
                )
                .await
                .map_err(|err| {
                    log::error!("error creating app server: {err}");
                    err
                })?;

                $crate::router::mock_server().await?;

                let server = app.router().await.unwrap();
                let listener = ::tokio::net::TcpListener::bind(&app_host).await.unwrap();

                ::tokio::spawn(async move {
                    ::axum::serve(listener, server).await.ok();
                });

                $fn_name(&db, &app)
                    .await
                    .and_then(|_| $crate::db::container::stop_db())
                    .or_else(|err| $crate::db::container::stop_db().and_then(|_| Err(err)))
            }
        }
    };

    ($mod:ident => $fn_name:ident ($db:ident, $app:ident) $test_fn:tt) => {
        $crate::router_test!($mod, $fn_name);

        #[allow(unused)]
        async fn $fn_name($db: &::activityforge::db::Db, $app: &::activityforge::app::App) -> ::activityforge::Result<()> {
            $test_fn
        }
    };
}
