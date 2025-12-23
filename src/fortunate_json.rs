pub mod parse;

use parse::ParseError;
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

pub fn extract_field<T>(o: &HashMap<String, Value>, key: &str) -> Result<T, DecodeError>
where
    T: FromJSON,
{
    let v = match o.get(key) {
        None => return Err(DecodeError {}),
        Some(a) => a,
    };

    T::from_json(v)
}

pub fn extract_optional_field<T>(
    o: &HashMap<String, Value>,
    key: &str,
) -> Result<Option<T>, DecodeError>
where
    T: FromJSON + Default,
{
    match o.get(key) {
        None => Ok(None),
        Some(a) => Ok(Some(T::from_json(a)?)),
    }
}

#[derive(Debug, PartialEq)]
pub struct DecodeError;

pub trait FromJSON
where
    Self: Sized,
{
    fn from_json(v: &Value) -> Result<Self, DecodeError>;
}

impl FromJSON for String {
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let s = v.as_string()?;
        Ok(s.clone())
    }
}

impl FromJSON for f32 {
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let n = v.as_float()?;
        Ok(n)
    }
}

impl FromJSON for u32 {
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let n = v.as_float()?;
        if n != n.floor() {
            Err(DecodeError {})
        } else {
            Ok(n as u32)
        }
    }
}

impl<T> FromJSON for Vec<T>
where
    T: FromJSON + Default,
{
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let a = v.as_array()?;
        let mut res = Vec::with_capacity(a.len());

        for elem in a {
            res.push(FromJSON::from_json(elem)?);
        }

        Ok(res)
    }
}

impl<T> FromJSON for std::collections::HashSet<T>
where
    T: FromJSON + Default + Eq + Hash,
{
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let a = v.as_array()?;
        let mut res = Self::with_capacity(a.len());

        for elem in a {
            res.insert(FromJSON::from_json(elem)?);
        }

        Ok(res)
    }
}

impl<K, V> FromJSON for HashMap<K, V>
where
    K: FromJSON + FromStr + Eq + Hash,
    V: FromJSON + Default,
{
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let hm = v.as_object()?;

        let mut res = Self::with_capacity(hm.len());

        for (k, v) in hm {
            // let key = k.as_str()?;
            let key = match FromStr::from_str(k) {
                Ok(k) => k,
                Err(_) => return Err(DecodeError {}), // FIXME: Better error here
            };

            res.insert(key, FromJSON::from_json(v)?);
        }

        Ok(res)
    }
}

impl<T> FromJSON for Option<T>
where
    T: FromJSON + Default,
{
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        if let Value::Null = v {
            Ok(None)
        } else {
            Ok(Some(FromJSON::from_json(v)?))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum JSONError {
    ParseError(String),
    DecodeError,
}

impl From<ParseError> for JSONError {
    fn from(p: ParseError) -> JSONError {
        let ParseError(msg) = p;
        JSONError::ParseError(msg)
    }
}

impl From<DecodeError> for JSONError {
    fn from(_: DecodeError) -> JSONError {
        JSONError::DecodeError {}
    }
}

pub fn decode<T>(s: &str) -> Result<T, JSONError>
where
    T: FromJSON + Default,
{
    let v = parse(s)?;

    Ok(FromJSON::from_json(&v)?)
}
