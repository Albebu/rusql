#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    Select { table_name: String },
}

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Select,
    From,
    Star,
    Semicolon,
    Identifier(String),
}
