use crate::{column::Column, row::Row};

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    Select {
        table_name: String,
    },
    CreateTable {
        table_name: String,
        columns: Vec<Column>,
    },
    InsertInto {
        table_name: String,
        rows: Vec<Row>,
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
    Insert,
    Into,
    Values,
    TextLiteral(String),
    NumberLiteral(i64),
}
