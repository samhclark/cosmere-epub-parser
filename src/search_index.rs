use std::{
    fs::File,
    io::{self, BufRead},
};

use tantivy::{
    doc,
    Index,
    schema::{Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, STORED, TEXT}, IndexReader,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InputSchema {
    book_title: String,
    chapter_title: String,
    searchable_text: String,
}

pub struct TantivyWrapper {
    pub index: Index,
    pub reader: IndexReader,
    pub book: Field,
    pub chapter: Field,
    pub searchable_text: Field,
}

impl TantivyWrapper {
    pub fn build() -> Self {
        println!("inside build");
        let wrapper = create_empty_index();
        load_search_index(&wrapper);
    
        wrapper
    }
}

fn create_empty_index() -> TantivyWrapper {
    println!("inside create");
    let mut schema_builder = Schema::builder();

    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("en_stem")
                .set_index_option(IndexRecordOption::Basic),
        )
        .set_stored();

    let book: Field = schema_builder.add_text_field("book_title", TEXT | STORED);
    let chapter: Field = schema_builder.add_text_field("chapter_title", TEXT | STORED);
    let searchable_text: Field = schema_builder.add_text_field("paragraph", text_options);
    let schema = schema_builder.build();

    let index = Index::create_from_tempdir(schema).unwrap();
    // index.set_multithread_executor(16).unwrap();
    let reader = index.reader().unwrap();

    TantivyWrapper {
        index,
        reader,
        book,
        chapter,
        searchable_text,
    }
}

fn load_search_index(tantivy: &TantivyWrapper) {
    println!("inside load");
    let mut index_writer = tantivy.index.writer(128_000_000).unwrap();

    let infile = File::open("input.json").expect("input file is required");
    let file_writer = io::BufReader::new(infile);
    for line in file_writer.lines() {
        let data: InputSchema = serde_json::from_str(line.unwrap().as_str()).unwrap();
        index_writer
            .add_document(doc!(
                tantivy.book => data.book_title,
                tantivy.chapter => data.chapter_title,
                tantivy.searchable_text => data.searchable_text))
            .unwrap();
    }

    index_writer.commit().unwrap();
    tantivy.reader.reload().unwrap();
}