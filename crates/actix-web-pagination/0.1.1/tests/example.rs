#[macro_use]
extern crate serde;

#[macro_use]
extern crate actix_web;

use actix_web::{test, App, HttpResponse};
use actix_web_pagination::Pagination;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct R {
    page: u32,
    per_page: u32,
    offset: u32,
}

impl R {
    fn new(page: u32, per_page: u32) -> Self {
        Self {
            page,
            per_page,
            offset: page * per_page,
        }
    }
}

#[get("/")]
async fn list(page: Pagination) -> HttpResponse {
    HttpResponse::Ok().json(R {
        page: page.page(),
        per_page: page.per_page(),
        offset: page.offset(),
    })
}

#[actix_rt::test]
async fn it_works_with_page() -> actix_web::Result<()> {
    let t = test::start(|| App::new().service(list));

    let mut res = t.get("/").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?page=-1").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?page=0").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?page=1").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?page=2").send().await?;
    assert_eq!(R::new(1, 20), res.json().await?);

    let mut res = t.get("/?page=10").send().await?;
    assert_eq!(R::new(9, 20), res.json().await?);

    let mut res = t.get("/?page=ok").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    Ok(())
}

#[actix_rt::test]
async fn it_works_with_per_page() -> actix_web::Result<()> {
    let t = test::start(|| App::new().service(list));

    let mut res = t.get("/").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?per_page=10").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?per_page=20").send().await?;
    assert_eq!(R::new(0, 20), res.json().await?);

    let mut res = t.get("/?per_page=100").send().await?;
    assert_eq!(R::new(0, 100), res.json().await?);

    let mut res = t.get("/?per_page=101").send().await?;
    assert_eq!(R::new(0, 100), res.json().await?);

    Ok(())
}

#[actix_rt::test]
async fn it_works_with_config() -> actix_web::Result<()> {
    let t = test::start(|| {
        App::new().service(list).data(
            Pagination::config()
                .page_name("page1")
                .per_page_name("limit")
                .default_per_page(50),
        )
    });

    let mut res = t.get("/").send().await?;
    assert_eq!(R::new(0, 50), res.json().await?);

    let mut res = t.get("/?page=10").send().await?;
    assert_eq!(R::new(0, 50), res.json().await?);

    let mut res = t.get("/?page1=10").send().await?;
    assert_eq!(R::new(9, 50), res.json().await?);

    let mut res = t.get("/?per_page=60").send().await?;
    assert_eq!(R::new(0, 50), res.json().await?);

    let mut res = t.get("/?limit=60").send().await?;
    assert_eq!(R::new(0, 60), res.json().await?);

    Ok(())
}
