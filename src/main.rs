use std::{collections::HashMap, fs::File, io::BufReader};

use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use epub::doc::EpubDoc;
use html2text::from_read;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Schema, STORED, TEXT},
    DocAddress, Index, Score,
};

struct SearchResult {
    book: String,
    chapter: String,
    text: String,
}

#[derive(Template)]
#[template(path = "results.html")]
struct ResultsTemplate {
    search_term: String,
    search_results: Vec<SearchResult>,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    let doc = EpubDoc::new("./the-bands-of-mourning.epub");
    let tantivy_index = match doc {
        Ok(bom) => build_tantivy_index(bom),
        Err(_) => panic!("Skipping, not found: The Bands of Mourning"),
    };

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
    let mut more_results: Vec<SearchResult> = vec![];
    for (_score, doc_address) in top_docs {
        // Retrieve the actual content of documents given its `doc_address`.
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        // println!("{}", index.schema().to_json(&retrieved_doc));
        more_results.push(SearchResult {
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

fn build_tantivy_index(mut doc: EpubDoc<BufReader<File>>) -> Index {
    let mut schema_builder = Schema::builder();
    let book_title_field = schema_builder.add_text_field("book_title", TEXT | STORED);
    let chapter_title_field = schema_builder.add_text_field("chapter_title", TEXT | STORED);
    let paragraph_field = schema_builder.add_text_field("paragraph", TEXT | STORED);
    let schema = schema_builder.build();

    let index = Index::create_from_tempdir(schema).unwrap();

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
            if line.starts_with('[') {
                continue;
            }
            index_writer
                .add_document(doc!(
                    book_title_field => book_title, 
                    chapter_title_field => chapter_title.clone(), 
                    paragraph_field => line))
                .unwrap();
        }
    }

    index_writer.commit().unwrap();

    index
}
