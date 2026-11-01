pub(crate) enum Statement {
    Select { table_name: String },
}

pub(crate) enum Token {
    Select,
    From,
    Star,
    Semicolon,
    Identifier(String),
}
