pub mod codegen; 

use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;

// 【关键修复 1】必须添加 Eq 和 Hash，这是 chumsky 的强制要求
#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("let")]
    Let,
    #[token("print")]
    Print,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // 【关键修复 2】必须显式指定解析为 i64 类型
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    // 【新增】匹配双引号内的字符串，并去掉首尾的引号
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

// 引入 fmt 模块
use std::fmt;

// 为 Token 实现 Display trait，让 chumsky 的错误信息能正常打印
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Number(n) => write!(f, "number {}", n),
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
// 让 Value 能够被打印
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}
// 使用 chumsky 的 recursive 来构建解析器，这是处理表达式最标准的方法
pub fn parser() -> impl Parser<Token, Vec<Expr>, Error = Simple<Token>> {
    // 定义递归的表达式解析器
    let expr = recursive(|expr| {
        let num = select! { Token::Number(n) => Expr::Number(n) };
        let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };

        // 解析 print(...)
        let print_expr = just(Token::Print)
            .ignore_then(just(Token::LParen))
            .ignore_then(expr.clone()) // 注意这里加了 .clone()
            .then_ignore(just(Token::RParen))
            .map(|e| Expr::Print(Box::new(e)));

          // 【修改】加入 str_lit
        print_expr.or(num).or(str_lit).or(ident)
    });

    // 解析由多个表达式组成的序列，直到文件结束
    expr.repeated().then_ignore(end())
}

// 解释器环境
pub struct Environment {
    variables: HashMap<String, Value>, // 【修改】改为 Value
}

impl Environment {
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    // 【修改】返回 Result<Value, String>
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLit(s) => Ok(Value::String(s.clone())), // 【新增】
            Expr::Identifier(name) => {
                self.variables.get(name).cloned().ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::Print(e) => {
                let val = self.eval(e)?;
                println!("Arc Output > {}", val); // 利用 Display trait 打印
                Ok(val)
            }
        }
    }
}