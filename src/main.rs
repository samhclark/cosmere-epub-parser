use std::{
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
};

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};

use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};
use prometheus_client::encoding::text::encode;

use prometheus_client::registry::Registry;
use search_index::TantivyWrapper;

use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};
use tracing::Level;

use metrics_wrapper::MetricsWrapper;

mod domain;
mod search_index;
mod metrics_wrapper;
mod main_controller;

#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let metrics = MetricsWrapper::build();

    let tantivy_wrapper = Arc::new(TantivyWrapper::build());

    // Create application server
    let app = Router::new()
        .fallback(get_service(ServeDir::new("./assets")).handle_error(handle_error))
        .route("/search", get(|q| main_controller::search(q, tantivy_wrapper, metrics.http_requests)))
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
        ));

    let app_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    let app_server = axum::Server::bind(&app_addr).serve(app.into_make_service());
    tracing::info!("Application listening on {app_addr}");

    // Create metrics server
    let metrics_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 9091);
    let arc_registry = Arc::new(metrics.registry);
    let metrics_server = axum::Server::bind(&metrics_addr).serve(make_service_fn(move |_conn| {
        let registry = arc_registry.clone();
        async move {
            let handler = make_handler(registry);
            Ok::<_, io::Error>(service_fn(handler))
        }
    }));
    tracing::info!("Metrics server listening on {metrics_addr}");

    // Start both servers
    future::join(app_server, metrics_server).await;
}

/// This function returns a HTTP handler (i.e. another function)
pub fn make_handler(
    registry: Arc<Registry>,
) -> impl Fn(Request<Body>) -> Pin<Box<dyn Future<Output = io::Result<Response<Body>>> + Send>> {
    // This closure accepts a request and responds with the OpenMetrics encoding of the metrics.
    move |_req: Request<Body>| {
        let reg = registry.clone();
        Box::pin(async move {
            let mut buf = Vec::new();
            encode(&mut buf, &reg.clone()).map(|_| {
                let body = Body::from(buf);
                Response::builder()
                    .header(
                        hyper::header::CONTENT_TYPE,
                        "application/openmetrics-text; version=1.0.0; charset=utf-8",
                    )
                    .body(body)
                    .unwrap()
            })
        })
    }
}

#[allow(clippy::unused_async)]
async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}


