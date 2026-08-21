use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};
use crate::registry::SharedRegistry;
use crate::resource::{AdminTitle, AdminPrefix};
use log::{info, error, warn};

pub async fn index(
    session: Session,
    registry: web::Data<SharedRegistry>,
    tmpl: web::Data<Tera>,
    title: web::Data<AdminTitle>,
    prefix: web::Data<AdminPrefix>,
) -> impl Responder {
    info!("Accessing dashboard");
    
    let user = match session.get::<String>("admin_user") {
        Ok(Some(u)) => {
            info!("User authenticated: {}", u);
            u
        },
        _ => {
            warn!("User not authenticated, redirecting to login");
            return HttpResponse::Found()
                .insert_header(("Location", format!("/{}/login", prefix.0)))
                .finish();
        }
    };

    let mut ctx = Context::new();
    ctx.insert("title", &title.0);
    ctx.insert("page_title", "Dashboard");
    ctx.insert("user", &user);
    ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
    ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
    ctx.insert("current_slug", ""); // Dashboard doesn't have a current slug
    
    // Simplify resources to basic strings to test serialization
    let simple_resources: Vec<_> = registry.all().iter().map(|r| {
        serde_json::json!({
            "name": r.name(),
            "plural_name": r.plural_name(),
            "slug": r.slug(),
            "icon": r.icon(),
            "path_list": format!("{}/{}/", prefix.0, r.slug()),
            "path_new": format!("{}/{}/new", prefix.0, r.slug()),
        })
    }).collect();
    info!("Found {} resources", simple_resources.len());
    ctx.insert("resources", &simple_resources);

    match tmpl.render("dashboard.html", &ctx) {
        Ok(rendered) => {
            info!("Dashboard template rendered successfully");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        },
        Err(e) => {
            error!("Dashboard template rendering failed: {}", e);
            error!("Template context: title={:?}, page_title={:?}, user={:?}, resources={:?}", 
                   title.0, "Dashboard", user, simple_resources);
            
            // Try to render without resources to isolate the issue
            let mut simple_ctx = Context::new();
            simple_ctx.insert("title", &title.0);
            simple_ctx.insert("page_title", "Dashboard");
            simple_ctx.insert("user", &user);
            simple_ctx.insert("path_dashboard", &format!("/{}/", prefix.0));
            simple_ctx.insert("path_logout", &format!("/{}/logout", prefix.0));
            simple_ctx.insert("resources", &Vec::<crate::resource::ResourceInfo>::new());
            
            match tmpl.render("dashboard.html", &simple_ctx) {
                Ok(_) => error!("Template works without resources - issue with ResourceInfo serialization"),
                Err(e2) => error!("Template fails even without resources: {}", e2),
            }
            
            HttpResponse::InternalServerError().body(format!("Template error: {}", e))
        }
    }
}
