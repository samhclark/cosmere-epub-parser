use std::{fmt::Display, fs::File, io::BufReader};

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use epub::doc::EpubDoc;

impl Display for RichParagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}", self.book, self.chapter, self.text)
    }
}

#[derive(Debug)]
pub struct RichParagraph {
    pub book: String,
    pub chapter: String,
    pub text: String,
}

#[derive(Template)]
#[template(path = "results.html")]
pub struct ResultsTemplate {
    pub search_term: String,
    pub search_results: Vec<RichParagraph>,
}

pub struct HtmlTemplate<T>(pub T);

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

pub struct IndexableBook {
    pub title: String,
    pub epub_file: EpubDoc<BufReader<File>>,
    pub first_chapter_index: usize,
    pub last_chapter_index: usize,
    pub skippable_chapters: Vec<usize>,
}
