use std::collections::HashMap;

use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use books::{
    build_alloy_of_law, build_bands_of_mourning, build_secret_history, build_shadows_of_self,
};
use domain::{HtmlTemplate, IndexableBook, ResultsTemplate, RichParagraph};
use html2text::from_read;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Schema, STORED, TEXT, TextOptions, TextFieldIndexing, IndexRecordOption},
    DocAddress, Index, Score,
};

mod books;
mod domain;

#[tokio::main]
async fn main() {
    let bands_of_mourning = build_bands_of_mourning();
    let shadows_of_self = build_shadows_of_self();
    let alloy_of_law = build_alloy_of_law();
    let secret_history = build_secret_history();
    // inspect(secret_history);

    let tantivy_index = build_search_index();
    add_book(bands_of_mourning, &tantivy_index);
    add_book(shadows_of_self, &tantivy_index);
    add_book(alloy_of_law, &tantivy_index);
    add_book(secret_history, &tantivy_index);

    let app = Router::new()
        .route("/", get(root))
        .route("/search", get(|q| search(q, tantivy_index)));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    let homepage = include_str!("../assets/index.html");
    Html(homepage)
}

async fn search(Query(params): Query<HashMap<String, String>>, index: Index) -> impl IntoResponse {
    let search_term = params.get("q").unwrap();
    println!("You searched for {}", search_term);
    let reader = index.reader().unwrap();

    let searcher = reader.searcher();

    let book_field = index.schema().get_field("book_title").unwrap();
    let chapter_field = index.schema().get_field("chapter_title").unwrap();
    let paragraph_field = index.schema().get_field("paragraph").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![paragraph_field]);

    // QueryParser may fail if the query is not in the right
    // format. For user facing applications, this can be a problem.
    // A ticket has been opened regarding this problem.
    let query = query_parser.parse_query(search_term).unwrap();

    // Perform search.
    // `topdocs` contains the 10 most relevant doc ids, sorted by decreasing scores...
    let top_docs: Vec<(Score, DocAddress)> =
        searcher.search(&query, &TopDocs::with_limit(20)).unwrap();

    // top_docs
    let mut more_results: Vec<RichParagraph> = vec![];
    for (_score, doc_address) in top_docs {
        // Retrieve the actual content of documents given its `doc_address`.
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        // println!("{}", index.schema().to_json(&retrieved_doc));
        more_results.push(RichParagraph {
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
        search_results: more_results,
    };
    HtmlTemplate(template)
}

fn inspect(book: IndexableBook) {
    println!("Spine: ");
    for (i, s) in book.epub_file.spine.iter().enumerate() {
        println!("{}\t{}", i, s);
    }
}

fn build_search_index() -> Index {
    let mut schema_builder = Schema::builder();

    let text_options = TextOptions::default()
        .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("en_stem")
            .set_index_option(IndexRecordOption::Basic)
        )
        .set_stored();

    schema_builder.add_text_field("book_title", TEXT | STORED);
    schema_builder.add_text_field("chapter_title", TEXT | STORED);
    schema_builder.add_text_field("paragraph", text_options);
    let schema = schema_builder.build();

    Index::create_from_tempdir(schema).expect("Failed to build index")
}

fn add_book(book: IndexableBook, index: &Index) {
    let mut index_writer = index.writer(128_000_000).unwrap();

    let book_field = index.schema().get_field("book_title").unwrap();
    let chapter_field = index.schema().get_field("chapter_title").unwrap();
    let paragraph_field = index.schema().get_field("paragraph").unwrap();

    let mut doc = book.epub_file;

    for i in book.first_chapter_index..=book.last_chapter_index {
        if book.skippable_chapters.contains(&i) {
            continue;
        }
        doc.set_current_page(i)
            .expect("You got your indexes wrong, dude");
        let chapter_title = doc.spine[i].clone();
        let this_page = doc.get_current().unwrap();
        let page_content = from_read(&this_page[..], usize::MAX);
        for line in page_content.lines() {
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                continue;
            }
            index_writer
                .add_document(doc!(
                    book_field => book.title.clone(),
                    chapter_field => pretty_chapter(&chapter_title),
                    paragraph_field => line))
                .unwrap();
        }
    }

    index_writer.commit().unwrap();
}

fn pretty_chapter(raw_chapter: &str) -> String {
    if raw_chapter.to_ascii_lowercase() == "prologue" {
        String::from("Prologue")
    } else if raw_chapter.to_ascii_lowercase() == "epilogue" {
        String::from("Epilogue")
    } else if raw_chapter.to_ascii_lowercase().starts_with("chapter") {
        let num: String = raw_chapter
            .chars()
            .into_iter()
            .filter(|c| c.is_ascii_digit())
            .collect();
        format!("Chapter {num}")
    } else {
        String::from(raw_chapter)
    }
}
