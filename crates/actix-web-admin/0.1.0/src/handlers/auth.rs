use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tera::{Context, Tera};
use crate::resource::{AdminTitle, AdminPrefix};

#[derive(Clone)]
pub struct SimpleAuth {
    pub username: String,
    pub password: String,
}

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
    auth: web::Data<SimpleAuth>,
    form: web::Form<LoginRequest>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
) -> impl Responder {
    if auth.check(&form.username, &form.password) {
        // Usamos un match para evitar el panic si el store de sesión falla
        if let Err(_) = session.insert("admin_user", &form.username) {
            return HttpResponse::InternalServerError().body("Session error");
        }
        return HttpResponse::Found()
            .insert_header(("Location", "/admin/"))
            .finish();
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
        .insert_header(("Location", "/admin/login"))
        .finish()
}
