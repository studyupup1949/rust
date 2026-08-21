use actix_form_data::{Field, Form, Value};
use actix_web::{
    web::{post, resource},
    App, HttpResponse, HttpServer, ResponseError,
};
use futures_util::stream::StreamExt;

#[derive(Debug, thiserror::Error)]
#[error("{inner}")]
struct MyError {
    inner: Box<dyn std::error::Error + 'static>,
}

impl MyError {
    fn new(e: impl std::error::Error + 'static) -> Self {
        Self { inner: Box::new(e) }
    }
}

impl ResponseError for MyError {}

async fn upload(uploaded_content: Value<()>) -> HttpResponse {
    println!("Uploaded Content: {:#?}", uploaded_content);
    HttpResponse::Created().finish()
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    let form = Form::new()
        .field("Hey", Field::text())
        .field(
            "Hi",
            Field::map()
                .field("One", Field::int())
                .field("Two", Field::float())
                .finalize(),
        )
        .field(
            "files",
            Field::array(Field::file(|_, _, mut stream| async move {
                while let Some(res) = stream.next().await {
                    res.map_err(MyError::new)?;
                }
                Ok(()) as Result<(), MyError>
            })),
        );

    println!("{:?}", form);

    HttpServer::new(move || {
        App::new()
            .wrap(form.clone())
            .service(resource("/upload").route(post().to(upload)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
