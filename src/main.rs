use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use search_index::TantivyWrapper;

use tower_http::{
    services::ServeDir, 
    set_header::SetResponseHeaderLayer,
};
use tracing::Level;

mod domain;
mod main_controller;
mod search_index;

#[derive(Clone)]
pub struct AppState {
    tantivy: TantivyWrapper,
}

#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let tantivy_wrapper = TantivyWrapper::new();

    let serve_dir = get_service(ServeDir::new("./assets")).handle_error(handle_error);

    let app = Router::new()
        .nest_service("/", serve_dir.clone())
        .fallback_service(serve_dir)
        .route(
            "/search",
            get(main_controller::search),
        )
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'none'; img-src 'self'; script-src 'self'; style-src 'self'",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000"),
        ))
        .with_state(AppState { tantivy: tantivy_wrapper });

    let app_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    tracing::info!("Application listening on {app_addr}");
    axum::Server::bind(&app_addr).serve(app.into_make_service()).await;
}

#[allow(clippy::unused_async)]
async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
