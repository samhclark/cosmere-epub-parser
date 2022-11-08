use std::collections::HashMap;

use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use domain::{HtmlTemplate, IndexableBook, ResultsTemplate, RichParagraph};
use epub::doc::EpubDoc;
use html2text::from_read;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Schema, STORED, TEXT},
    DocAddress, Index, Score,
};

mod domain;

#[tokio::main]
async fn main() {
    let bands_of_mourning = build_bands_of_mourning();
    let shadows_of_self = build_shadows_of_self();
    // inspect(shadows_of_self);

    let tantivy_index = build_search_index();
    add_book(bands_of_mourning, &tantivy_index);
    add_book(shadows_of_self, &tantivy_index);

    let app = Router::new()
        .route("/", get(index))
        .route("/search", get(|q| search(q, tantivy_index)));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    let index = include_str!("../assets/index.html");
    Html(index)
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
        searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

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

fn build_bands_of_mourning() -> IndexableBook {
    let epub =
        EpubDoc::new("./the-bands-of-mourning.epub").expect("Not found: The Bands of Mourning");
    IndexableBook {
        title: "The Bands of Mourning".to_string(),
        epub_file: epub,
        first_chapter_index: 7,
        last_chapter_index: 42,
        skippable_chapters: vec![8, 13, 26],
    }
}

fn build_shadows_of_self() -> IndexableBook {
    let epub = EpubDoc::new("./shadows-of-self.epub").expect("Not found: Shadows of Self");
    IndexableBook {
        title: "Shadows of Self".to_string(),
        epub_file: epub,
        first_chapter_index: 7,
        last_chapter_index: 37,
        skippable_chapters: vec![8, 13, 31],
    }
}

fn build_search_index() -> Index {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("book_title", TEXT | STORED);
    schema_builder.add_text_field("chapter_title", TEXT | STORED);
    schema_builder.add_text_field("paragraph", TEXT | STORED);
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
                    chapter_field => chapter_title.clone(),
                    paragraph_field => line))
                .unwrap();
        }
    }

    index_writer.commit().unwrap();
}
