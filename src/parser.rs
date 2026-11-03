use crate::statement::{
    Statement,
    Token::{self},
};

#[derive(Debug, PartialEq)]
pub(crate) enum ParseSqlErrors {
    EmptyStatement,
    MissingSelectStatement,
    MissingStartStatement,
    MissinFromStatement,
    MissingTable,
    UnexpectedStatement,
}

pub(crate) fn parse_sql(tokens: &[Token]) -> Result<Statement, ParseSqlErrors> {
    if tokens.is_empty() {
        return Err(ParseSqlErrors::EmptyStatement);
    }

    let mut tokens = tokens.iter();

    if tokens.next() != Some(&Token::Select) {
        return Err(ParseSqlErrors::MissingSelectStatement);
    }

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
        let tokens = tokenize("SELECT * FROM users;");

        let result = parse_sql(&tokens);

        assert_eq!(
            Statement::Select {
                table_name: String::from("users")
            },
            result.unwrap()
        );
    }
}
