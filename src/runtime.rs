use std::collections::HashMap;
use std::rc::Rc;

use tale::Lambda;

use super::tale;
use super::tale::{Environment, Error, Expression};

pub fn parse_eval(expr: String, env: &mut Environment) -> Result<Expression, Error> {
    let (parsed_exp, _) = tale::parse(&tale::tokenize(expr))?;
    let evaled_exp = eval(&parsed_exp, env)?;

    Ok(evaled_exp)
}

#[macro_use()]
macro_rules! ensure_tonicity {
    ($check_fn:expr) => {{
        |args: &[Expression]| -> Result<Expression, Error> {
            let floats = tale::parse_list_of_floats(args)?;
            let first = floats
                .first()
                .ok_or(Error::Generic("Expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f(prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            };
            Ok(Expression::Boolean(f(first, rest)))
        }
    }};
}

pub fn default_environment<'a>() -> Environment<'a> {
    let mut environment: HashMap<String, Expression> = HashMap::new();

    environment.insert(
        "+".to_string(),
        Expression::Function(|args: &[Expression]| -> Result<Expression, Error> {
            let sum = tale::parse_list_of_floats(args)?
                .iter()
                .fold(0.0, |sum, a| sum + a);

            Ok(Expression::Number(sum))
        }),
    );

    environment.insert(
        "-".to_string(),
        Expression::Function(|args: &[Expression]| -> Result<Expression, Error> {
            let floats = tale::parse_list_of_floats(args)?;
            let first = *floats
                .first()
                .ok_or(Error::Generic("expected at least one number".to_string()))?;
            let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);

            Ok(Expression::Number(first - sum_of_rest))
        }),
    );

    environment.insert(
        "=".to_string(),
        Expression::Function(ensure_tonicity!(|a, b| a == b)),
    );
    environment.insert(
        ">".to_string(),
        Expression::Function(ensure_tonicity!(|a, b| a > b)),
    );
    environment.insert(
        ">=".to_string(),
        Expression::Function(ensure_tonicity!(|a, b| a >= b)),
    );
    environment.insert(
        "<".to_string(),
        Expression::Function(ensure_tonicity!(|a, b| a < b)),
    );
    environment.insert(
        "<=".to_string(),
        Expression::Function(ensure_tonicity!(|a, b| a <= b)),
    );

    Environment {
        data: environment,
        scope: None,
    }
}

fn env_get(key: &str, env: &Environment) -> Option<Expression> {
    match env.data.get(key) {
        Some(exp) => Some(exp.clone()),
        None => match &env.scope {
            Some(scope) => env_get(key, &scope),
            None => None,
        },
    }
}

fn env_for_lambda<'a>(
    params: Rc<Expression>,
    arg_forms: &[Expression],
    scope: &'a mut Environment,
) -> Result<Environment<'a>, Error> {
    let ks = tale::parse_list_of_symbol_strings(params)?;
    if ks.len() != arg_forms.len() {
        return Err(Error::Generic(format!(
            "expected {} arguments, got {}",
            ks.len(),
            arg_forms.len()
        )));
    }
    let vs = eval_forms(arg_forms, scope)?;
    let mut data: HashMap<String, Expression> = HashMap::new();
    for (k, v) in ks.iter().zip(vs.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(Environment {
        data,
        scope: Some(scope),
    })
}

fn eval(exp: &Expression, env: &mut Environment) -> Result<Expression, Error> {
    match exp {
        Expression::Symbol(key) => env_get(key, env)
            .ok_or(Error::Generic(format!("Unexpected symbol '{}'", key)))
            .map(|x| x.clone()),
        Expression::Number(_a) => Ok(exp.clone()),
        Expression::List(list) => {
            let first_form = list
                .first()
                .ok_or(Error::Generic("expected a non-empty list".to_string()))?;
            let arg_forms = &list[1..];
            match eval_built_in_form(first_form, arg_forms, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        Expression::Function(f) => f(&eval_forms(arg_forms, env)?),
                        Expression::Lambda(lambda) => {
                            let new_env = &mut env_for_lambda(lambda.params, arg_forms, env)?;
                            eval(&lambda.body, new_env)
                        }
                        _ => Err(Error::Generic("first form must be a function".to_string())),
                    }
                }
            }
        }
        Expression::Function(_) => Err(Error::Generic("unexpected form".to_string())),
        Expression::Boolean(_a) => Ok(exp.clone()),
        Expression::Lambda(_) => Err(Error::Generic("unexpected form".to_string())),
    }
}

fn eval_forms(arg_forms: &[Expression], env: &mut Environment) -> Result<Vec<Expression>, Error> {
    arg_forms.iter().map(|x| eval(x, env)).collect()
}

fn eval_built_in_form(
    exp: &Expression,
    arg_forms: &[Expression],
    env: &mut Environment,
) -> Option<Result<Expression, Error>> {
    match exp {
        Expression::Symbol(s) => match s.as_ref() {
            "if" => Some(eval_if_args(arg_forms, env)),
            "def" => Some(eval_def_args(arg_forms, env)),
            "fn" => Some(eval_lambda_args(arg_forms)),
            _ => None,
        },
        _ => None,
    }
}

fn eval_if_args(arg_forms: &[Expression], env: &mut Environment) -> Result<Expression, Error> {
    let test_form = arg_forms
        .first()
        .ok_or(Error::Generic("expected test form".to_string()))?;
    let test_eval = eval(test_form, env)?;
    match test_eval {
        Expression::Boolean(b) => {
            let form_idx = if b { 1 } else { 2 };
            let res_form = arg_forms
                .get(form_idx)
                .ok_or(Error::Generic(format!("expected form idx={}", form_idx)))?;
            let res_eval = eval(res_form, env);

            res_eval
        }
        _ => Err(Error::Generic(format!(
            "unexpected test form='{}'",
            test_form.to_string()
        ))),
    }
}

fn eval_def_args(arg_forms: &[Expression], env: &mut Environment) -> Result<Expression, Error> {
    let first_form = arg_forms
        .first()
        .ok_or(Error::Generic("expected first form".to_string()))?;
    let first_str = match first_form {
        Expression::Symbol(s) => Ok(s.clone()),
        _ => Err(Error::Generic(
            "expected first form to be a symbol".to_string(),
        )),
    }?;
    let second_form = arg_forms
        .get(1)
        .ok_or(Error::Generic("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(Error::Generic("def can only have two forms ".to_string()));
    }
    let second_eval = eval(second_form, env)?;
    env.data.insert(first_str, second_eval);

    Ok(first_form.clone())
}

fn eval_lambda_args(arg_forms: &[Expression]) -> Result<Expression, Error> {
    let params_exp = arg_forms
        .first()
        .ok_or(Error::Generic("expected args form".to_string()))?;
    let body_exp = arg_forms
        .get(1)
        .ok_or(Error::Generic("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(Error::Generic(
            "fn definition can only have two forms ".to_string(),
        ));
    }

    Ok(Expression::Lambda(Lambda {
        body: Rc::new(body_exp.clone()),
        params: Rc::new(params_exp.clone()),
    }))
}
