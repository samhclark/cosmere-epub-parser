use std::{fs::File, io::BufReader};

use epub::doc::EpubDoc;
use html2text::from_read;

fn main() {
    let doc = EpubDoc::new("./the-bands-of-mourning.epub");
    match doc {
        Ok(bom) => print_to_stdout(bom),
        Err(_) => println!("Skipping, not found: The Bands of Mourning"),
    }
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
            println!("{}|{}|{}", book_title, chapter_title, line);
        }
    }
}
