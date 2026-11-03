use crate::column::Column;

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    Select {
        table_name: String,
    },
    CreateTable {
        table_name: String,
        columns: Vec<Column>,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Select,
    From,
    Star,
    Semicolon,
    Identifier(String),
    Create,
    Table,
    Text,
    Number,
    LeftParen,
    RightParen,
    Comma,
}
