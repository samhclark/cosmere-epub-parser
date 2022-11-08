use crate::domain::IndexableBook;
use epub::doc::EpubDoc;

pub fn build_bands_of_mourning() -> IndexableBook {
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

pub fn build_shadows_of_self() -> IndexableBook {
    let epub = EpubDoc::new("./shadows-of-self.epub").expect("Not found: Shadows of Self");
    IndexableBook {
        title: "Shadows of Self".to_string(),
        epub_file: epub,
        first_chapter_index: 7,
        last_chapter_index: 37,
        skippable_chapters: vec![8, 13, 31],
    }
}

pub fn build_alloy_of_law() -> IndexableBook {
    let epub = EpubDoc::new("./the-alloy-of-law.epub").expect("Not found: The Alloy of Law");
    IndexableBook {
        title: "The Alloy of Law".to_string(),
        epub_file: epub,
        first_chapter_index: 7,
        last_chapter_index: 32,
        skippable_chapters: vec![10, 16, 22, 26],
    }
}

pub fn build_secret_history() -> IndexableBook {
    let epub = EpubDoc::new("./secret-history.epub").expect("Not found: Secret History");
    IndexableBook {
        title: "Secret History".to_string(),
        epub_file: epub,
        first_chapter_index: 5,
        last_chapter_index: 35,
        skippable_chapters: vec![7, 12, 16, 21, 25],
    }
}
