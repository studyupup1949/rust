# actix_session_mongodb

A library for actix-session which implements the SessionStore trait for MongoDB.

NOT AN OFFICIAL LIBRARY FROM actix!

## Support

This library was built for a Project of mine, and as such, is only tested for my specific use-case. I will try to support other use-cases, so please open Issues for errors you encounter, but this is more "best-effort" support than full.

This library is not tested very well, so use at your own risk. You also probably shouldn't use this in your production, but I'm not your Boss so do as you wish.

Minimum Supported Rust Version (MSRV): 1.63

## Example Usage

You can use the MongoSessionStore similarly to the CookieSessionStore or RedisSessionStore, but you'll have to connect and check your database first:

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = mongodb::Client::with_uri_str(&uri).await.expect("failed to connect");
    let db = client.database("database_name");
    let mongo_session_store = actix_session_mongodb::connect_and_init(&db, "Sessions").await?;

    HttpServer::new(move || {
        App::new()
            .app_data(...)
            // Install the identity framework first.
            .wrap(
                IdentityMiddleware::builder()
                    .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
                    .build(),
            )
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            .wrap(
                SessionMiddleware::builder(mongo_session_store.clone(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(Duration::days(2))
                            .session_ttl_extension_policy(
                                actix_session::config::TtlExtensionPolicy::OnEveryRequest,
                            ),
                    )
                    .build(),
            )
            .service(...)
    })
    // Use OS's keep-alive (usually quite long)
    .keep_alive(actix_web::http::KeepAlive::Os)
    .bind(bind_addr)?
    .run()
    .await
}
```
