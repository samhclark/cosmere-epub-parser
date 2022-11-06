use epub::doc::EpubDoc;
use html2text::from_read;

fn main() {
    let doc = EpubDoc::new("/home/sam/Downloads/cosmere-ebooks/the-bands-of-mourning.epub");
    assert!(doc.is_ok());
    let mut doc = doc.unwrap();
    // println!("{}", doc.mdata("title").unwrap());
    // println!("spine: {:?}", doc.spine);
    // println!("metadata: {:?}", doc.metadata);
    // println!("toc: {:?}", doc.toc[9].children[2].content);

    // doc.set_current_page(9).unwrap();
    // println!("?? {:?}", doc.get_current_str());
    // dammit, it doesn't actually get me much. Just prints out the html. Could have done this after unzipping the epub

    // let this_page = doc.get_current().unwrap();
    // let page_content = from_read(&this_page[..], usize::MAX);
    // println!("page as string?: {:?}", foo)
    // okay! That kinda works. Each paragraph is separated by a blank newline. Would need to recombine the lines that from_read breaks

    // just gonna max out usize and do it by lines
    let book_title = "The Bands of Mourning";
    let first_chapter_index: usize = 7;
    let skipable_indexes = vec![8, 13, 26];
    let last_chapter_index: usize = 42;
    
    for i in first_chapter_index..=last_chapter_index {
        if skipable_indexes.contains(&i) {
            continue;
        }
        doc.set_current_page(i).expect("You got your indexes wrong, dude");
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

    // for line in page_content.lines() {
    //     if line.is_empty() {
    //         continue;
    //     }
    //     // println!("The Bands of Mourning|Chapter 5|{}", line);
    // }
}
