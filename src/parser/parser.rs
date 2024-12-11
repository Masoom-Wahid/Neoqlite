use crate::{
    core::db::{ColumnType, Constraints},
    parser::lexer::{Keyword, Token},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Number(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub left: Expr,
    pub operator: String,
    pub right: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectQuery {
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteQuery {
    pub table: String,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertQuery {
    pub columns: Vec<String>,
    pub table: String,
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableQuery {
    pub table: String,
    pub columns: Vec<(String, ColumnType, Vec<Constraints>)>,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Insert(InsertQuery),
    Select(SelectQuery),
    Delete(DeleteQuery),
    CreateTable(CreateTableQuery),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect_peek_token(&mut self, token: Token) -> Result<(), String> {
        if let Some(ref curr_token) = self.peek_token() {
            println!(
                "curr_token is {:?} ,expected token is {:?}",
                curr_token, token
            );
            if **curr_token == token {
                Ok(())
            } else {
                Err(format!("Expected {:?}, but found {:?}", token, curr_token))
            }
        } else {
            Err(format!(
                "Expected {:?}, but found {:?}",
                token,
                self.current_token()
            ))
        }
    }

    fn expect_token(&mut self, token: Token) -> Result<(), String> {
        if let Some(ref curr_token) = self.current_token() {
            if **curr_token == token {
                self.advance();
                Ok(())
            } else {
                Err(format!("Expected {:?}, but found {:?}", token, curr_token))
            }
        } else {
            Err(format!(
                "Expected {:?}, but found {:?}",
                token,
                self.current_token()
            ))
        }
    }
    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), String> {
        if let Some(Token::Keyword(ref kw)) = self.current_token() {
            if *kw == keyword {
                self.advance();
                Ok(())
            } else {
                Err(format!("Expected {:?}, but found {:?}", keyword, kw))
            }
        } else {
            Err(format!(
                "Expected {:?}, but found {:?}",
                keyword,
                self.current_token()
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        if let Some(Token::Ident(ref id)) = self.current_token() {
            let identifier = id.clone();
            self.advance();
            Ok(identifier)
        } else {
            Err(format!(
                "Expected identifier, but found {:?}",
                self.current_token()
            ))
        }
    }

    fn parse_insert(&mut self) -> Result<InsertQuery, String> {
        self.expect_keyword(Keyword::Insert)?;
        self.expect_keyword(Keyword::Into)?;

        let table = self.expect_identifier()?;
        self.expect_token(Token::LParen)?;
        let mut columns = Vec::new();
        loop {
            columns.push(self.expect_identifier()?);
            if let Some(Token::Comma) = self.current_token() {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_token(Token::RParen)?;
        self.expect_keyword(Keyword::Values)?;
        self.expect_token(Token::LParen)?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_expr()?);
            if let Some(Token::Comma) = self.current_token() {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_token(Token::RParen)?;
        self.expect_token(Token::Semicolon)?;
        Ok(InsertQuery {
            columns,
            values,
            table,
        })
    }

    fn to_column_type(input: &str) -> Result<ColumnType, String> {
        match input.to_uppercase().as_str() {
            "INT" => Ok(ColumnType::Int),
            "TEXT" => Ok(ColumnType::String),
            _ => Err(format!("Invalid Column type , got {}", input)),
        }
    }

    fn parse_constraint(input: &str) -> Result<Constraints, String> {
        match input.to_uppercase().as_str() {
            "NOTNULL" => Ok(Constraints::NotNull),
            "NULL" => Ok(Constraints::Null),
            _ => Err(format!("Invalid constraint got {}", input)),
        }
    }

    fn parse_create_table(&mut self) -> Result<CreateTableQuery, String> {
        self.expect_keyword(Keyword::Create)?;
        self.expect_keyword(Keyword::Table)?;
        let table = self.expect_identifier()?;
        self.expect_token(Token::LParen)?;
        let mut columns: Vec<(String, ColumnType, Vec<Constraints>)> = Vec::new();
        loop {
            let column_name = self.expect_identifier()?;
            let column_type = Self::to_column_type(&self.expect_identifier()?)?;
            let mut constraints: Vec<Constraints> = Vec::new();
            println!("here");
            loop {
                println!("{:?}", self.current_token());
                if matches!(self.current_token(), Some(Token::Comma))
                    || matches!(self.current_token(), Some(Token::RParen))
                    || self.expect_peek_token(Token::RParen).is_ok()
                {
                    break;
                } else {
                    let constraint = Self::parse_constraint(&self.expect_identifier()?)?;
                    constraints.push(constraint);
                }
            }

            columns.push((column_name, column_type, constraints));
            println!("after {:?}", columns);

            if let Some(Token::Comma) = self.current_token() {
                /*
                 *   just in case if we have been in a place that after comma is a right paren
                 */
                match self.peek_token() {
                    Some(Token::RParen) => {
                        //self.advance();
                        break;
                    }
                    _ => {}
                }
                self.advance();
            } else {
                break;
            }
        }
        println!("columns : {:?}", columns);
        self.expect_token(Token::RParen)?;
        self.expect_token(Token::Semicolon)?;

        Ok(CreateTableQuery { table, columns })
    }

    pub fn parse(&mut self) -> Result<Query, String> {
        let result = match &mut self.tokens.first().unwrap() {
            Token::Keyword(Keyword::Select) => Query::Select(self.parse_select()?),
            Token::Keyword(Keyword::Insert) => Query::Insert(self.parse_insert()?),
            Token::Keyword(Keyword::Delete) => Query::Delete(self.parse_delete()?),
            Token::Keyword(Keyword::Create) => Query::CreateTable(self.parse_create_table()?),
            _ => return Err(format!("Invalid Query got {:?}", self.tokens.first())),
        };
        Ok(result)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let token = self.current_token().cloned();
        match token {
            Some(Token::Ident(id)) => {
                self.advance();
                Ok(Expr::Ident(id.to_string()))
            }
            Some(Token::Number(num)) => {
                self.advance();
                Ok(Expr::Number(num))
            }
            _ => Err(format!(
                "Expected expression, but found {:?}",
                self.current_token()
            )),
        }
    }

    pub fn parse_where_clause(&mut self) -> Option<WhereClause> {
        let does_have_where = self.expect_keyword(Keyword::Where);
        let where_clause = {
            if does_have_where.is_ok() {
                let left = self.parse_expr().ok()?;
                let _ = self.expect_token(Token::Operator("=".to_string())).ok()?;
                let right = self.parse_expr().ok()?;
                Some(WhereClause {
                    left,
                    operator: "=".to_string(),
                    right,
                })
            } else {
                None
            }
        };
        where_clause
    }

    pub fn parse_delete(&mut self) -> Result<DeleteQuery, String> {
        self.expect_keyword(Keyword::Delete)?;
        self.expect_keyword(Keyword::From)?;
        let table = self.expect_identifier()?;
        let where_clause = self.parse_where_clause();
        Ok(DeleteQuery {
            table,
            where_clause,
        })
    }

    pub fn parse_select(&mut self) -> Result<SelectQuery, String> {
        self.expect_keyword(Keyword::Select)?;

        let mut columns = Vec::new();
        loop {
            columns.push(self.expect_identifier()?);
            if let Some(Token::Comma) = self.current_token() {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_keyword(Keyword::From)?;

        let table = self.expect_identifier()?;

        //let where_clause = self.parse_where_clause()?;

        let where_clause = self.parse_where_clause();
        if let Some(Token::Semicolon) = self.current_token() {
            self.advance();
        }

        Ok(SelectQuery {
            columns,
            table,
            where_clause,
        })
    }
}
