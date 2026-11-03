use crate::{
    column::{Column, ColumnDataType},
    row::{Row, RowValue},
    statement::{Statement, Token},
};

#[derive(Debug, PartialEq)]
pub(crate) enum ParseSqlErrors {
    EmptyStatement,
    MissingSelectStatement,
    MissingStartStatement,
    MissinFromStatement,
    MissingTable,
    UnexpectedStatement,
    ExpectedTableKeyword,
    ExpectedTableName,
    ExpectedOpeningParen,
    ExpectedColumnName,
    ExpectedColumnType,
    ExpectedCommaOrClosingParen,
    ExpectedIntoKeyword,
    ExpectedValuesKeyword,
    ExpectedValue,
}

pub(crate) fn parse_sql(tokens: &[Token]) -> Result<Statement, ParseSqlErrors> {
    if tokens.is_empty() {
        return Err(ParseSqlErrors::EmptyStatement);
    }

    match tokens.first() {
        Some(Token::Create) => parse_create_table(&tokens[1..]),
        Some(Token::Select) => parse_select(&tokens[1..]),
        Some(Token::Insert) => parse_insert(&tokens[1..]),
        _ => Err(ParseSqlErrors::MissingSelectStatement),
    }
}

fn parse_insert(tokens: &[Token]) -> Result<Statement, ParseSqlErrors> {
    let mut tokens = tokens.iter();

    if tokens.next() != Some(&Token::Into) {
        return Err(ParseSqlErrors::ExpectedIntoKeyword);
    }
    let table_name = match tokens.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(ParseSqlErrors::ExpectedTableName),
    };
    if tokens.next() != Some(&Token::Values) {
        return Err(ParseSqlErrors::ExpectedValuesKeyword);
    }
    if tokens.next() != Some(&Token::LeftParen) {
        return Err(ParseSqlErrors::ExpectedOpeningParen);
    }

    let mut values = Vec::new();
    loop {
        let value = match tokens.next() {
            Some(Token::TextLiteral(value)) => RowValue::Text(value.clone()),
            Some(Token::NumberLiteral(value)) => RowValue::Number(*value),
            _ => return Err(ParseSqlErrors::ExpectedValue),
        };
        values.push(value);

        match tokens.next() {
            Some(Token::Comma) => {}
            Some(Token::RightParen) => break,
            _ => return Err(ParseSqlErrors::ExpectedCommaOrClosingParen),
        }
    }

    match (tokens.next(), tokens.next()) {
        (None, _) | (Some(Token::Semicolon), None) => Ok(Statement::InsertInto {
            table_name,
            rows: vec![Row { values }],
        }),
        _ => Err(ParseSqlErrors::UnexpectedStatement),
    }
}

fn parse_select(tokens: &[Token]) -> Result<Statement, ParseSqlErrors> {
    let mut tokens = tokens.iter();

    if tokens.next() != Some(&Token::Star) {
        return Err(ParseSqlErrors::MissingStartStatement);
    }

    if tokens.next() != Some(&Token::From) {
        return Err(ParseSqlErrors::MissinFromStatement);
    }

    let table_name = match tokens.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(ParseSqlErrors::MissingTable),
    };

    let next_token = tokens.next();

    if next_token != Some(&Token::Semicolon) && next_token != None {
        return Err(ParseSqlErrors::UnexpectedStatement);
    }

    if next_token == Some(&Token::Semicolon) && tokens.next() != None {
        return Err(ParseSqlErrors::UnexpectedStatement);
    }

    Ok(Statement::Select { table_name })
}

fn parse_create_table(tokens: &[Token]) -> Result<Statement, ParseSqlErrors> {
    let mut tokens = tokens.iter();

    if tokens.next() != Some(&Token::Table) {
        return Err(ParseSqlErrors::ExpectedTableKeyword);
    }
    let table_name = match tokens.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(ParseSqlErrors::ExpectedTableName),
    };
    if tokens.next() != Some(&Token::LeftParen) {
        return Err(ParseSqlErrors::ExpectedOpeningParen);
    }

    let mut columns = Vec::new();
    loop {
        let name = match tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err(ParseSqlErrors::ExpectedColumnName),
        };
        let data_type = match tokens.next() {
            Some(Token::Text) => ColumnDataType::Text,
            Some(Token::Number) => ColumnDataType::Number,
            _ => return Err(ParseSqlErrors::ExpectedColumnType),
        };
        columns.push(Column { name, data_type });

        match tokens.next() {
            Some(Token::Comma) => {}
            Some(Token::RightParen) => break,
            _ => return Err(ParseSqlErrors::ExpectedCommaOrClosingParen),
        }
    }

    match (tokens.next(), tokens.next()) {
        (None, _) | (Some(Token::Semicolon), None) => Ok(Statement::CreateTable {
            table_name,
            columns,
        }),
        _ => Err(ParseSqlErrors::UnexpectedStatement),
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        parser::parse_sql,
        statement::{
            Statement,
            Token::{From, Identifier, Select, Semicolon, Star},
        },
        tokenizer::tokenize,
    };

    #[test]
    fn parser_returns_correct_sql_structure() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert_eq!(
            Statement::Select {
                table_name: String::from("users")
            },
            result.unwrap()
        );
    }

    #[test]
    fn parser_returns_correct_table_for_another_identifier() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("orders")),
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert_eq!(
            Statement::Select {
                table_name: String::from("orders")
            },
            result.unwrap()
        );
    }

    #[test]
    fn parser_empty_input() {
        let tokens = [];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_incomplete_statement() {
        let tokens = [Select, Star];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_unexpected_multiple_semicolons() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_missing_select_statement() {
        let tokens = [Star, From, Identifier(String::from("users")), Semicolon];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_missing_star_statement() {
        let tokens = [Select, From, Identifier(String::from("users")), Semicolon];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_missing_from_statement() {
        let tokens = [Select, Star, Identifier(String::from("users")), Semicolon];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_accepts_missing_semicolon() {
        let tokens = [Select, Star, From, Identifier(String::from("users"))];

        let result = parse_sql(&tokens);

        assert_eq!(
            Statement::Select {
                table_name: String::from("users")
            },
            result.unwrap()
        );
    }

    #[test]
    fn parser_missing_identifier() {
        let tokens = [Select, Star, From, Semicolon];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_unexpected_extra_token() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Identifier(String::from("extra")),
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_unexpected_trailing_token() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
            Identifier(String::from("extra")),
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_unexpected_multiple_statements() {
        let tokens = [
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
            Select,
            Star,
            From,
            Identifier(String::from("orders")),
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_unexpected_sql_order() {
        let tokens = [
            From,
            Identifier(String::from("users")),
            Select,
            Star,
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn parser_rejects_specific_column() {
        let tokens = [
            Select,
            Identifier(String::from("name")),
            From,
            Identifier(String::from("users")),
            Semicolon,
        ];

        let result = parse_sql(&tokens);

        assert!(result.is_err());
    }

    #[test]
    fn tokenizer_and_parser_work_together() {
        let tokens = tokenize("SELECT * FROM users;").unwrap();

        let result = parse_sql(&tokens);

        assert_eq!(
            Statement::Select {
                table_name: String::from("users")
            },
            result.unwrap()
        );
    }

    #[test]
    fn parser_create_table_preserves_column_names_types_and_order() {
        use crate::column::{Column, ColumnDataType};

        let tokens = tokenize("create table Users(id NUMBER,Name TEXT,age NUMBER)").unwrap();
        let statement = parse_sql(&tokens).unwrap();

        assert_eq!(
            statement,
            Statement::CreateTable {
                table_name: String::from("Users"),
                columns: vec![
                    Column {
                        name: String::from("id"),
                        data_type: ColumnDataType::Number
                    },
                    Column {
                        name: String::from("Name"),
                        data_type: ColumnDataType::Text
                    },
                    Column {
                        name: String::from("age"),
                        data_type: ColumnDataType::Number
                    },
                ],
            }
        );
    }

    #[test]
    fn parser_rejects_invalid_create_table_statements() {
        for sql in [
            "CREATE",
            "CREATE users (id NUMBER)",
            "CREATE TABLE (id NUMBER)",
            "CREATE TABLE users id NUMBER)",
            "CREATE TABLE users ()",
            "CREATE TABLE users (id)",
            "CREATE TABLE users (id BOOLEAN)",
            "CREATE TABLE users (id NUMBER,)",
            "CREATE TABLE users (,id NUMBER)",
            "CREATE TABLE users (id NUMBER name TEXT)",
            "CREATE TABLE users (id NUMBER",
            "CREATE TABLE users (id NUMBER NOT NULL)",
            "CREATE TABLE users (id NUMBER PRIMARY KEY)",
            "CREATE TABLE users (id NUMBER);;",
            "CREATE TABLE users (id NUMBER); extra",
            "CREATE TABLE users (id NUMBER); SELECT * FROM users",
            "CREATE TABLE SELECT (id NUMBER)",
            "CREATE TABLE users (FROM TEXT)",
        ] {
            let tokens = tokenize(sql).unwrap();
            assert!(parse_sql(&tokens).is_err(), "accepted {sql}");
        }
    }

    #[test]
    fn parser_insert_preserves_literal_values_and_order() {
        use crate::row::{Row, RowValue};

        for sql in [
            "INSERT INTO users VALUES (1, 'O''Brien', -2, '')",
            "insert into users values (1, 'O''Brien', -2, '');",
        ] {
            let tokens = tokenize(sql).unwrap();
            assert_eq!(
                parse_sql(&tokens),
                Ok(Statement::InsertInto {
                    table_name: String::from("users"),
                    rows: vec![Row {
                        values: vec![
                            RowValue::Number(1),
                            RowValue::Text(String::from("O'Brien")),
                            RowValue::Number(-2),
                            RowValue::Text(String::new()),
                        ],
                    }],
                })
            );
        }
    }

    #[test]
    fn parser_rejects_invalid_insert_statements() {
        for sql in [
            "INSERT",
            "INSERT users VALUES (1)",
            "INSERT INTO",
            "INSERT INTO VALUES (1)",
            "INSERT INTO users",
            "INSERT INTO users (1)",
            "INSERT INTO users VALUES",
            "INSERT INTO users VALUES ()",
            "INSERT INTO users VALUES (Ada)",
            "INSERT INTO users VALUES (,1)",
            "INSERT INTO users VALUES (1,)",
            "INSERT INTO users VALUES (1 2)",
            "INSERT INTO users VALUES (1",
            "INSERT INTO users VALUES (1), (2)",
            "INSERT INTO users (id) VALUES (1)",
            "INSERT INTO users VALUES (1);;",
            "INSERT INTO users VALUES (1); SELECT * FROM users",
        ] {
            let tokens = tokenize(sql).unwrap();
            assert!(parse_sql(&tokens).is_err(), "accepted {sql}");
        }
    }

    #[test]
    fn parser_reports_insert_order_errors() {
        use crate::parser::ParseSqlErrors;

        for (sql, expected) in [
            (
                "INSERT users VALUES (1)",
                ParseSqlErrors::ExpectedIntoKeyword,
            ),
            ("INSERT INTO VALUES (1)", ParseSqlErrors::ExpectedTableName),
            (
                "INSERT INTO users (1)",
                ParseSqlErrors::ExpectedValuesKeyword,
            ),
            (
                "INSERT INTO users VALUES 1",
                ParseSqlErrors::ExpectedOpeningParen,
            ),
            ("INSERT INTO users VALUES ()", ParseSqlErrors::ExpectedValue),
            (
                "INSERT INTO users VALUES (1 2)",
                ParseSqlErrors::ExpectedCommaOrClosingParen,
            ),
        ] {
            assert_eq!(parse_sql(&tokenize(sql).unwrap()), Err(expected), "{sql}");
        }
    }
}
