use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, response::IntoResponse};
use prometheus_client::metrics::{family::Family, counter::Counter};
#[allow(clippy::unused_async)]
use tantivy::{
    collector::TopDocs,
    query::QueryParser,

    DocAddress, Score,
};

use crate::{search_index::TantivyWrapper, domain::{RichParagraph, ResultsTemplate, HtmlTemplate}};

pub async fn search(
    Query(params): Query<HashMap<String, String>>,
    tantivy: Arc<TantivyWrapper>,
    http_requests: Family<(String, String), Counter>,
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

    let searcher = tantivy.reader.searcher();
    let query_parser = QueryParser::for_index(&tantivy.index, vec![tantivy.searchable_text]);

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
                .get_first(tantivy.book)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            chapter: retrieved_doc
                .get_first(tantivy.chapter)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            text: retrieved_doc
                .get_first(tantivy.searchable_text)
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