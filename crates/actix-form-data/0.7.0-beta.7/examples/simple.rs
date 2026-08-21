use actix_form_data::{Error, Field, Form, FormData, Multipart, Value};
use actix_web::{
    web::{post, resource},
    App, HttpRequest, HttpResponse, HttpServer,
};
use futures_util::stream::StreamExt;

struct UploadedContent(Value<()>);

impl FormData for UploadedContent {
    type Item = ();
    type Error = Error;

    fn form(_: &HttpRequest) -> Result<Form<Self::Item, Self::Error>, Self::Error> {
        Ok(Form::new()
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
                        res?;
                    }
                    Ok(()) as Result<(), Error>
                })),
            ))
    }

    fn extract(value: Value<Self::Item>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(UploadedContent(value))
    }
}

async fn upload(Multipart(UploadedContent(value)): Multipart<UploadedContent>) -> HttpResponse {
    println!("Uploaded Content: {:#?}", value);
    HttpResponse::Created().finish()
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    HttpServer::new(move || App::new().service(resource("/upload").route(post().to(upload))))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
