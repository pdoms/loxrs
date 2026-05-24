#![allow(clippy::ptr_arg)]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    errors::RuntimeError,
    nodes::{Lit, NativeFunction},
};

// === TIME ===
fn clock(_args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Ok(Lit::Number(secs))
}

// === MATH ===
fn sqrt(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    num_1(&args[0], f64::sqrt)
}

fn floor(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    num_1(&args[0], f64::floor)
}

fn ceil(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    num_1(&args[0], f64::ceil)
}

fn abs(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    num_1(&args[0], f64::abs)
}

fn pow(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 2)?;
    num_2(&args[0], &args[1], f64::powf)
}

fn to_number(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    match &args[0] {
        Lit::Number(n) => Ok(Lit::Number(*n)),
        Lit::String(s) => Ok(s.parse::<f64>().map(Lit::Number).unwrap_or(Lit::Nil)),
        _ => Ok(Lit::Nil),
    }
}

fn string_len(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    match &args[0] {
        Lit::String(s) => Ok(Lit::Number(s.len() as f64)),
        _ => Err(RuntimeError::TypeError {
            msg: "expected string".to_string(),
        }),
    }
}

fn read_file(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 1)?;
    match &args[0] {
        Lit::String(path) => Ok(std::fs::read_to_string(path)
            .map(Lit::String)
            .map_err(|err| RuntimeError::Io {
                msg: format!("could not read file {err}"),
            })?),
        _ => Err(RuntimeError::TypeError {
            msg: "expected string path".to_string(),
        }),
    }
}
fn write_file(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 2)?;
    match (&args[0], &args[1]) {
        (Lit::String(path), Lit::String(content)) => Ok(std::fs::write(path, content)
            .map(|_| Lit::Bool(true))
            .map_err(|err| RuntimeError::Io {
                msg: format!("could not read file {err}"),
            })?),
        _ => Err(RuntimeError::TypeError {
            msg: "expected string path".to_string(),
        }),
    }
}
fn append_file(args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    validate_args_len(args, 2)?;
    match (&args[0], &args[1]) {
        (Lit::String(path), Lit::String(content)) => {
            use std::io::Write;
            let result = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .and_then(|mut f| f.write_all(content.as_bytes()))
                .map(|_| Lit::Bool(true))
                .map_err(|err| RuntimeError::Io {
                    msg: format!("could not read file {err}"),
                });
            Ok(Lit::Bool(result.is_ok()))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "expected string path".to_string(),
        }),
    }
}

fn validate_args_len(args: &[Lit], len: usize) -> Result<(), RuntimeError> {
    if args.len() != len {
        return Err(RuntimeError::ArityMismatch {
            expected: len,
            got: args.len(),
        });
    }
    Ok(())
}
fn num_1(a: &Lit, f: fn(f64) -> f64) -> Result<Lit, RuntimeError> {
    match a {
        Lit::Number(n) => Ok(Lit::Number(f(*n))),
        _ => Err(RuntimeError::TypeError {
            msg: "expected a number".to_string(),
        }),
    }
}

fn num_2(a: &Lit, b: &Lit, f: fn(f64, f64) -> f64) -> Result<Lit, RuntimeError> {
    match (a, b) {
        (Lit::Number(x), Lit::Number(y)) => Ok(Lit::Number(f(*x, *y))),
        _ => Err(RuntimeError::TypeError {
            msg: "expected a number".to_string(),
        }),
    }
}

pub fn native_functions() -> Vec<(String, Lit)> {
    vec![
        //time
        native("clock", 0, clock),
        //math
        native("sqrt", 1, sqrt),
        native("floor", 1, floor),
        native("ceil", 1, ceil),
        native("abs", 1, abs),
        native("pow", 2, pow),
        //type
        native("type_of", 1, |args| {
            validate_args_len(args, 1)?;
            let t = match &args[0] {
                Lit::Number(_) => "number",
                Lit::String(_) => "string",
                Lit::Bool(_) => "bool",
                Lit::Nil => "nil",
                Lit::Function(_) => "fun",
                Lit::NativeFunction(_) => "native_fun",
            };
            Ok(Lit::String(t.to_string()))
        }),
        native("to_string", 1, |args| {
            validate_args_len(args, 1)?;
            Ok(Lit::String(args[0].to_string()))
        }),
        native("to_number", 1, to_number),
        // string
        native("len", 1, string_len),
        native("read_file", 1, read_file),
        native("write_file", 2, write_file),
        native("append_file", 2, append_file),
    ]
}

fn native(
    name: &'static str,
    arity: usize,
    func: fn(&Vec<Lit>) -> Result<Lit, RuntimeError>,
) -> (String, Lit) {
    (
        name.to_string(),
        Lit::NativeFunction(NativeFunction { name, arity, func }),
    )
}
