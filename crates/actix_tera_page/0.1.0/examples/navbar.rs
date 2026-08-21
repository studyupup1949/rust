use actix_tera_page::TeraPage;
use actix_web::{
    web::{self, Data},
    App, HttpRequest, HttpServer,
};
use tera::{Context, Tera};

struct State {
    name: String,
}

async fn base_context(req: HttpRequest) -> Context {
    let state = req.app_data::<Data<State>>().unwrap();
    let mut context = Context::new();

    context.insert("username", &state.name);

    context
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(State {
        name: "User Name".to_string(),
    });

    let tera = web::Data::new(match Tera::new("examples/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(tera.clone())
            .wrap(TeraPage::new("pages", base_context))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
