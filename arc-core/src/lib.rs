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
    #[token(";")] Semicolon,
    #[token("=")] Assign,
    #[token("(")] LParen,
    #[token(")")] RParen,
    
    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),
    
    #[token("Int")] TypeInt,
    #[token("Float")] TypeFloat,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"), Token::Print => write!(f, "print"),
            Token::Bridge => write!(f, "bridge"), Token::Func => write!(f, "func"),
            Token::RArrow => write!(f, "->"), Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"), Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"), Token::Assign => write!(f, "="),
            Token::LParen => write!(f, "("), Token::RParen => write!(f, ")"),
            Token::Add => write!(f, "+"), Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"), Token::Div => write!(f, "/"),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::TypeInt => write!(f, "Int"), Token::TypeFloat => write!(f, "Float"),
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
    BinOp(Box<Expr>, String, Box<Expr>), 
}

#[derive(Debug, PartialEq, Clone)]
pub enum BridgeParam {
    Param { name: String, ty: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    BridgeDecl {
        lang: String, lib: String, name: String,
        params: Vec<BridgeParam>, ret_ty: String,
    },
    LetDecl { name: String, value: Box<Expr> },
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
    // 【核心修复】利用 recursive 闭包，让 print 能够引用完整的 expr
    let expr = recursive(|expr| {
        
        // 1. 原子表达式 (注意：这里不再包含 print！)
        let atom = {
            let num = select! { Token::Number(n) => Expr::Number(n) };
            let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
            let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };

            let call = select! { Token::Identifier(name) => name }
                .then(just(Token::LParen))
                .then(expr.clone().separated_by(just(Token::Comma)).allow_trailing()) // 函数参数也是完整的 expr
                .then(just(Token::RParen))
                .map(|(((name, _lp), args), _rp)| Expr::Call(name, args));

            let paren = just(Token::LParen)
                .ignore_then(expr.clone()) // 括号里也是完整的 expr
                .then_ignore(just(Token::RParen));

            num.or(str_lit).or(ident).or(call).or(paren)
        };

        // 2. 乘除 (高优先级)
        let term = atom.clone().then(
            just(Token::Mul).to("*".to_string())
                .or(just(Token::Div).to("/".to_string()))
                .then(atom.clone())
                .repeated()
        ).map(|(head, tail)| {
            tail.into_iter().fold(head, |lhs, (op, rhs)| {
                Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
            })
        });

        // 3. 加减 (低优先级)
        let full_expr = term.clone().then(
            just(Token::Add).to("+".to_string())
                .or(just(Token::Sub).to("-".to_string()))
                .then(term.clone())
                .repeated()
        ).map(|(head, tail)| {
            tail.into_iter().fold(head, |lhs, (op, rhs)| {
                Expr::BinOp(Box::new(lhs), op, Box::new(rhs))
            })
        });

        // 4. Print 表达式 (【核心修复】接受完整的 expr，而不是 atom！)
        let print_expr = just(Token::Print)
            .then(just(Token::LParen))
            .then(expr.clone()) // <--- 这里使用了闭包参数 expr，它代表最顶层的表达式
            .then(just(Token::RParen))
            .map(|(((_print, _lp), e), _rp)| Expr::Print(Box::new(e)));

        // print 的优先级和 full_expr 平级
        print_expr.or(full_expr)
    });

    // 5. Let 声明 (末尾可选分号)
    let let_decl = just(Token::Let)
        .ignore_then(select! { Token::Identifier(name) => name })
        .then_ignore(just(Token::Assign))
        .then(expr.clone()) // Let 的右侧也是完整的 expr
        .then(just(Token::Semicolon).or_not()) 
        .map(|((name, value), _semi)| TopLevel::LetDecl { name, value: Box::new(value) });

    // 6. Bridge 声明
    let param = select! { Token::Identifier(name) => name }
        .then(just(Token::Assign))
        .then(select! { Token::TypeInt => "Int".to_string(), Token::TypeFloat => "Float".to_string() })
        .map(|((name, _eq), ty)| BridgeParam::Param { name, ty });

    let params_list = param.separated_by(just(Token::Comma)).allow_trailing();

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

    // 7. 顶层语句 (末尾可选分号)
    let statement = expr
        .then(just(Token::Semicolon).or_not())
        .map(|(e, _semi)| TopLevel::Statement(e));

    bridge_decl.or(let_decl).or(statement)
        .repeated()
        .then_ignore(end())
}

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self { Self { variables: HashMap::new() } }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => self.variables.get(name).cloned().ok_or_else(|| format!("Undefined: {}", name)),
            Expr::BinOp(l, op, r) => {
                let lv = self.eval(l)?; let rv = self.eval(r)?;
                if let (Value::Number(a), Value::Number(b)) = (lv, rv) {
                    match op.as_str() {
                        "+" => Ok(Value::Number(a + b)),
                        "-" => Ok(Value::Number(a - b)),
                        "*" => Ok(Value::Number(a * b)),
                        "/" => Ok(Value::Number(a / b)),
                        _ => Err("Unknown op".into()),
                    }
                } else { Err("Type mismatch".into()) }
            }
            Expr::Call(name, _) => Err(format!("Bridge '{}' not supported in interpreter.", name)),
            Expr::Print(e) => { let v = self.eval(e)?; println!("Arc Output > {}", v); Ok(v) }
        }
    }
}