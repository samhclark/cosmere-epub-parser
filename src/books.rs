use crate::domain::IndexableBook;

pub fn load_all() -> Vec<IndexableBook> {
    vec![
        IndexableBook::from(
            String::from("The Alloy of Law"),
            "./the-alloy-of-law.epub",
            7,
            32,
            vec![10, 16, 22, 26],
        ),
        IndexableBook::from(
            String::from("Shadows of Self"),
            "./shadows-of-self.epub",
            7,
            37,
            vec![8, 13, 31],
        ),
        IndexableBook::from(
            String::from("The Bands of Mourning"),
            "./the-bands-of-mourning.epub",
            7,
            42,
            vec![8, 13, 26],
        ),
        IndexableBook::from(
            String::from("Secret History"),
            "./secret-history.epub",
            5,
            35,
            vec![7, 12, 16, 21, 25],
        ),
    ]
}
