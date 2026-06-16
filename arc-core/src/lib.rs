use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;
use std::fmt;

// 声明 codegen 模块
pub mod codegen;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("let")]
    Let,
    #[token("print")]
    Print,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),
    
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::Assign => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(i64),
    StringLit(String),
    Identifier(String),
    Print(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

pub fn parser() -> impl Parser<Token, Vec<Expr>, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let num = select! { Token::Number(n) => Expr::Number(n) };
        let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
        let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };

        let print_expr = just(Token::Print)
            .ignore_then(just(Token::LParen))
            .ignore_then(expr.clone())
            .then_ignore(just(Token::RParen))
            .map(|e| Expr::Print(Box::new(e)));

        print_expr.or(num).or(str_lit).or(ident)
    });

    expr.repeated().then_ignore(end())
}

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => {
                self.variables.get(name).cloned().ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::Print(e) => {
                let val = self.eval(e)?;
                println!("Arc Output > {}", val);
                Ok(val)
            }
        }
    }
}