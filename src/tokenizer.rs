use crate::statement::Token;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenizeError {
    InvalidIdentifier(String),
    UnterminatedString,
    IntegerOutOfRange(String),
}

pub(crate) fn tokenize(query: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut word = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    let mut characters = query.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' {
            tokenize_word(&mut word, &mut tokens)?;
            let mut value = String::new();
            loop {
                match characters.next() {
                    Some('\'') if characters.peek() == Some(&'\'') => {
                        characters.next();
                        value.push('\'');
                    }
                    Some('\'') => break,
                    Some(character) => value.push(character),
                    None => return Err(TokenizeError::UnterminatedString),
                }
            }
            tokens.push(Token::TextLiteral(value));
        } else if character.is_whitespace() || matches!(character, '*' | ';' | '(' | ')' | ',') {
            tokenize_word(&mut word, &mut tokens)?;
            match character {
                '*' => tokens.push(Token::Star),
                ';' => tokens.push(Token::Semicolon),
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                ',' => tokens.push(Token::Comma),
                _ => {}
            }
        } else {
            word.push(character);
        }
    }

    tokenize_word(&mut word, &mut tokens)?;
    Ok(tokens)
}

fn tokenize_word(word: &mut String, tokens: &mut Vec<Token>) -> Result<(), TokenizeError> {
    if word.is_empty() {
        return Ok(());
    }

    if word.eq_ignore_ascii_case("SELECT") {
        tokens.push(Token::Select);
    } else if word.eq_ignore_ascii_case("FROM") {
        tokens.push(Token::From);
    } else if word.eq_ignore_ascii_case("CREATE") {
        tokens.push(Token::Create);
    } else if word.eq_ignore_ascii_case("TABLE") {
        tokens.push(Token::Table);
    } else if word.eq_ignore_ascii_case("TEXT") {
        tokens.push(Token::Text);
    } else if word.eq_ignore_ascii_case("NUMBER") {
        tokens.push(Token::Number);
    } else if word.eq_ignore_ascii_case("INSERT") {
        tokens.push(Token::Insert)
    } else if word.eq_ignore_ascii_case("INTO") {
        tokens.push(Token::Into);
    } else if word.eq_ignore_ascii_case("VALUES") {
        tokens.push(Token::Values);
    } else if {
        let digits = word.strip_prefix('-').unwrap_or(word);
        !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit())
    } {
        let number = word
            .parse::<i64>()
            .map_err(|_| TokenizeError::IntegerOutOfRange(word.clone()))?;
        tokens.push(Token::NumberLiteral(number));
    } else {
        let mut characters = word.chars();
        let first = characters.next().expect("word is not empty");
        if !(first.is_ascii_alphabetic() || first == '_')
            || !characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(TokenizeError::InvalidIdentifier(word.clone()));
        }
        tokens.push(Token::Identifier(word.clone()));
    }
    word.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TokenizeError, tokenize};
    use crate::statement::Token::{From, Identifier, Select, Semicolon, Star};

    #[test]
    fn tokenizer_correctly_tokens_output() {
        let sql_sentence = "SELECT * FROM users;";
        let result = tokenize(sql_sentence).unwrap();

        let expected = vec![
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_incomplete_statement() {
        let sql_sentence = "SELECT";
        let result = tokenize(sql_sentence).unwrap();

        let expected = vec![Select];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_missing_star() {
        let result = tokenize("SELECT FROM users;").unwrap();

        let expected = vec![Select, From, Identifier(String::from("users")), Semicolon];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_missing_table() {
        let result = tokenize("SELECT * FROM;").unwrap();

        let expected = vec![Select, Star, From, Semicolon];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_specific_column() {
        let result = tokenize("SELECT name FROM users;").unwrap();

        let expected = vec![
            Select,
            Identifier(String::from("name")),
            From,
            Identifier(String::from("users")),
            Semicolon,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_unexpected_trailing_output() {
        let result = tokenize("SELECT * FROM users extra;").unwrap();

        let expected = vec![
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Identifier(String::from("extra")),
            Semicolon,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_unexpected_second_semicolon() {
        let result = tokenize("SELECT * FROM users;;").unwrap();

        let expected = vec![
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
            Semicolon,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_multiple_white_spaces() {
        let result = tokenize("SELECT            * FROM users           ;").unwrap();
        let expected = vec![
            Select,
            Star,
            From,
            Identifier(String::from("users")),
            Semicolon,
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_accepts_valid_identifiers_and_mixed_case_keywords() {
        for name in ["users", "_users2", "A1_b", "_"] {
            let result = tokenize(&format!("sElEcT*FrOm {name};")).unwrap();
            assert_eq!(
                result,
                vec![
                    Select,
                    Star,
                    From,
                    Identifier(String::from(name)),
                    Semicolon
                ]
            );
        }
    }

    #[test]
    fn tokenizer_rejects_invalid_identifiers() {
        for name in ["2users", "users!", "user-name", "éusers", "ſelect"] {
            let result = tokenize(&format!("SELECT * FROM {name};"));
            assert_eq!(
                result,
                Err(TokenizeError::InvalidIdentifier(name.to_string()))
            );
        }
    }

    #[test]
    fn tokenizer_rejects_invalid_identifier_at_end_of_input() {
        assert_eq!(
            tokenize("SELECT * FROM 2users"),
            Err(TokenizeError::InvalidIdentifier(String::from("2users")))
        );
    }

    #[test]
    fn tokenizer_accepts_empty_input() {
        assert_eq!(tokenize(""), Ok(vec![]));
    }

    #[test]
    fn tokenizer_recognizes_create_table_and_adjacent_punctuation() {
        use crate::statement::Token::{Comma, Create, LeftParen, Number, RightParen, Table, Text};

        assert_eq!(
            tokenize("create table users(id number,name TEXT);").unwrap(),
            vec![
                Create,
                Table,
                Identifier(String::from("users")),
                LeftParen,
                Identifier(String::from("id")),
                Number,
                Comma,
                Identifier(String::from("name")),
                Text,
                RightParen,
                Semicolon,
            ]
        );
    }

    #[test]
    fn tokenizer_recognizes_insert_literals_and_keywords() {
        use crate::statement::Token::{
            Comma, Insert, Into, LeftParen, NumberLiteral, RightParen, TextLiteral, Values,
        };

        assert_eq!(
            tokenize("iNsErT InTo users VaLuEs(-1,'O''Brien, Ada', '', 9223372036854775807, -9223372036854775808);").unwrap(),
            vec![
                Insert,
                Into,
                Identifier(String::from("users")),
                Values,
                LeftParen,
                NumberLiteral(-1),
                Comma,
                TextLiteral(String::from("O'Brien, Ada")),
                Comma,
                TextLiteral(String::new()),
                Comma,
                NumberLiteral(i64::MAX),
                Comma,
                NumberLiteral(i64::MIN),
                RightParen,
                Semicolon,
            ]
        );
    }

    #[test]
    fn tokenizer_rejects_invalid_literals() {
        assert_eq!(
            tokenize("'unfinished"),
            Err(TokenizeError::UnterminatedString)
        );
        assert_eq!(
            tokenize("'escaped''"),
            Err(TokenizeError::UnterminatedString)
        );
        for number in ["9223372036854775808", "-9223372036854775809"] {
            assert_eq!(
                tokenize(number),
                Err(TokenizeError::IntegerOutOfRange(number.to_string()))
            );
        }
        for value in ["-", "+1", "1.5", "-1x"] {
            assert_eq!(
                tokenize(value),
                Err(TokenizeError::InvalidIdentifier(value.to_string()))
            );
        }
    }
}
