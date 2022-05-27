
use std::collections::hash_map::HashMap;
use crate::fortunate_json::{Value, DecodeError, FromJSON, parse, extract_field};

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
