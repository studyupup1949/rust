# Actix-route-config
Allow clean configuration of [actix-web](https://crates.io/crates/actix-web) routes.

## Explanation
I like the concept of 'routers' to configure my Actix web servers.
This trait specifies a function `configure` which can be called directly from Actix's [configure](https://docs.rs/actix-web/latest/actix_web/struct.App.html#method.configure) method.

## Example
```rust
use actix_route_config::Routable;
use actix_web::web;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/api")
            // Assuming there's a submodule `foo` with a handler function `foo`
            .route("/foo", web::get().to(foo:foo))
        );
    }
}
```
This will create a route `/api/foo` ending up at the `foo::foo` function.

## License
This project is licensed under:
- [MIT](LICENSE-MIT)
- [Apache 2.0](LICENSE-APACHE)

At your option.