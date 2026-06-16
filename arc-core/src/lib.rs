use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;
use std::fmt;

pub mod codegen;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("let")]
    Let,
    #[token("print")]
    Print,
    #[token("bridge")]
    Bridge,
    #[token("func")]
    Func,
    #[token("->")]
    RArrow,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    
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
    #[token("py")]
    Py,
    #[token("Int")]
    TypeInt,
    #[token("Float")]
    TypeFloat,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::Bridge => write!(f, "bridge"),
            Token::Func => write!(f, "func"),
            Token::RArrow => write!(f, "->"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::Assign => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Py => write!(f, "py"),
            Token::TypeInt => write!(f, "Int"),
            Token::TypeFloat => write!(f, "Float"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(i64),
    StringLit(String),
    Identifier(String),
    Print(Box<Expr>),
    Call(String), 
}

#[derive(Debug, PartialEq, Clone)]
pub enum BridgeParam {
    Param { name: String, ty: String }, // 例如: Param { name: "x", ty: "Float" }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    BridgeDecl {
        lang: String,  // "c" 或 "py"
        lib: String,   // 例如 "math"
        name: String,  // 函数名
        params: Vec<BridgeParam>,
        ret_ty: String, // "Int" 或 "Float"
    },
    Statement(Expr),
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

pub fn parser() -> impl Parser<Token, Vec<TopLevel>, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let num = select! { Token::Number(n) => Expr::Number(n) };
        let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
        let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };

        let call = select! { Token::Identifier(name) => name }
            .then_ignore(just(Token::LParen))
            .ignore_then(just(Token::RParen))
            .map(|name| Expr::Call(name));

        let print_expr = just(Token::Print)
            .ignore_then(just(Token::LParen))
            .ignore_then(expr.clone())
            .then_ignore(just(Token::RParen))
            .map(|e| Expr::Print(Box::new(e)));

        print_expr.or(call).or(num).or(str_lit).or(ident)
    });

    // 解析单个参数
    let param = select! { Token::Identifier(name) => name }
        .then_ignore(just(Token::Assign))
        .ignore_then(select! { Token::TypeInt => "Int".to_string(), Token::TypeFloat => "Float".to_string() })
        .map(|(name, ty)| BridgeParam::Param { name, ty });

    // 解析参数列表
    let params_list = param
        .separated_by(just(Token::Assign))
        .allow_trailing();

    // 解析 Bridge 声明
    let bridge_decl = just(Token::Bridge)
        .ignore_then(select! { Token::Identifier(lang) => lang })
        .then(select! { Token::StringLit(lib) => lib })
        .ignore_then(just(Token::LBrace))
        .ignore_then(just(Token::Func))
        .then(select! { Token::Identifier(name) => name })
        .ignore_then(just(Token::LParen))
        .then(params_list)
        .ignore_then(just(Token::RParen))
        .ignore_then(just(Token::RArrow))
        .ignore_then(select! { Token::TypeInt => "Int".to_string(), Token::TypeFloat => "Float".to_string() })
        .ignore_then(just(Token::RBrace))
        .map(|((((lang, lib), name), params), ret_ty)| {
            TopLevel::BridgeDecl {
                lang,
                lib,
                name,
                params,
                ret_ty,
            }
        });

    let statement = expr.map(TopLevel::Statement);

    bridge_decl.or(statement)
        .repeated()
        .then_ignore(end())
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
            Expr::Call(name) => Err(format!("Bridge function '{}' cannot be called in interpreter mode. Please use 'build'.", name)),
            Expr::Print(e) => {
                let val = self.eval(e)?;
                println!("Arc Output > {}", val);
                Ok(val)
            }
        }
    }
}