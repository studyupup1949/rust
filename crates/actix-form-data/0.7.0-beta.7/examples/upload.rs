use actix_form_data::{Error, Field, Form, FormData, Multipart, Value};
use actix_web::{
    http::StatusCode,
    middleware::Logger,
    web::{post, resource, Bytes},
    App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use std::{
    env,
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tracing::info;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct JsonError {
    msg: String,
}

impl<T> From<T> for JsonError
where
    T: std::error::Error,
{
    fn from(e: T) -> Self {
        JsonError {
            msg: format!("{}", e),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, thiserror::Error)]
#[error("Errors occurred")]
struct Errors {
    errors: Vec<JsonError>,
}

impl From<JsonError> for Errors {
    fn from(e: JsonError) -> Self {
        Errors { errors: vec![e] }
    }
}

impl ResponseError for Errors {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(self)
    }
}

struct UploadedContent(Value<PathBuf>);

impl FormData for UploadedContent {
    type Item = PathBuf;
    type Error = Errors;

    fn form(req: &HttpRequest) -> Result<Form<Self::Item, Self::Error>, Self::Error> {
        let file_count = req.app_data::<Arc<AtomicUsize>>().expect("Set config");
        let file_count = Arc::clone(file_count);

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
                Field::array(Field::file(move |_filename, _content_type, stream| {
                    let count = file_count.fetch_add(1, Ordering::Relaxed);
                    async move {
                        save_file(stream, count)
                            .await
                            .map(PathBuf::from)
                            .map_err(Errors::from)
                    }
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
    info!("Uploaded Content: {:#?}", value);

    HttpResponse::Created().finish()
}

async fn save_file(
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>,
    count: usize,
) -> Result<String, JsonError> {
    use futures_lite::io::AsyncWriteExt;

    let mut stream = stream.err_into::<JsonError>();
    let filename = format!("examples/filename{}.png", count);

    let mut file = async_fs::File::create(&filename).await?;
    while let Some(res) = stream.next().await {
        let bytes = res?;
        file.write_all(&bytes).await?;
    }
    file.flush().await?;

    Ok(filename)
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "upload=debug,actix_form_data=debug");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let file_count = Arc::new(AtomicUsize::new(0));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(file_count.clone())
            .service(resource("/upload").route(post().to(upload)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
