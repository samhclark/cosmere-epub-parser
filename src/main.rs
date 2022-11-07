use std::{fs::File, io::BufReader, collections::HashMap};

use axum::{
    extract::{Path, Query, Json},
    routing::{get, post},
    http::StatusCode,
    response::{IntoResponse, Html},
    Router,
};
use epub::doc::EpubDoc;
use html2text::from_read;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Schema, STORED, TEXT},
    DocAddress, Index, Score,
};

#[derive(Serialize, Deserialize, Debug)]
struct LineScheme {
    book_title: String,
    chapter_title: String,
    chapter_content: String,
}

#[tokio::main]
async fn main() {
    let doc = EpubDoc::new("./the-bands-of-mourning.epub");
    match doc {
        Ok(bom) => do_tantivy(bom),
        Err(_) => println!("Skipping, not found: The Bands of Mourning"),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(index))
        .route("/search", get(search));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    let index = include_str!("../assets/index.html");
    Html(index)
}

async fn search(Query(params): Query<HashMap<String, String>>) {
    let search_term = params.get("q").unwrap();
    println!("You searched for {}", search_term);
}

fn print_to_stdout(mut doc: EpubDoc<BufReader<File>>) {
    let book_title = "The Bands of Mourning";
    let first_chapter_index: usize = 7;
    let skipable_indexes = vec![8, 13, 26];
    let last_chapter_index: usize = 42;

    for i in first_chapter_index..=last_chapter_index {
        if skipable_indexes.contains(&i) {
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
            if line.chars().nth(0).unwrap() == '[' {
                continue;
            }
            // println!("{}|{}|{}", book_title, chapter_title, line);
            let foo = LineScheme {
                book_title: book_title.to_string(),
                chapter_title: chapter_title.clone(),
                chapter_content: line.to_string(),
            };
            let j = serde_json::to_string(&foo).unwrap();
            println!("{}", j);
        }
    }
}

fn do_tantivy(mut doc: EpubDoc<BufReader<File>>) {
    let mut schema_builder = Schema::builder();
    let book_title_field = schema_builder.add_text_field("book_title", TEXT | STORED);
    let chapter_title_field = schema_builder.add_text_field("chapter_title", TEXT | STORED);
    let paragraph_field = schema_builder.add_text_field("paragraph_field", TEXT | STORED);
    let schema = schema_builder.build();

    let index = Index::create_from_tempdir(schema.clone()).unwrap();

    // Here we use a buffer of 100MB that will be split
    // between indexing threads.
    let mut index_writer = index.writer(128_000_000).unwrap();

    let book_title = "The Bands of Mourning";
    let first_chapter_index: usize = 7;
    let skipable_indexes = vec![8, 13, 26];
    let last_chapter_index: usize = 42;

    for i in first_chapter_index..=last_chapter_index {
        if skipable_indexes.contains(&i) {
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
            if line.chars().nth(0).unwrap() == '[' {
                continue;
            }
            // println!("{}|{}|{}", book_title, chapter_title, line);
            index_writer.add_document(doc!(book_title_field => book_title, chapter_title_field => chapter_title.clone(), paragraph_field => line)).unwrap();
        }
    }

    index_writer.commit().unwrap();

    let reader = index.reader().unwrap();

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![paragraph_field]);

    // QueryParser may fail if the query is not in the right
    // format. For user facing applications, this can be a problem.
    // A ticket has been opened regarding this problem.
    let query = query_parser.parse_query("sea whale").unwrap();

    // Perform search.
    // `topdocs` contains the 10 most relevant doc ids, sorted by decreasing scores...
    let top_docs: Vec<(Score, DocAddress)> =
        searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

    for (_score, doc_address) in top_docs {
        // Retrieve the actual content of documents given its `doc_address`.
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        println!("{}", schema.to_json(&retrieved_doc));
    }
}
