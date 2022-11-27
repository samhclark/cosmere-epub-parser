use axum::{extract::{Query, State}, response::IntoResponse};
use serde::Deserialize;
use tantivy::{collector::TopDocs, query::QueryParser, DocAddress, Score};

use crate::{
    domain::{HtmlTemplate, ResultsTemplate, RichParagraph},
    AppState,
};

/// GET /search
#[allow(clippy::unused_async)]
pub async fn search(
    Query(params): Query<Params>,
    State(state): State<AppState>
) -> impl IntoResponse {
    let search_term: String = params.q
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c == &' ')
        .collect();
    tracing::info!("Searched for \"{}\"", search_term);

    let searcher = state.tantivy.reader.searcher();
    let query_parser = QueryParser::for_index(&state.tantivy.index, vec![state.tantivy.searchable_text]);

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
                .get_first(state.tantivy.book)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            chapter: retrieved_doc
                .get_first(state.tantivy.chapter)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string(),
            text: retrieved_doc
                .get_first(state.tantivy.searchable_text)
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

#[derive(Debug, Deserialize)]
pub struct Params {
    q: String,
}