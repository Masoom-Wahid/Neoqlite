#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Keyword(Keyword),
    Operator(String),
    Number(i64),
    StringLiteral(String),
    Comma,
    Semicolon,
    LParen,
    RParen,
    Whitespace,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Select,
    From,
    Where,
    Insert,
    Table,
    Into,
    Values,
    Update,
    Create,
    Set,
    Delete,
    And,
    Or,
    Not,
    In,
    Like,
    Join,
    On,
    As,
    GroupBy,
    OrderBy,
    Limit,
}

/*
*   let mut lexer = Lexer::new(input);
*   println!("lexer : {:?}",lexer);
*
*
*   CREATE TABLE users(
*       id int,
*       username text,
*       email text,
*   );
*
*/

pub struct Lexer {
    input: String,
}

impl Lexer {
    fn classify_token(token: &str) -> Token {
        match token.to_uppercase().as_str() {
            "SELECT" => Token::Keyword(Keyword::Select),
            "FROM" => Token::Keyword(Keyword::From),
            "WHERE" => Token::Keyword(Keyword::Where),
            "INSERT" => Token::Keyword(Keyword::Insert),
            "UPDATE" => Token::Keyword(Keyword::Update),
            "DELETE" => Token::Keyword(Keyword::Delete),
            "CREATE" => Token::Keyword(Keyword::Create),
            "AND" => Token::Keyword(Keyword::And),
            "OR" => Token::Keyword(Keyword::Or),
            "NOT" => Token::Keyword(Keyword::Not),
            "IN" => Token::Keyword(Keyword::In),
            "LIKE" => Token::Keyword(Keyword::Like),
            "JOIN" => Token::Keyword(Keyword::Join),
            "ON" => Token::Keyword(Keyword::On),
            "AS" => Token::Keyword(Keyword::As),
            "INTO" => Token::Keyword(Keyword::Into),
            "GROUP" => Token::Keyword(Keyword::GroupBy),
            "ORDER" => Token::Keyword(Keyword::OrderBy),
            "LIMIT" => Token::Keyword(Keyword::Limit),
            "VALUES" => Token::Keyword(Keyword::Values),
            "TABLE" => Token::Keyword(Keyword::Table),
            _ => Token::Ident(token.to_string()),
        }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = self.input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' | '*' => {
                    current.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_alphanumeric() || next == '_' {
                            current.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    tokens.push(Self::classify_token(&current));
                    current.clear();
                }
                '0'..='9' => {
                    current.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_digit(10) {
                            current.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    if let Ok(num) = current.parse::<i64>() {
                        tokens.push(Token::Number(num));
                    }

                    current.clear();
                }

                '\'' | '\"' => {
                    while let Some(&next) = chars.peek() {
                        if next == '\'' || next == '\"' {
                            chars.next().unwrap();
                            break;
                        } else {
                            current.push(chars.next().unwrap());
                        }
                    }

                    tokens.push(Token::Ident(current.clone()));
                    current.clear();
                }

                '=' | '>' | '<' | '!' => {
                    current.push(c);
                    if let Some(&next) = chars.peek() {
                        if next == '=' {
                            current.push(chars.next().unwrap());
                        }
                    }

                    tokens.push(Token::Operator(current.clone()));
                    current.clear();
                }

                ',' => tokens.push(Token::Comma),
                ';' => tokens.push(Token::Semicolon),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                _ => println!("Invalid token {}", c),
            }
        }

        tokens
    }

    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
        }
    }
}
