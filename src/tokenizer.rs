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
