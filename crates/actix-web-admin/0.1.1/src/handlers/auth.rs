use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tera::{Context, Tera};
use crate::auth::{UserStore, verify_password};
use crate::resource::{AdminTitle, AdminPrefix};

#[derive(Clone)]
#[deprecated(note = "use UserStore trait with a concrete implementation instead")]
pub struct SimpleAuth {
    pub username: String,
    pub password: String,
}

#[allow(deprecated)]
impl SimpleAuth {
    pub fn check(&self, username: &str, password: &str) -> bool {
        self.username == username && self.password == password
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login_page(tmpl: web::Data<Tera>, title: web::Data<AdminTitle>, prefix: web::Data<AdminPrefix>) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("title", &title.0);
    ctx.insert("page_title", "Login");
    ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
    ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
    ctx.insert("resources", &Vec::<crate::resource::ResourceInfo>::new());

    match tmpl.render("login.html", &ctx) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn login(
    session: Session,
    store: web::Data<Arc<dyn UserStore>>,
    form: web::Form<LoginRequest>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
) -> impl Responder {
    let user = if form.username.contains('@') {
        store.find_by_email(&form.username).await
    } else {
        store.find_by_username(&form.username).await
    };

    match user {
        Ok(Some(u)) if verify_password(&form.password, &u.password_hash).unwrap_or(false) => {
            if session.insert("admin_user", &u.username).is_err() {
                return HttpResponse::InternalServerError().body("Session error");
            }
            return HttpResponse::Found()
                .insert_header(("Location", format!("/{}/", prefix.0)))
                .finish();
        }
        Ok(_) | Err(_) => {}
    }

    let mut ctx = Context::new();
    ctx.insert("title", &title.0);
    ctx.insert("page_title", "Login");
    ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
    ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
    ctx.insert("resources", &Vec::<crate::resource::ResourceInfo>::new());
    ctx.insert("error", "Invalid username or password");

    match tmpl.render("login.html", &ctx) {
        Ok(rendered) => HttpResponse::Unauthorized().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn logout(session: Session, prefix: web::Data<AdminPrefix>) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .insert_header(("Location", format!("/{}/login", prefix.0)))
        .finish()
}
