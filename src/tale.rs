use std::collections::HashMap;
use std::fmt;
use std::num::ParseFloatError;

use std::rc::Rc;

#[derive(Clone)]
pub enum Expression {
    Symbol(String),
    Number(f64),
    List(Vec<Expression>),
    Function(fn(&[Expression]) -> Result<Expression, Error>),
    Boolean(bool),
    Lambda(Lambda),
}

#[derive(Clone)]
pub struct Lambda {
    pub params: Rc<Expression>,
    pub body: Rc<Expression>,
}

#[derive(Debug)]
pub enum Error {
    /// Unbalanced parens
    UnbalancedParens(u32),
    /// Syntax error returning line number and column number
    SyntaxError(u32, u32),
    /// Generic catch all error for backwards compatibility
    Generic(String),
}

#[derive(Clone)]
pub struct Environment<'a> {
    pub data: HashMap<String, Expression>,
    pub scope: Option<&'a Environment<'a>>,
}

pub fn parse<'a>(tokens: &'a [String]) -> Result<(Expression, &'a [String]), Error> {
    let (token, rest) = tokens.split_first().ok_or(Error::Generic(
        "Could not get token from expression".to_string(),
    ))?;

    match &token[..] {
        "(" => read_seq(rest),
        ")" => Err(Error::Generic("Unexpected `)`".to_string())),
        _ => Ok((parse_atom(token), rest)),
    }
}

pub fn parse_list_of_floats(args: &[Expression]) -> Result<Vec<f64>, Error> {
    args.iter().map(|x| parse_single_float(x)).collect()
}

pub fn parse_single_float(exp: &Expression) -> Result<f64, Error> {
    match exp {
        Expression::Number(num) => Ok(*num),
        _ => Err(Error::Generic("expected a number".to_string())),
    }
}

pub fn read_seq<'a>(tokens: &'a [String]) -> Result<(Expression, &'a [String]), Error> {
    let mut expressions: Vec<Expression> = vec![];
    let mut local_tokens = tokens;

    loop {
        let (next, rest) = local_tokens
            .split_first()
            .ok_or(Error::Generic("Could not find closing `)`".to_string()))?;

        if next == ")" {
            return Ok((Expression::List(expressions), rest));
        }

        let (expression, new_tokens) = parse(&local_tokens)?;
        local_tokens = new_tokens;
        expressions.push(expression);
    }
}

pub fn parse_atom(token: &str) -> Expression {
    match token.as_ref() {
        "true" => Expression::Boolean(true),
        "false" => Expression::Boolean(false),
        _ => {
            let potential_float: Result<f64, ParseFloatError> = token.parse();
            match potential_float {
                Ok(v) => Expression::Number(v),
                Err(_) => Expression::Symbol(token.to_string().clone()),
            }
        }
    }
}

pub fn parse_list_of_symbol_strings(form: Rc<Expression>) -> Result<Vec<String>, Error> {
    let list = match form.as_ref() {
        Expression::List(s) => Ok(s.clone()),
        _ => Err(Error::Generic(
            "expected args form to be a list".to_string(),
        )),
    }?;
    list.iter()
        .map(|x| match x {
            Expression::Symbol(s) => Ok(s.clone()),
            _ => Err(Error::Generic(
                "expected symbols in the argument list".to_string(),
            )),
        })
        .collect()
}

pub fn tokenize(expression: String) -> Vec<String> {
    expression
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Expression::Symbol(s) => s.clone(),
            Expression::Number(n) => n.to_string(),
            Expression::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(","))
            }
            Expression::Function(_) => "Function {}".to_string(),
            Expression::Boolean(a) => a.to_string(),
            Expression::Lambda(_) => "Lambda {}".to_string(),
        };

        write!(f, "{}", str)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Error::SyntaxError(_line, _col) => "Syntax error at line {}, column {}".to_string(),
            Error::UnbalancedParens(_u32) => "Unbalanced parens, need {} more".to_string(),
            Error::Generic(reason) => reason.to_string(),
        };

        write!(f, "{}", str)
    }
}
