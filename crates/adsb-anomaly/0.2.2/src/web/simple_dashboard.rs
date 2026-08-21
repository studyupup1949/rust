// ABOUTME: Simplified dashboard implementation for testing template compilation
// ABOUTME: Basic HTML responses without complex template derivation

use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::store::sessions;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
}

/// Simple dashboard routes
pub fn router() -> Router<SqlitePool> {
    Router::new().route("/simple", get(simple_sessions_page))
}

/// Handler for simple sessions page
async fn simple_sessions_page(
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let limit = pagination.limit.unwrap_or(10).min(50);

    match sessions::list_sessions(&pool, limit).await {
        Ok(sessions) => {
            let mut html = String::from(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>Simple ADS-B Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <h1>Aircraft Sessions</h1>
    <table>
        <thead>
            <tr>
                <th>HEX</th>
                <th>Flight</th>
                <th>Position</th>
                <th>Last Seen</th>
                <th>Messages</th>
            </tr>
        </thead>
        <tbody>
"#,
            );

            for session in sessions {
                let flight = session.flight.unwrap_or_else(|| "—".to_string());
                let position = match (session.lat, session.lon) {
                    (Some(lat), Some(lon)) => format!("{:.4}, {:.4}", lat, lon),
                    _ => "—".to_string(),
                };

                html.push_str(&format!(
                    r#"            <tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
"#,
                    session.hex, flight, position, session.last_seen_ms, session.message_count
                ));
            }

            html.push_str(
                r#"        </tbody>
    </table>
</body>
</html>"#,
            );

            Ok(Html(html))
        }
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
