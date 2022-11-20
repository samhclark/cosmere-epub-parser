use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    io::{self, BufRead},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
};

use axum::{
    extract::Query,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use domain::{HtmlTemplate, InputSchema, ResultsTemplate, RichParagraph};
use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions, STORED, TEXT},
    DocAddress, Index, Score,
};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};
use tracing::Level;

mod domain;

type Label = (String, String);

#[allow(unused_must_use)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let mut registry = <Registry>::with_prefix("csearch");
    let http_requests = Family::<Label, Counter>::default();
    registry.register(
        "http_requests",
        "Number of HTTP requests received",
        Box::new(http_requests.clone()),
    );

    let tantivy_index = build_search_index();
    load_search_index(&tantivy_index);

    // Create application server
    let app = Router::new()
        .fallback(get_service(ServeDir::new("./assets")).handle_error(handle_error))
        .route("/search", get(|q| search(q, tantivy_index, http_requests)))
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
    let arc_registry = Arc::new(registry);
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

#[allow(clippy::unused_async)]
async fn search(
    Query(params): Query<HashMap<String, String>>,
    index: Index,
    http_requests: Family<Label, Counter>,
) -> impl IntoResponse {
    http_requests
        .get_or_create(&(String::from("GET"), String::from("/search")))
        .inc();
    let search_term: String = params
        .get("q")
        .unwrap()
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c == &' ')
        .collect();
    tracing::info!("Searched for \"{}\"", search_term);
    let reader = index.reader().unwrap();

    let searcher = reader.searcher();

    let book_field = index.schema().get_field("book_title").unwrap();
    let chapter_field = index.schema().get_field("chapter_title").unwrap();
    let paragraph_field = index.schema().get_field("paragraph").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![paragraph_field]);

    // QueryParser may fail if the query is not in the right format
    // TODO: toss up a 400 Bad Request when that happens
    let query = query_parser.parse_query(&search_term).unwrap();

    let top_docs: Vec<(Score, DocAddress)> =
        searcher.search(&query, &TopDocs::with_limit(20)).unwrap();

    let mut results: Vec<RichParagraph> = vec![];
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        results.push(RichParagraph {
            book: retrieved_doc
                .get_first(book_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            chapter: retrieved_doc
                .get_first(chapter_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            text: retrieved_doc
                .get_first(paragraph_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
        });
    }

    let template = ResultsTemplate {
        search_term: search_term.clone(),
        search_results: results,
    };
    HtmlTemplate(template)
}

fn build_search_index() -> Index {
    let mut schema_builder = Schema::builder();

    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("en_stem")
                .set_index_option(IndexRecordOption::Basic),
        )
        .set_stored();

    schema_builder.add_text_field("book_title", TEXT | STORED);
    schema_builder.add_text_field("chapter_title", TEXT | STORED);
    schema_builder.add_text_field("paragraph", text_options);
    let schema = schema_builder.build();

    Index::create_in_ram(schema)
}

fn load_search_index(index: &Index) {
    let mut index_writer = index.writer(128_000_000).unwrap();

    let book_field = index.schema().get_field("book_title").unwrap();
    let chapter_field = index.schema().get_field("chapter_title").unwrap();
    let paragraph_field = index.schema().get_field("paragraph").unwrap();

    let infile = File::open("input.json").expect("input file is required");
    let file_writer = io::BufReader::new(infile);
    for line in file_writer.lines() {
        let data: InputSchema = serde_json::from_str(line.unwrap().as_str()).unwrap();
        index_writer
            .add_document(doc!(
                    book_field => data.book_title,
                    chapter_field => data.chapter_title,
                    paragraph_field => data.searchable_text))
            .unwrap();
    }

    index_writer.commit().unwrap();
}
