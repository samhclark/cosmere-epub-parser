use std::fmt::Display;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InputSchema {
    pub book_title: String,
    pub chapter_title: String,
    pub searchable_text: String,
}

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
