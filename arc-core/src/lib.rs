use logos::Logos;
use chumsky::prelude::*;
use std::collections::HashMap;
use std::fmt;

pub mod codegen;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"([ \t\n\r\f]+|//[^\n]*)")]
pub enum Token {
    #[token("let")] Let,
    #[token("print")] Print,
    #[token("bridge")] Bridge,
    #[token("func")] Func,
    #[token("return")] Return,
    #[token("use")] Use,
    #[token("system")] System,
    #[token("package")] Package,
    #[token("service")] Service,
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
    
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    FloatLit(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),
    
    #[token("Int")] TypeInt,
    #[token("Float")] TypeFloat,
    #[token("String")] TypeString,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "let"), Token::Print => write!(f, "print"),
            Token::Bridge => write!(f, "bridge"), Token::Func => write!(f, "func"),
            Token::Return => write!(f, "return"), Token::Use => write!(f, "use"),
            Token::System => write!(f, "system"), Token::Package => write!(f, "package"),
            Token::Service => write!(f, "service"), Token::RArrow => write!(f, "->"),
            Token::If => write!(f, "if"), Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"), Token::True => write!(f, "true"),
            Token::False => write!(f, "false"), Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"), Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"), Token::Assign => write!(f, "="),
            Token::LParen => write!(f, "("), Token::RParen => write!(f, ")"),
            Token::Add => write!(f, "+"), Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"), Token::Div => write!(f, "/"),
            Token::EqEq => write!(f, "=="), Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"), Token::Gt => write!(f, ">"),
            Token::Le => write!(f, "<="), Token::Ge => write!(f, ">="),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::FloatLit(s) => write!(f, "float {}", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::StringLit(s) => write!(f, "string '{}'", s),
            Token::TypeInt => write!(f, "Int"), Token::TypeFloat => write!(f, "Float"),
            Token::TypeString => write!(f, "String"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(i64), FloatLit(f64), StringLit(String), Identifier(String), Bool(bool),
    Print(Box<Expr>), Call(String, Vec<Expr>), BinOp(Box<Expr>, String, Box<Expr>),
    Compare(Box<Expr>, String, Box<Expr>), Assign(String, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr), If(Box<Expr>, Vec<Stmt>, Vec<Stmt>), While(Box<Expr>, Vec<Stmt>), Return(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam { pub name: String, pub ty: String }

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDecl { pub name: String, pub params: Vec<FuncParam>, pub ret_ty: String, pub body: Vec<Stmt> }

#[derive(Debug, PartialEq, Clone)]
pub enum BridgeParam { Param { name: String, ty: String } }

#[derive(Debug, PartialEq, Clone)]
pub struct ConfigItem { pub key: String, pub value: Expr }

#[derive(Debug, PartialEq, Clone)]
pub enum SystemUnit {
    Package { name: String, config: Vec<ConfigItem> },
    Service { name: String, config: Vec<ConfigItem> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevel {
    BridgeDecl { lang: String, lib: String, name: String, params: Vec<BridgeParam>, ret_ty: String },
    LetDecl { name: String, value: Box<Expr> },
    FuncDecl(FuncDecl),
    SystemDecl { units: Vec<SystemUnit> },
    UseDecl { path: String },
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub enum ExecutionResult { Value(Value), Return(Value) }

#[derive(Debug, Clone)]
pub enum Value { Number(i64), Float(f64), String(String), Bool(bool) }

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n), Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s), Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub fn parser() -> impl Parser<Token, Vec<TopLevel>, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let atom = {
            let num = select! { Token::Number(n) => Expr::Number(n) };
            let float_num = select! { Token::FloatLit(s) => Expr::FloatLit(s.parse::<f64>().unwrap()) };
            let str_lit = select! { Token::StringLit(s) => Expr::StringLit(s) };
            let ident = select! { Token::Identifier(i) => Expr::Identifier(i) };
            let bool_lit = select! { Token::True => Expr::Bool(true), Token::False => Expr::Bool(false) };
            let call = select! { Token::Identifier(name) => name }
                .then(just(Token::LParen)).then(expr.clone().separated_by(just(Token::Comma)).allow_trailing()).then(just(Token::RParen))
                .map(|(((name, _lp), args), _rp)| Expr::Call(name, args));
            let paren = just(Token::LParen).ignore_then(expr.clone()).then_ignore(just(Token::RParen));
            let print_expr = just(Token::Print).then(just(Token::LParen)).then(expr.clone()).then(just(Token::RParen))
                .map(|(((_print, _lp), e), _rp)| Expr::Print(Box::new(e)));
            print_expr.or(call).or(float_num).or(num).or(str_lit).or(ident).or(bool_lit).or(paren)
        };
        let term = atom.clone().then(just(Token::Mul).to("*".to_string()).or(just(Token::Div).to("/".to_string())).then(atom.clone()).repeated())
            .map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::BinOp(Box::new(lhs), op, Box::new(rhs))));
        let bin_expr = term.clone().then(just(Token::Add).to("+".to_string()).or(just(Token::Sub).to("-".to_string())).then(term.clone()).repeated())
            .map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::BinOp(Box::new(lhs), op, Box::new(rhs))));
        bin_expr.clone().then(
            just(Token::EqEq).to("==".to_string()).or(just(Token::NotEq).to("!=".to_string()))
                .or(just(Token::Lt).to("<".to_string())).or(just(Token::Gt).to(">".to_string()))
                .or(just(Token::Le).to("<=".to_string())).or(just(Token::Ge).to(">=".to_string()))
                .then(bin_expr.clone()).repeated()
        ).map(|(head, tail)| tail.into_iter().fold(head, |lhs, (op, rhs)| Expr::Compare(Box::new(lhs), op, Box::new(rhs))))
    });

    let stmt = recursive(|stmt| {
        let block = just(Token::LBrace).ignore_then(stmt.repeated()).then_ignore(just(Token::RBrace));
        let if_stmt = just(Token::If).ignore_then(just(Token::LParen)).ignore_then(expr.clone()).then_ignore(just(Token::RParen))
            .then(block.clone()).then(just(Token::Else).ignore_then(block.clone()).or_not())
            .map(|((cond, then), else_block)| Stmt::If(Box::new(cond), then, else_block.unwrap_or_default()));
        let while_stmt = just(Token::While).ignore_then(just(Token::LParen)).ignore_then(expr.clone()).then_ignore(just(Token::RParen))
            .then(block.clone()).map(|(cond, body)| Stmt::While(Box::new(cond), body));
        let return_stmt = just(Token::Return).ignore_then(expr.clone()).then_ignore(just(Token::Semicolon).or_not())
            .map(|e| Stmt::Return(Box::new(e)));
        let assign_stmt = select! { Token::Identifier(name) => name }.then_ignore(just(Token::Assign)).then(expr.clone()).then_ignore(just(Token::Semicolon).or_not())
            .map(|(name, value)| Stmt::Expr(Expr::Assign(name, Box::new(value))));
        let expr_stmt = expr.clone().then_ignore(just(Token::Semicolon).or_not()).map(Stmt::Expr);
        if_stmt.or(while_stmt).or(return_stmt).or(assign_stmt).or(expr_stmt)
    });

    let func_param = select! { Token::Identifier(name) => name }.then(just(Token::Assign))
        .then(select! { Token::TypeInt => "Int".to_string() }.or(select! { Token::TypeFloat => "Float".to_string() }).or(select! { Token::TypeString => "String".to_string() }))
        .map(|((name, _eq), ty)| FuncParam { name, ty });
    let func_params_list = func_param.separated_by(just(Token::Comma)).allow_trailing();
    let func_decl = just(Token::Func).ignore_then(select! { Token::Identifier(name) => name })
        .then_ignore(just(Token::LParen)).then(func_params_list).then_ignore(just(Token::RParen)).then_ignore(just(Token::RArrow))
        .then(select! { Token::TypeInt => "Int".to_string() }.or(select! { Token::TypeFloat => "Float".to_string() }).or(select! { Token::TypeString => "String".to_string() }))
        .then_ignore(just(Token::LBrace)).then(stmt.clone().repeated()).then_ignore(just(Token::RBrace))
        .map(|(((name, params), ret_ty), body)| TopLevel::FuncDecl(FuncDecl { name, params, ret_ty, body }));

    let block_for_top = just(Token::LBrace).ignore_then(stmt.repeated()).then_ignore(just(Token::RBrace));
    let top_if = just(Token::If).ignore_then(just(Token::LParen)).ignore_then(expr.clone()).then_ignore(just(Token::RParen))
        .then(block_for_top.clone()).then(just(Token::Else).ignore_then(block_for_top.clone()).or_not()).then_ignore(just(Token::Semicolon).or_not())
        .map(|((cond, then), else_block)| TopLevel::Stmt(Stmt::If(Box::new(cond), then, else_block.unwrap_or_default())));
    let top_while = just(Token::While).ignore_then(just(Token::LParen)).ignore_then(expr.clone()).then_ignore(just(Token::RParen))
        .then(block_for_top.clone()).then_ignore(just(Token::Semicolon).or_not())
        .map(|(cond, body)| TopLevel::Stmt(Stmt::While(Box::new(cond), body)));
    let let_decl = just(Token::Let).ignore_then(select! { Token::Identifier(name) => name }).then_ignore(just(Token::Assign)).then(expr.clone()).then_ignore(just(Token::Semicolon).or_not()) 
        .map(|(name, value)| TopLevel::LetDecl { name, value: Box::new(value) });

    let param = select! { Token::Identifier(name) => name }.then(just(Token::Assign))
        .then(select! { Token::TypeInt => "Int".to_string() }.or(select! { Token::TypeFloat => "Float".to_string() }).or(select! { Token::TypeString => "String".to_string() }))
        .map(|((name, _eq), ty)| BridgeParam::Param { name, ty });
    let params_list = param.separated_by(just(Token::Comma)).allow_trailing();
    let bridge_decl = just(Token::Bridge).then(select! { Token::Identifier(lang) => lang }).then(select! { Token::StringLit(lib) => lib })
        .then(just(Token::LBrace)).then(just(Token::Func)).then(select! { Token::Identifier(name) => name })
        .then(just(Token::LParen)).then(params_list).then(just(Token::RParen)).then(just(Token::RArrow))
        .then(select! { Token::TypeInt => "Int".to_string() }.or(select! { Token::TypeFloat => "Float".to_string() }))
        .then(just(Token::RBrace))
        .map(|(((((((((((_bridge, lang), lib), _lb), _func), name), _lp), params), _rp), _ra), ret_ty), _rb)| {
            TopLevel::BridgeDecl { lang, lib, name, params, ret_ty }
        });

    let config_item = select! { Token::Identifier(k) => k }.then_ignore(just(Token::Assign)).then(expr.clone()).then_ignore(just(Token::Semicolon).or_not())
        .map(|(k, v)| ConfigItem { key: k, value: v });
    let config_block = just(Token::LBrace).ignore_then(config_item.repeated()).then_ignore(just(Token::RBrace));
    let package_unit = just(Token::Package).ignore_then(select! { Token::StringLit(n) => n }).then(config_block.clone())
        .map(|(name, config)| SystemUnit::Package { name, config });
    let service_unit = just(Token::Service).ignore_then(select! { Token::StringLit(n) => n }).then(config_block)
        .map(|(name, config)| SystemUnit::Service { name, config });
    let system_decl = just(Token::System).ignore_then(just(Token::LBrace))
        .ignore_then(package_unit.or(service_unit).repeated()).then_ignore(just(Token::RBrace))
        .map(|units| TopLevel::SystemDecl { units });

    let use_decl = just(Token::Use).ignore_then(select! { Token::StringLit(path) => path }).then_ignore(just(Token::Semicolon).or_not())
        .map(|path| TopLevel::UseDecl { path });

    let top_stmt = expr.then_ignore(just(Token::Semicolon).or_not()).map(|e| TopLevel::Stmt(Stmt::Expr(e)));

    bridge_decl.or(let_decl).or(func_decl).or(system_decl).or(use_decl).or(top_if).or(top_while).or(top_stmt).repeated().then_ignore(end())
}

pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, FuncDecl>,
}

impl Environment {
    pub fn new() -> Self { Self { variables: HashMap::new(), functions: HashMap::new() } }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ExecutionResult, String> {
        match stmt {
            Stmt::Expr(e) => Ok(ExecutionResult::Value(self.eval_expr(e)?)),
            Stmt::If(cond, then, else_) => {
                if let Value::Bool(b) = self.eval_expr(cond)? {
                    if b { for s in then { let res = self.eval_stmt(s)?; if let ExecutionResult::Return(_) = res { return Ok(res); } } } 
                    else { for s in else_ { let res = self.eval_stmt(s)?; if let ExecutionResult::Return(_) = res { return Ok(res); } } }
                    Ok(ExecutionResult::Value(Value::Bool(b)))
                } else { Err("Condition must be bool".into()) }
            }
            Stmt::While(cond, body) => {
                while let Value::Bool(true) = self.eval_expr(cond)? {
                    for s in body { let res = self.eval_stmt(s)?; if let ExecutionResult::Return(_) = res { return Ok(res); } }
                }
                Ok(ExecutionResult::Value(Value::Bool(false)))
            }
            Stmt::Return(e) => Ok(ExecutionResult::Return(self.eval_expr(e)?)),
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)), Expr::FloatLit(n) => Ok(Value::Float(*n)),
            Expr::StringLit(s) => Ok(Value::String(s.clone())), Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Identifier(name) => self.variables.get(name).cloned().ok_or_else(|| format!("Undefined: {}", name)),
            Expr::BinOp(l, op, r) => {
                let lv = self.eval_expr(l)?; let rv = self.eval_expr(r)?;
                match (&lv, &rv) {
                    (Value::Float(a), Value::Float(b)) => match op.as_str() {
                        "+" => Ok(Value::Float(a + b)), "-" => Ok(Value::Float(a - b)),
                        "*" => Ok(Value::Float(a * b)), "/" => Ok(Value::Float(a / b)), _ => Err("Unknown op".into()),
                    },
                    (Value::Number(a), Value::Number(b)) => match op.as_str() {
                        "+" => Ok(Value::Number(a + b)), "-" => Ok(Value::Number(a - b)),
                        "*" => Ok(Value::Number(a * b)), "/" => Ok(Value::Number(a / b)), _ => Err("Unknown op".into()),
                    },
                    _ => Err("Type mismatch".into()),
                }
            }
            Expr::Compare(l, op, r) => {
                let lv = self.eval_expr(l)?; let rv = self.eval_expr(r)?;
                if let (Value::Number(a), Value::Number(b)) = (lv, rv) {
                    let res = match op.as_str() {
                        "==" => a == b, "!=" => a != b, "<" => a < b, ">" => a > b, "<=" => a <= b, ">=" => a >= b, _ => false,
                    };
                    Ok(Value::Bool(res))
                } else { Err("Type mismatch".into()) }
            }
            Expr::Assign(name, value) => { let v = self.eval_expr(value)?; self.variables.insert(name.clone(), v.clone()); Ok(v) }
            Expr::Print(e) => { let v = self.eval_expr(e)?; println!("Arc Output > {}", v); Ok(v) }
            Expr::Call(name, args) => {
                if let Some(func) = self.functions.get(name).cloned() { 
                    if args.len() != func.params.len() { return Err(format!("Function '{}' expects {} args, got {}", name, func.params.len(), args.len())); }
                    let mut new_env = Environment::new();
                    new_env.variables = self.variables.clone(); new_env.functions = self.functions.clone();
                    for (i, arg) in args.iter().enumerate() { let val = self.eval_expr(arg)?; new_env.variables.insert(func.params[i].name.clone(), val); }
                    for s in &func.body { match new_env.eval_stmt(s)? { ExecutionResult::Return(val) => return Ok(val), ExecutionResult::Value(_) => continue, } }
                    Ok(Value::Number(0)) 
                } else { Err(format!("Bridge function '{}' not supported in interpreter.", name)) }
            }
        }
    }

    pub fn eval_system_decl(&self, units: &[SystemUnit]) {
        println!("\n[A.R.C.A.E.A. SYSTEM] 正在解析声明式配置...");
        for unit in units {
            match unit {
                SystemUnit::Package { name, config } => {
                    println!("\n  [PACKAGE] {}", name);
                    for item in config { println!("    ├─ {} = {:?}", item.key, item.value); }
                }
                SystemUnit::Service { name, config } => {
                    println!("\n  [SERVICE] {}", name);
                    for item in config { println!("    ├─ {} = {:?}", item.key, item.value); }
                }
            }
        }
        println!("\n[A.R.C.A.E.A. SYSTEM] 配置解析完成。\n");
    }
}