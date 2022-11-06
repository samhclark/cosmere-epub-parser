use epub::doc::EpubDoc;

fn main() {
    let doc = EpubDoc::new("/home/sam/Downloads/cosmere-ebooks/the-bands-of-mourning.epub");
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();
    println!("{}", doc.mdata("title").unwrap());
    println!("spine: {:?}", doc.spine);
    println!("metadata: {:?}", doc.metadata);
    println!("toc: {:?}", doc.toc[9].children[2].content);

    doc.set_current_page(9).unwrap();
    println!("?? {:?}", doc.get_current_str());
    // dammit, it doesn't actually get me much. Just prints out the html. Could have done this after unzipping the epub
}
