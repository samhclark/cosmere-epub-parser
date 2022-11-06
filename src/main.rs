use epub::doc::EpubDoc;

fn main() {
    let doc = EpubDoc::new("/home/sam/Downloads/cosmere-ebooks/the-bands-of-mourning.epub");
    assert!(doc.is_ok());
    let doc = doc.unwrap();
    println!("{}", doc.mdata("title").unwrap());
}
