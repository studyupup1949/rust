use actix_form_data::{Error, Field, Form, Value};
use actix_web::{
    http::StatusCode,
    middleware::Logger,
    web::{post, resource, Bytes},
    App, HttpResponse, HttpServer, ResponseError,
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

#[derive(Clone, Debug)]
struct AppState {
    form: Form<PathBuf, Errors>,
}

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

async fn upload(uploaded_content: Value<PathBuf>) -> HttpResponse {
    info!("Uploaded Content: {:#?}", uploaded_content);

    HttpResponse::Created().finish()
}

async fn save_file(
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, Error<Errors>>>>>,
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
            Field::array(Field::file(move |_filename, _content_type, stream| {
                let count = file_count.clone().fetch_add(1, Ordering::Relaxed);
                async move {
                    save_file(stream, count)
                        .await
                        .map(PathBuf::from)
                        .map_err(Errors::from)
                }
            })),
        );

    info!("{:#?}", form);

    HttpServer::new(move || {
        App::new()
            .wrap(form.clone())
            .wrap(Logger::default())
            .service(resource("/upload").route(post().to(upload)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
