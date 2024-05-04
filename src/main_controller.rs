use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::Form;
use serde::Deserialize;
use tantivy::{
    collector::{Count, TopDocs},
    query::QueryParser,
    schema::Value,
    DocAddress, Score, TantivyDocument,
};

use crate::{
    domain::{BookState, HtmlTemplate, ResultsTemplate, RichParagraph},
    AppState,
};

struct Book<'a> {
    short_name: &'a str,
    long_name: &'a str,
}

const SEARCHABLE_BOOKS: &[&Book] = &[
    &Book {
        short_name: "wok",
        long_name: "The Way of Kings",
    },
    &Book {
        short_name: "aol",
        long_name: "The Alloy of Law",
    },
    &Book {
        short_name: "sos",
        long_name: "Shadows of Self",
    },
    &Book {
        short_name: "bom",
        long_name: "Bands of Mourning",
    },
    &Book {
        short_name: "sh",
        long_name: "Secret History",
    },
    &Book {
        short_name: "wb",
        long_name: "Warbreaker",
    },
    &Book {
        short_name: "tes",
        long_name: "The Emperor's Soul",
    },
    &Book {
        short_name: "thoe",
        long_name: "The Hope of Elantris",
    },
];

/// GET /search
#[allow(clippy::unused_async)]
pub async fn search(
    State(state): State<AppState>,
    Form(params): Form<Params>,
) -> impl IntoResponse {
    let search_term: String = params
        .query
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c == &' ')
        .collect();

    let search_books: Vec<String> = params
        .books
        .into_iter()
        .filter_map(|it| get_title(&it))
        .collect();
    tracing::info!("Searched for \"{}\" in {:?}", search_term, search_books);

    let searcher = state.tantivy.reader.searcher();

    let query_parser = QueryParser::for_index(
        &state.tantivy.index,
        vec![state.tantivy.book, state.tantivy.searchable_text],
    );
    let complete_query_text = if search_books.is_empty() || search_term.trim().is_empty() {
        search_term.clone()
    } else {
        let book_filter_query = search_books
            .iter()
            .map(|it| format!("book_title:\"{it}\""))
            .collect::<Vec<String>>()
            .join(" OR ");
        format!("({book_filter_query}) AND paragraph:\"{search_term}\"")
    };
    tracing::debug!("Constructed query is {}", &complete_query_text);
    let query = query_parser.parse_query(&complete_query_text).unwrap();

    let total_matches = searcher.search(&query, &Count).unwrap();
    let top_docs: Vec<(Score, DocAddress)> =
        searcher.search(&query, &TopDocs::with_limit(20)).unwrap();

    let mut results: Vec<RichParagraph> = vec![];
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
        results.push(RichParagraph {
            book: retrieved_doc
                .get_first(state.tantivy.book)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            chapter: retrieved_doc
                .get_first(state.tantivy.chapter)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            text: retrieved_doc
                .get_first(state.tantivy.passage)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        });
    }

    let search_state = SEARCHABLE_BOOKS
        .iter()
        .map(|it| BookState {
            title: String::from(it.long_name),
            short_name: String::from(it.short_name),
            checked: search_books.contains(&String::from(it.long_name)),
        })
        .collect();
    let template = ResultsTemplate {
        search_term,
        search_results: results,
        total_matches,
        search_state,
    };
    HtmlTemplate(template)
}

fn get_title(abbr: &String) -> Option<String> {
    for &book in SEARCHABLE_BOOKS {
        if book.short_name == abbr {
            return Some(String::from(book.long_name));
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct Params {
    #[serde(default, rename = "q")]
    query: String,

    #[serde(default, rename = "book")]
    books: Vec<String>,
}
