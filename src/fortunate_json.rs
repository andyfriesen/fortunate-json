pub mod parse;

use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::str::FromStr;

pub use parse::parse;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn as_string(&self) -> Result<&String, DecodeError> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(DecodeError {})
        }
    }

    pub fn as_float(&self) -> Result<f32, DecodeError> {
        if let Value::Number(n) = self {
            Ok(*n)
        } else {
            Err(DecodeError {})
        }
    }

    pub fn as_array(&self) -> Result<&Vec<Value>, DecodeError> {
        if let Value::Array(a) = self {
            Ok(a)
        } else {
            Err(DecodeError {})
        }
    }

    pub fn as_object(&self) -> Result<&HashMap<String, Value>, DecodeError> {
        if let Value::Object(hm) = self {
            Ok(hm)
        } else {
            Err(DecodeError {})
        }
    }
}

pub fn extract_field<T>(
    o: &HashMap<String, Value>,
    key: &str,
    res: &mut T,
) -> Result<(), DecodeError>
where
    T: FromJSON,
{
    let v = match o.get(key) {
        None => return Err(DecodeError {}),
        Some(a) => a,
    };

    T::from_json(v, res)?;

    Ok(())
}

pub fn extract_optional_field<T>(
    o: &HashMap<String, Value>,
    key: &str,
    res: &mut Option<T>,
) -> Result<(), DecodeError>
where
    T: FromJSON + Default,
{
    let v = match o.get(key) {
        None => return Ok(()),
        Some(a) => a,
    };

    let mut r = Default::default();

    T::from_json(v, &mut r)?;

    *res = Some(r);

    Ok(())
}

#[derive(Debug)]
pub struct DecodeError;

pub trait FromJSON {
    // fn from_json(v: &Value) -> Result<Self, DecodeError>;
    fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError>;
}

impl FromJSON for String {
    fn from_json(v: &Value, res: &mut String) -> Result<(), DecodeError> {
        let s = v.as_string()?;
        res.clone_from(s);
        Ok(())
    }
}

impl FromJSON for f32 {
    fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError> {
        let n = v.as_float()?;
        *res = n;
        Ok(())
    }
}

impl<T> FromJSON for Vec<T>
where
    T: FromJSON + Default,
{
    fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError> {
        let a = v.as_array()?;
        res.clear();
        res.reserve_exact(a.len());

        for elem in a {
            let mut e = Default::default();
            FromJSON::from_json(elem, &mut e)?;
            res.push(e);
        }

        Ok(())
    }
}

impl<K, V> FromJSON for HashMap<K, V>
where
    K: FromJSON + FromStr + Eq + Hash,
    V: FromJSON + Default,
{
    fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError> {
        let hm = v.as_object()?;

        res.clear();

        for (k, v) in hm {
            let key = match FromStr::from_str(k.as_str()) {
                Ok(k) => k,
                Err(_) => return Err(DecodeError {}), // FIXME
            };

            let mut value = Default::default();
            FromJSON::from_json(v, &mut value)?;

            res.insert(key, value);
        }

        Ok(())
    }
}

impl<T> FromJSON for Option<T>
where
    T: FromJSON + Default,
{
    fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError> {
        if let Value::Null = v {
            *res = None;
        } else {
            let mut r = Default::default();
            FromJSON::from_json(v, &mut r)?;
            *res = Some(r);
        }
        Ok(())
    }
}
