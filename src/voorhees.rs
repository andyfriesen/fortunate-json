use std::str::CharIndices;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    // Number(f32)
    Array(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub struct ParseError(String);

fn is_identifier(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

fn lex(s: &str) -> (&str, &str) {
    if 0 == s.len() {
        return (s, s);
    }

    let mut chars = s.char_indices();
    let (_, c) = chars.next().unwrap();

    let stop = |mut chars: CharIndices| {
        if let Some((ofs, _)) = chars.next() {
            s.split_at(ofs)
        } else {
            (s, "")
        }
    };

    if let Some(_) = "{}[]:,".find(c) {
        stop(chars)
    } else if is_identifier(c) {
        while let Some((ofs, ch)) = chars.next() {
            if !is_identifier(ch) {
                return s.split_at(ofs);
            }
        }
        (s, "")
    } else {
        (s, s)
    }
}

pub fn parse(s: &str) -> Result<Value, ParseError> {
    let (v, rest) = parse_(s)?;

    if rest.len() > 0 {
        Err(ParseError(
            "Extra goop at the end of the file: ".to_owned() + rest,
        ))
    } else {
        Ok(v)
    }
}

pub fn parse_(s: &str) -> Result<(Value, &str), ParseError> {
    let (next, mut rest) = lex(s);
    println!("Next: {}", next);
    if next.len() == 0 {
        Err(ParseError("Unexpected end of document".to_owned()))
    } else if next == "null" {
        Ok((Value::Null, rest))
    } else if next == "true" {
        Ok((Value::Boolean(true), rest))
    } else if next == "false" {
        Ok((Value::Boolean(false), rest))
    } else if next == "[" {
        let mut arr = Vec::new();
        loop {
            let (value, rest_) = parse_(rest)?;

            arr.push(value);

            if rest_.len() == 0 {
                return Err(ParseError("Unexpected end of document".to_owned()));
            }

            rest = rest_;

            let (next, rest_) = lex(rest);
            rest = rest_;
            if next == "]" {
                break;
            } else if next == "," {
                continue;
            } else if next == "" {
                return Err(ParseError("Unexpected end of document".to_owned()));
            } else {
                return Err(ParseError(
                    "Expected ',' or ']' but got '".to_owned() + next + "'",
                ));
            }
        }

        Ok((Value::Array(arr), rest))
    } else {
        Err(ParseError("Unknown token '".to_owned() + next + "'"))
    }
}

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
        Value::Array(
            vec![Value::Boolean(false), Value::Null]
        )
    ]);

    assert_eq!(Ok(expected), parse("[true,[false,null]]"));
}
