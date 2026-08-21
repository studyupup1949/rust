use actix_web::web::{self, Data, ServiceConfig};
use actix_web::HttpResponse;

use crate::*;

pub fn configure<C: Claims>(
    session_info_path: Option<&'static str>,
    refresh_path: &'static str,
    config: &mut ServiceConfig,
) {
    if let Some(session_info) = session_info_path {
        config.service(
            web::resource(session_info)
                .get(|auth: Authenticated<C>| async move { HttpResponse::Ok().json(&*auth) }),
        );
    }
    config.service(web::resource(refresh_path).get(
        |refresh_token: Authenticated<RefreshToken>,
         storage: Data<SessionStorage>,
         extractors: Data<Extractors<_>>| {
            refresh_session::<C>(refresh_token, storage, extractors)
        },
    ));
}

async fn refresh_session<AppClaims: Claims>(
    refresh_token: Authenticated<RefreshToken>,
    storage: Data<SessionStorage>,
    extractors: Data<Extractors<AppClaims>>,
) -> HttpResponse {
    let s = storage.into_inner();
    let pair = match s.refresh::<AppClaims>(refresh_token.access_jti()).await {
        Err(e) => {
            tracing::warn!("Failed to refresh token: {e}");
            return HttpResponse::BadRequest().finish();
        }
        Ok(pair) => pair,
    };

    let encrypted_jwt = match pair.jwt.encode() {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!("Failed to encode claims: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    let encrypted_refresh = match pair.refresh.encode() {
        Err(e) => {
            tracing::warn!("Failed to encode claims: {e}");
            return HttpResponse::InternalServerError().finish();
        }
        Ok(text) => text,
    };

    let mut builder = HttpResponse::Ok();
    let bearer = format!("Bearer {encrypted_jwt}");
    for (_, name) in extractors
        .jwt_extractors
        .iter()
        .filter_map(|e| e.extractor_key())
        .filter(|(kind, _)| *kind == ExtractorKind::Header)
    {
        builder.append_header((name.to_string().as_str(), bearer.as_str()));
    }
    for (_, name) in extractors
        .refresh_extractors
        .iter()
        .filter_map(|e| e.extractor_key())
        .filter(|(kind, _)| *kind == ExtractorKind::Header)
    {
        builder.append_header((name.to_string().as_str(), encrypted_refresh.as_str()));
    }
    builder.append_header((
        "ACX-JWT-TTL",
        (pair.refresh.issues_at + pair.refresh.refresh_ttl.0).to_string(),
    ));
    builder.finish()
}
