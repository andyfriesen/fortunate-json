use crate::fortunate_json::{
    decode, extract_field, parse, DecodeError, FromJSON, JSONError, Value,
};
use std::collections::hash_map::HashMap;

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
fn busted_unicode_escape() {
    assert_eq!(
        Err(JSONError::ParseError(
            "Unexpected EOF when parsing unicode escape in string literal".to_owned()
        )),
        decode::<String>("\"\\u00\"")
    );
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

#[derive(Debug, PartialEq, Default)]
struct Point {
    x: f32,
    y: f32,
}

impl FromJSON for Point {
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let o = v.as_object()?;

        Ok(Point {
            x: extract_field(o, "x")?,
            y: extract_field(o, "y")?,
        })
    }
}

#[test]
fn unpack_struct() {
    let json = "{\"x\": 3.14, \"y\": 1.161}";

    let parsed = parse(json).unwrap();

    let p: Point = FromJSON::from_json(&parsed).unwrap();

    assert_eq!(p, Point { x: 3.14, y: 1.161 });
}

#[derive(Debug, PartialEq, Default)]
struct Mesh {
    points: Vec<Point>,
    indeces: Vec<u32>,
}

impl FromJSON for Mesh {
    fn from_json(v: &Value) -> Result<Self, DecodeError> {
        let o = v.as_object()?;

        Ok(Mesh {
            points: extract_field(o, "points")?,
            indeces: extract_field(o, "indeces")?,
        })
    }
}

#[test]
fn unpack_vec() {
    let expected = Mesh {
        points: vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 10.0 },
            Point { x: 10.0, y: 0.0 },
        ],
        indeces: vec![0, 2, 1],
    };

    let json = "{\"points\":[{\"x\":0.0, \"y\":0.0}, {\"x\": 0.0, \"y\": 10}, {\"x\": 1e1, \"y\": 0.0}], \"indeces\":[0, 2, 1]}";

    let parsed = parse(json).unwrap();

    let m: Mesh = FromJSON::from_json(&parsed).unwrap();

    assert_eq!(m, expected);
}

#[test]
fn unpack_map() {}
