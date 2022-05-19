
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

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

fn is_identifier(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

fn peek(chars: &CharIndices) -> Option<(usize, char)> {
    let mut clone = chars.clone();
    clone.next()
}

fn eat_whitespace(chars: &mut CharIndices) -> Option<usize> {
    while let Some((ofs, ch)) = peek(chars) {
        if is_whitespace(ch) {
            chars.next();
        } else {
            return Some(ofs);
        }
    }

    return None;
}

fn slice(s: &str, start: usize, end: Option<usize>) -> (&str, &str, &str) {
    let (first, second) = s.split_at(start);
    if let Some(e) = end {
        let (third, fourth) = second.split_at(e - start);
        (first, third, fourth)
    } else {
        (first, second, "")
    }
}

fn lex(s: &str) -> (&str, &str) {
    if 0 == s.len() {
        return (s, s);
    }

    let mut chars = s.char_indices();
    let start_offset = match eat_whitespace(&mut chars) {
        Some(o) => o,
        None => return ("", "")
    };

    let (char_offset, c) = chars.next().unwrap();

    if let Some(_) = "{}[]:,".find(c) {
        if let Some((end_offset, c)) = chars.next() {
            let (f, ch, rest) = slice(s, start_offset, Some(end_offset));
            println!("symbol '{}' rest='{}'", ch, rest);
            (ch, rest)
        } else {
            (s, "")
        }
        
    } else if is_identifier(c) {
        while let Some((ofs, ch)) = chars.next() {
            if !is_identifier(ch) {
                let (a, b, c) = slice(s, start_offset, Some(ofs));
                return (b, c);
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
            rest = rest_;

            arr.push(value);

            if rest_.len() == 0 {
                return Err(ParseError("Unexpected end of document".to_owned()));
            }

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

#[test]
fn whitespace() {
    let expected = Value::Array(vec![
        Value::Boolean(true),
        Value::Boolean(false)
    ]);

    assert_eq!(Ok(expected), parse(" [ true , false ] "));
}
