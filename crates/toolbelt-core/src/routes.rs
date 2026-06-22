use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use include_dir::{include_dir, Dir};
use toolbelt_types::HealthResponse;

static WEB_DIST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../web/dist");

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .fallback(get(serve_ui))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

async fn serve_ui(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = WEB_DIST.get_file(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        return ([(header::CONTENT_TYPE, mime)], file.contents().to_vec()).into_response();
    }

    // SPA fallback: unknown paths serve index.html so client-side routing works
    if let Some(file) = WEB_DIST.get_file("index.html") {
        return (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            file.contents().to_vec(),
        )
            .into_response();
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        "UI not built — run `cargo xtask build-web` first",
    )
        .into_response()
}
