use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    errors::RuntimeError,
    nodes::{Lit, NativeFunction},
};

pub fn clock(_args: &Vec<Lit>) -> Result<Lit, RuntimeError> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Ok(Lit::Number(secs))
}

pub fn native_functions() -> Vec<(String, Lit)> {
    vec![native("clock", 0, clock)]
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
