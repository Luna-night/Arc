use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;
use std::fmt;

pub mod codegen;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("let")] Let,
    #[token("print")] Print,
    #[token("bridge")] Bridge,
    #[token("func")] Func,
    #[token("->")] RArrow,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token(",")] Comma,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),
    
    #[token("=")] Assign,
    #[token(";")] Semicolon,
    #[token("(")] LParen,
    #[token(")")] RParen,
    
    #[token("Int")] TypeInt,
    #[token("Float")] TypeFloat,
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
            Token::Comma => write!(f, ","),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::Assign => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
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
    Call(String, Vec<Expr>), 
}

#[derive(Debug, PartialEq, Clone)]
pub enum BridgeParam {
    Param { name: String, ty: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    BridgeDecl {
        lang: String,
        lib: String,
        name: String,
        params: Vec<BridgeParam>,
        ret_ty: String,
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

        // 【核心修复】放弃 then_ignore，全部用 .then 保留，然后在 .map 里用 _ 丢弃！
        let call = select! { Token::Identifier(name) => name }
            .then(just(Token::LParen))
            .then(expr.clone().separated_by(just(Token::Comma)).allow_trailing())
            .then(just(Token::RParen))
            .map(|(((name, _lp), args), _rp)| Expr::Call(name, args));

        let print_expr = just(Token::Print)
            .then(just(Token::LParen))
            .then(expr.clone())
            .then(just(Token::RParen))
            .map(|(((_print, _lp), e), _rp)| Expr::Print(Box::new(e)));

        print_expr.or(call).or(num).or(str_lit).or(ident)
    });

    let param = select! { Token::Identifier(name) => name }
        .then(just(Token::Assign))
        .then(select! { Token::TypeInt => "Int".to_string(), Token::TypeFloat => "Float".to_string() })
        .map(|((name, _eq), ty)| BridgeParam::Param { name, ty });

    let params_list = param
        .separated_by(just(Token::Comma))
        .allow_trailing();

    // 【核心修复】同样使用 .then 和 .map 手动解构，彻底避免类型推导错误
    let bridge_decl = just(Token::Bridge)
        .then(select! { Token::Identifier(lang) => lang })
        .then(select! { Token::StringLit(lib) => lib })
        .then(just(Token::LBrace))
        .then(just(Token::Func))
        .then(select! { Token::Identifier(name) => name })
        .then(just(Token::LParen))
        .then(params_list)
        .then(just(Token::RParen))
        .then(just(Token::RArrow))
        .then(select! { Token::TypeInt => "Int".to_string(), Token::TypeFloat => "Float".to_string() })
        .then(just(Token::RBrace))
        .map(|(((((((((((_bridge, lang), lib), _lb), _func), name), _lp), params), _rp), _ra), ret_ty), _rb)| {
            TopLevel::BridgeDecl { lang, lib, name, params, ret_ty }
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
            Expr::Call(name, _) => Err(format!("Bridge function '{}' cannot be called in interpreter mode.", name)),
            Expr::Print(e) => {
                let val = self.eval(e)?;
                println!("Arc Output > {}", val);
                Ok(val)
            }
        }
    }
}