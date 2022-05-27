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
    fn as_string(&self) -> Result<&String, DecodeError> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(DecodeError {})
        }
    }

    fn as_float(&self) -> Result<f32, DecodeError> {
        if let Value::Number(n) = self {
            Ok(*n)
        } else {
            Err(DecodeError {})
        }
    }

    fn as_array(&self) -> Result<&Vec<Value>, DecodeError> {
        if let Value::Array(a) = self {
            Ok(a)
        } else {
            Err(DecodeError {})
        }
    }

    fn as_object(&self) -> Result<&HashMap<String, Value>, DecodeError> {
        if let Value::Object(hm) = self {
            Ok(hm)
        } else {
            Err(DecodeError {})
        }
    }
}

pub fn extract_field<T>(o: &HashMap<String, Value>, key: &str, res: &mut T) -> Result<(), DecodeError> where T : FromJSON {
    let v = match o.get(key) {
        None => return Err(DecodeError{}),
        Some(a) => a
    };

    T::from_json(v, res)?;

    Ok(())
}

pub fn extract_optional_field<T>(o: &HashMap<String, Value>, key: &str, res: &mut T) -> Result<(), DecodeError> where T : FromJSON {
    let v = match o.get(key) {
        None => return Ok(()),
        Some(a) => a
    };

    T::from_json(v, res)?;

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

// TODO: Like a billion tests around error conditions.

#[test]
fn prims() {
    assert_eq!(Ok(Value::Null), parse("null"));
    assert_eq!(Ok(Value::Boolean(true)), parse("true"));
    assert_eq!(Ok(Value::Boolean(false)), parse("false"));
}

#[test]
fn simple_array() {
    let expected = vec![Value::Boolean(true), Value::Boolean(false), Value::Null];
    assert_eq!(Ok(Value::Array(expected)), parse("[true,false,null]"));
}

#[test]
fn nested_array() {
    let expected = Value::Array(vec![
        Value::Boolean(true),
        Value::Array(vec![Value::Boolean(false), Value::Null]),
    ]);

    assert_eq!(Ok(expected), parse("[true,[false,null]]"));
}

#[test]
fn whitespace() {
    let expected = Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]);

    assert_eq!(Ok(expected), parse(" [ true , false ] "));
}

#[test]
fn string() {
    let expected = Value::String("Hello World!".to_owned());

    assert_eq!(Ok(expected), parse("\"Hello World!\""));
}

#[test]
fn japanese() {
    let expected = Value::String("こんにちは".to_owned());

    assert_eq!(Ok(expected), parse("\"こんにちは\""));
}

#[test]
fn string_with_newline() {
    let expected = Value::String("Hello\nWorld".to_owned());

    assert_eq!(Ok(expected), parse("\"Hello\\nWorld\""));
}

#[test]
fn object() {
    let expected = Value::Object(HashMap::from([
        ("foo".to_owned(), Value::String("bar".to_owned())),
        ("baz".to_owned(), Value::Boolean(true)),
    ]));

    assert_eq!(Ok(expected), parse("{\"foo\": \"bar\", \"baz\" : true}"))
}

#[test]
fn integers() {
    let expected = Value::Array(vec![
        Value::Number(0.0),
        Value::Number(2.0),
        Value::Number(4.0),
        Value::Number(8.0),
        Value::Number(128.0),
        Value::Number(65535.0),
        Value::Number(-131085.0),
    ]);

    assert_eq!(Ok(expected), parse("[0, 2, 4 , 8, 128 \t ,65535, -131085]"));
}

#[test]
fn float() {
    let expected = Value::Number(3.141);

    assert_eq!(Ok(expected), parse("3.141"));
}

#[test]
fn exponential_notation() {
    let expected = Value::Array(vec![Value::Number(1000.0), Value::Number(0.00055)]);

    assert_eq!(Ok(expected), parse("[1e3, 5.5e-4]"));
}

#[test]
fn unpack_struct() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: f32,
        y: f32
    }

    impl FromJSON for Point {
        fn from_json(v: &Value, res: &mut Self) -> Result<(), DecodeError> {
            let o = v.as_object()?;

            extract_field(o, "x", &mut res.x)?;
            extract_field(o, "y", &mut res.y)?;

            Ok(())
        }
    }

    let mut p = Point{x: 0.0, y: 0.0};

    let json = "{\"x\": 3.14, \"y\": 1.161}";

    let parsed = parse(json).unwrap();

    FromJSON::from_json(&parsed, &mut p).unwrap();

    assert_eq!(p, Point{x: 3.14, y:1.161});
}
