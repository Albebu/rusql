use crate::statement::Token;

pub(crate) fn tokenize(query: &str) -> Vec<Token> {
    let mut word = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    for character in query.chars() {
        if character.is_whitespace() || character == '*' || character == ';' {
            tokenize_word(&mut word, &mut tokens);
            match character {
                '*' => tokens.push(Token::Star),
                ';' => tokens.push(Token::Semicolon),
                _ => {}
            }
        } else {
            word.push(character);
        }
    }

    tokenize_word(&mut word, &mut tokens);
    tokens
}

fn tokenize_word(word: &mut String, tokens: &mut Vec<Token>) {
    if word.is_empty() {
        return;
    }

    match word.to_uppercase().as_str() {
        "SELECT" => tokens.push(Token::Select),
        "FROM" => tokens.push(Token::From),
        _ => tokens.push(Token::Identifier(word.clone())),
    }
    word.clear();
}

#[cfg(test)]
mod tests {
    use super::tokenize;
    use crate::statement::Token::{From, Identifier, Select, Semicolon, Star};

    #[test]
    fn tokenizer_correctly_tokens_output() {
        let sql_sentence = "SELECT * FROM users;";
        let result = tokenize(sql_sentence);

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
        let result = tokenize(sql_sentence);

        let expected = vec![Select];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_missing_star() {
        let result = tokenize("SELECT FROM users;");

        let expected = vec![Select, From, Identifier(String::from("users")), Semicolon];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_missing_table() {
        let result = tokenize("SELECT * FROM;");

        let expected = vec![Select, Star, From, Semicolon];

        assert_eq!(result, expected);
    }

    #[test]
    fn tokenizer_specific_column() {
        let result = tokenize("SELECT name FROM users;");

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
        let result = tokenize("SELECT * FROM users extra;");

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
        let result = tokenize("SELECT * FROM users;;");

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
}
