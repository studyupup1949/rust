use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;
use tera::{Context, Tera};
use crate::registry::SharedRegistry;
use crate::types::*;
use crate::resource::{AdminTitle, AdminPrefix};
use log::{info, error, warn};

fn get_context(
    session: &Session,
    title: &AdminTitle,
    _resource: &dyn crate::resource::AdminResource,
    slug: &str,
) -> Context {
    let mut ctx = Context::new();
    ctx.insert("title", &title.0);
    let user = session.get::<String>("admin_user").ok().flatten().unwrap_or_else(|| "Guest".to_string());
    ctx.insert("user", &user);
    ctx.insert("path_dashboard", "/admin/"); 
    ctx.insert("path_logout", "/admin/logout"); 
    ctx.insert("current_slug", slug);
    ctx
}

pub async fn list(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<String>,
    query: web::Query<ListQuery>,
) -> impl Responder {
    let slug = path.into_inner();
    info!("Accessing list for slug: {}", slug);
    
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => {
            info!("User authenticated: {}", u);
            u
        },
        _ => {
            warn!("User not authenticated, redirecting to login");
            return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish();
        }
    };

    let resource = match registry.get(&slug) {
        Some(r) => {
            info!("Resource found: {}", r.name());
            r
        },
        None => {
            error!("Resource not found for slug: {}", slug);
            return HttpResponse::NotFound().finish();
        }
    };

    let result = match resource.list(query.into_inner()).await {
        Ok(res) => {
            info!("List query successful, {} items", res.rows.len());
            res
        },
        Err(e) => {
            error!("List query failed: {}", e);
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    let mut ctx = get_context(&session, &title, &*resource, &slug);
    ctx.insert("page_title", resource.plural_name());
    ctx.insert("resource", &resource.info(&prefix.0));
    ctx.insert("columns", &resource.list_columns());
    ctx.insert("rows", &result.rows);
    ctx.insert("page", &result.page);
    ctx.insert("per_page", &result.per_page);
    ctx.insert("total", &result.total);
    ctx.insert("total_pages", &((result.total + result.per_page - 1) / result.per_page));
    
    let prefix_trimmed = prefix.0.trim_start_matches('/');
    ctx.insert("path_dashboard", &format!("/{}/", prefix_trimmed));
    ctx.insert("path_logout", &format!("/{}/logout", prefix_trimmed));
    ctx.insert("user", &user);
    
    ctx.insert("resources", &registry.all().iter().map(|r| r.info(&prefix.0)).collect::<Vec<_>>());

    match tmpl.render("list.html", &ctx) {
        Ok(rendered) => {
            info!("Template rendered successfully for list page");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        },
        Err(e) => {
            error!("Template rendering failed: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn new(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish(),
    };

    let resource = match registry.get(&slug) {
        Some(r) => r,
        None => return HttpResponse::NotFound().finish(),
    };

    let mut ctx = get_context(&session, &title, &*resource, &slug);
    ctx.insert("page_title", "New Record");
    ctx.insert("resource", &resource.info(&prefix.0));
    ctx.insert("fields", &resource.form_fields());
    ctx.insert("values", &HashMap::<String, serde_json::Value>::new());
    ctx.insert("errors", &HashMap::<String, String>::new());
    ctx.insert("is_new", &true);
    ctx.insert("resources", &registry.all().iter().map(|r| r.info(&prefix.0)).collect::<Vec<_>>());
    let prefix_trimmed = prefix.0.trim_start_matches('/');
    ctx.insert("path_dashboard", &format!("/{}/", prefix_trimmed));
    ctx.insert("path_logout", &format!("/{}/logout", prefix_trimmed));
    ctx.insert("user", &user);

    match tmpl.render("form.html", &ctx) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<String>,
    form: web::Form<HashMap<String, String>>,
) -> impl Responder {
    let slug = path.into_inner();
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish(),
    };

    let resource = match registry.get(&slug) {
        Some(r) => r,
        None => return HttpResponse::NotFound().finish(),
    };

    let mut data = HashMap::new();
    for (k, v) in form.into_inner() {
        data.insert(k, serde_json::Value::String(v));
    }

    match resource.validate(&data).await {
        Ok(_) => {
            match resource.create(data).await {
                Ok(_) => HttpResponse::Found().insert_header(("Location", format!("/{}/{}/", prefix.0, slug))).finish(),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(errors) => {
            let mut ctx = get_context(&session, &title, &*resource, &slug);
            ctx.insert("page_title", "New Record");
            ctx.insert("resource", &resource.info(&prefix.0));
            ctx.insert("fields", &resource.form_fields());
            ctx.insert("values", &data);
            ctx.insert("errors", &errors);
            ctx.insert("is_new", &true);
            ctx.insert("resources", &registry.all().iter().map(|r| r.info(&prefix.0)).collect::<Vec<_>>());
            ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
            ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
            ctx.insert("user", &user);
            
            match tmpl.render("form.html", &ctx) {
                Ok(rendered) => HttpResponse::BadRequest().content_type("text/html").body(rendered),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

pub async fn edit(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (slug, id) = path.into_inner();
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish(),
    };

    let resource = match registry.get(&slug) {
        Some(r) => r,
        None => return HttpResponse::NotFound().finish(),
    };

    let val = match resource.get(&id).await {
        Ok(v) => v,
        Err(AdminError::NotFound) => return HttpResponse::NotFound().finish(),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let mut values = HashMap::new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            values.insert(k.clone(), v.clone());
        }
    }

    let mut ctx = get_context(&session, &title, &*resource, &slug);
    ctx.insert("page_title", "Edit Record");
    ctx.insert("resource", &resource.info(&prefix.0));
    ctx.insert("fields", &resource.form_fields());
    ctx.insert("values", &values);
    ctx.insert("errors", &HashMap::<String, String>::new());
    ctx.insert("is_new", &false);
    ctx.insert("resources", &registry.all().iter().map(|r| r.info(&prefix.0)).collect::<Vec<_>>());
    let prefix_trimmed = prefix.0.trim_start_matches('/');
    ctx.insert("path_dashboard", &format!("/{}/", prefix_trimmed));
    ctx.insert("path_logout", &format!("/{}/logout", prefix_trimmed));
    ctx.insert("user", &user);

    match tmpl.render("form.html", &ctx) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<(String, String)>,
    form: web::Form<HashMap<String, String>>,
) -> impl Responder {
    let (slug, id) = path.into_inner();
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish(),
    };

    let resource = match registry.get(&slug) {
        Some(r) => r,
        None => return HttpResponse::NotFound().finish(),
    };

    let mut data = HashMap::new();
    for (k, v) in form.into_inner() {
        data.insert(k, serde_json::Value::String(v));
    }

    match resource.validate(&data).await {
        Ok(_) => {
            match resource.update(&id, data).await {
                Ok(_) => HttpResponse::Found().insert_header(("Location", format!("/{}/{}/", prefix.0, slug))).finish(),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(errors) => {
            let mut ctx = get_context(&session, &title, &*resource, &slug);
            ctx.insert("page_title", "Edit Record");
            ctx.insert("resource", &resource.info(&prefix.0));
            ctx.insert("fields", &resource.form_fields());
            ctx.insert("values", &data);
            ctx.insert("errors", &errors);
            ctx.insert("is_new", &false);
            ctx.insert("resources", &registry.all().iter().map(|r| r.info(&prefix.0)).collect::<Vec<_>>());
            ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
            ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
            ctx.insert("user", &user);
            
            match tmpl.render("form.html", &ctx) {
                Ok(rendered) => HttpResponse::BadRequest().content_type("text/html").body(rendered),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

pub async fn delete(
    session: Session,
    registry: web::Data<SharedRegistry>,
    prefix: web::Data<AdminPrefix>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (slug, id) = path.into_inner();
    let _user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Found().insert_header(("Location", format!("/{}/login", prefix.0))).finish(),
    };

    let resource = match registry.get(&slug) {
        Some(r) => r,
        None => return HttpResponse::NotFound().finish(),
    };

    match resource.delete(&id).await {
        Ok(_) => HttpResponse::Found().insert_header(("Location", format!("/{}/{}/", prefix.0, slug))).finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
