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
    #[token("if")] If,
    #[token("else")] Else,
    #[token("while")] While,
    #[token("true")] True,
    #[token("false")] False,
    
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
    
    #[token("==")] EqEq,
    #[token("!=")] NotEq,
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("<=")] Le,
    #[token(">=")] Ge,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // 【修复】在 Token 层只存字符串，避免 f64 无法实现 Eq/Hash 的问题
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    FloatLit(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),
    
    #[token("Int")] TypeInt,
    #[token("Float")] TypeFloat,
    #[token("String")] TypeString, // 【新增】Bridge 参数类型关键字
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"), Token::Print => write!(f, "print"),
            Token::Bridge => write!(f, "bridge"), Token::Func => write!(f, "func"),
            Token::RArrow => write!(f, "->"), Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"), Token::While => write!(f, "while"),
            Token::True => write!(f, "true"), Token::False => write!(f, "false"),
            Token::LBrace => write!(f, "{{"), Token::RBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","), Token::Semicolon => write!(f, ";"),
            Token::Assign => write!(f, "="), Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Add => write!(f, "+"), Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"), Token::Div => write!(f, "/"),
            Token::EqEq => write!(f, "=="), Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"), Token::Gt => write!(f, ">"),
            Token::Le => write!(f, "<="), Token::Ge => write!(f, ">="),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::FloatLit(s) => write!(f, "float {}", s), // 【修复】
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::TypeInt => write!(f, "Int"), Token::TypeFloat => write!(f, "Float"),
            Token::TypeString => write!(f, "String"), // 【新增】
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(i64),
    FloatLit(f64), // 【新增】
    StringLit(String),
    Identifier(String),
    Bool(bool),
    Print(Box<Expr>),
    Call(String, Vec<Expr>),
    BinOp(Box<Expr>, String, Box<Expr>), 
    Compare(Box<Expr>, String, Box<Expr>),
    Assign(String, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>),
    While(Box<Expr>, Vec<Stmt>),
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
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Float(f64), // 【新增】
    String(String),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n), // 【新增】
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub fn parser() -> impl Parser<Token, Vec<TopLevel>, Error = Simple<Token>> {
    // 1. 原子表达式
    let atom = recursive(|atom| {
        let num = select! { Token::Number(n) => Expr::Number(n) };
            // 【修复】在 Parser 层将 String 解析为 f64
        let float_num = select! { Token::FloatLit(s) => Expr::FloatLit(s.parse::<f64>().unwrap()) };
        let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
        let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };
        let bool_lit = select! { Token::True => Expr::Bool(true), Token::False => Expr::Bool(false) };

        let call = select! { Token::Identifier(name) => name }
            .then(just(Token::LParen))
            .then(atom.clone().separated_by(just(Token::Comma)).allow_trailing())
            .then(just(Token::RParen))
            .map(|(((name, _lp), args), _rp)| Expr::Call(name, args));

        let paren = just(Token::LParen)
            .ignore_then(atom.clone())
            .then_ignore(just(Token::RParen));

        let print_expr = just(Token::Print)
            .then(just(Token::LParen))
            .then(atom.clone())
            .then(just(Token::RParen))
            .map(|(((_print, _lp), e), _rp)| Expr::Print(Box::new(e)));

        print_expr.or(call).or(float_num).or(num).or(str_lit).or(ident).or(bool_lit).or(paren)
    });

    // 2. 乘除
    let term = atom.clone().then(
        just(Token::Mul).to("*".to_string()).or(just(Token::Div).to("/".to_string())).then(atom.clone()).repeated()
    ).map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::BinOp(Box::new(lhs), op, Box::new(rhs))));

    // 3. 加减
    let expr = term.clone().then(
        just(Token::Add).to("+".to_string()).or(just(Token::Sub).to("-".to_string())).then(term.clone()).repeated()
    ).map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::BinOp(Box::new(lhs), op, Box::new(rhs))));

    // 4. 比较
    let compare = expr.clone().then(
        just(Token::EqEq).to("==".to_string())
            .or(just(Token::NotEq).to("!=".to_string()))
            .or(just(Token::Lt).to("<".to_string()))
            .or(just(Token::Gt).to(">".to_string()))
            .or(just(Token::Le).to("<=".to_string()))
            .or(just(Token::Ge).to(">=".to_string()))
            .then(expr.clone())
            .repeated()
    ).map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::Compare(Box::new(lhs), op, Box::new(rhs))));

    // 5. 语句定义
    let stmt = recursive(|stmt| {
        let block = just(Token::LBrace)
            .ignore_then(stmt.repeated())
            .then_ignore(just(Token::RBrace));

        let if_stmt = just(Token::If)
            .ignore_then(just(Token::LParen))
            .ignore_then(compare.clone())
            .then_ignore(just(Token::RParen))
            .then(block.clone())
            .then(just(Token::Else).ignore_then(block.clone()).or_not())
            .map(|((cond, then), else_block)| Stmt::If(Box::new(cond), then, else_block.unwrap_or_default()));

        let while_stmt = just(Token::While)
            .ignore_then(just(Token::LParen))
            .ignore_then(compare.clone())
            .then_ignore(just(Token::RParen))
            .then(block.clone())
            .map(|(cond, body)| Stmt::While(Box::new(cond), body));

        let assign_stmt = select! { Token::Identifier(name) => name }
            .then_ignore(just(Token::Assign))
            .then(compare.clone())
            .then_ignore(just(Token::Semicolon).or_not())
            .map(|(name, value)| Stmt::Expr(Expr::Assign(name, Box::new(value))));

        let expr_stmt = compare.clone()
            .then_ignore(just(Token::Semicolon).or_not())
            .map(Stmt::Expr);

        if_stmt.or(while_stmt).or(assign_stmt).or(expr_stmt)
    });

    let block_for_top = just(Token::LBrace)
        .ignore_then(stmt.repeated())
        .then_ignore(just(Token::RBrace));

    let top_if = just(Token::If)
        .ignore_then(just(Token::LParen))
        .ignore_then(compare.clone())
        .then_ignore(just(Token::RParen))
        .then(block_for_top.clone())
        .then(just(Token::Else).ignore_then(block_for_top.clone()).or_not())
        .then_ignore(just(Token::Semicolon).or_not())
        .map(|((cond, then), else_block)| TopLevel::Stmt(Stmt::If(Box::new(cond), then, else_block.unwrap_or_default())));

    let top_while = just(Token::While)
        .ignore_then(just(Token::LParen))
        .ignore_then(compare.clone())
        .then_ignore(just(Token::RParen))
        .then(block_for_top.clone())
        .then_ignore(just(Token::Semicolon).or_not())
        .map(|(cond, body)| TopLevel::Stmt(Stmt::While(Box::new(cond), body)));

    let let_decl = just(Token::Let)
        .ignore_then(select! { Token::Identifier(name) => name })
        .then_ignore(just(Token::Assign))
        .then(compare.clone())
        .then_ignore(just(Token::Semicolon).or_not()) 
        .map(|(name, value)| TopLevel::LetDecl { name, value: Box::new(value) });

    // 7. Bridge 声明 (支持 String 类型)
    let param = select! { Token::Identifier(name) => name }
        .then(just(Token::Assign))
        .then(
            select! { Token::TypeInt => "Int".to_string() }
            .or(select! { Token::TypeFloat => "Float".to_string() })
            .or(select! { Token::TypeString => "String".to_string() }) // 【新增】
        )
        .map(|((name, _eq), ty)| BridgeParam::Param { name, ty });
        
    let params_list = param.separated_by(just(Token::Comma)).allow_trailing();
    let bridge_decl = just(Token::Bridge)
        .then(select! { Token::Identifier(lang) => lang })
        .then(select! { Token::StringLit(lib) => lib })
        .then(just(Token::LBrace)).then(just(Token::Func))
        .then(select! { Token::Identifier(name) => name })
        .then(just(Token::LParen)).then(params_list).then(just(Token::RParen))
        .then(just(Token::RArrow))
        .then(
            select! { Token::TypeInt => "Int".to_string() }
            .or(select! { Token::TypeFloat => "Float".to_string() })
        )
        .then(just(Token::RBrace))
        .map(|(((((((((((_bridge, lang), lib), _lb), _func), name), _lp), params), _rp), _ra), ret_ty), _rb)| {
            TopLevel::BridgeDecl { lang, lib, name, params, ret_ty }
        });

    let top_stmt = compare
        .then_ignore(just(Token::Semicolon).or_not())
        .map(|e| TopLevel::Stmt(Stmt::Expr(e)));

    bridge_decl.or(let_decl).or(top_if).or(top_while).or(top_stmt)
        .repeated()
        .then_ignore(end())
}

pub struct Environment {
    variables: HashMap<String, Value>,
}
impl Environment {
    pub fn new() -> Self { Self { variables: HashMap::new() } }
    pub fn eval(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match stmt {
            Stmt::Expr(e) => self.eval_expr(e),
            Stmt::If(cond, then, else_) => {
                if let Value::Bool(b) = self.eval_expr(cond)? {
                    if b { for s in then { self.eval(s)?; } }
                    else { for s in else_ { self.eval(s)?; } }
                    Ok(Value::Bool(b))
                } else { Err("Condition must be bool".into()) }
            }
            Stmt::While(cond, body) => {
                while let Value::Bool(true) = self.eval_expr(cond)? {
                    for s in body { self.eval(s)?; }
                }
                Ok(Value::Bool(false))
            }
        }
    }
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::FloatLit(n) => Ok(Value::Float(*n)), // 【新增】
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Identifier(name) => self.variables.get(name).cloned().ok_or_else(|| format!("Undefined: {}", name)),
            Expr::BinOp(l, op, r) => {
                let lv = self.eval_expr(l)?; 
                let rv = self.eval_expr(r)?;
                
                // 【修复】使用 match 和引用 (&lv, &rv) 避免所有权移动
                match (&lv, &rv) {
                    (Value::Float(a), Value::Float(b)) => {
                        match op.as_str() {
                            "+" => Ok(Value::Float(a + b)), "-" => Ok(Value::Float(a - b)),
                            "*" => Ok(Value::Float(a * b)), "/" => Ok(Value::Float(a / b)),
                            _ => Err("Unknown op".into()),
                        }
                    }
                    (Value::Number(a), Value::Number(b)) => {
                        match op.as_str() {
                            "+" => Ok(Value::Number(a + b)), "-" => Ok(Value::Number(a - b)),
                            "*" => Ok(Value::Number(a * b)), "/" => Ok(Value::Number(a / b)),
                            _ => Err("Unknown op".into()),
                        }
                    }
                    _ => Err("Type mismatch".into()),
                }
            }
            Expr::Compare(l, op, r) => {
                let lv = self.eval_expr(l)?; let rv = self.eval_expr(r)?;
                if let (Value::Number(a), Value::Number(b)) = (lv, rv) {
                    let res = match op.as_str() {
                        "==" => a == b, "!=" => a != b, "<" => a < b, ">" => a > b, "<=" => a <= b, ">=" => a >= b,
                        _ => false,
                    };
                    Ok(Value::Bool(res))
                } else { Err("Type mismatch".into()) }
            }
            Expr::Assign(name, value) => {
                let v = self.eval_expr(value)?;
                self.variables.insert(name.clone(), v.clone());
                Ok(v)
            }
            Expr::Print(e) => { let v = self.eval_expr(e)?; println!("Arc Output > {}", v); Ok(v) }
            Expr::Call(name, _) => Err(format!("Bridge '{}' not supported in interpreter.", name)),
        }
    }
}