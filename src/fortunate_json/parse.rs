use std::collections::hash_map::HashMap;

use crate::fortunate_json::Value;

#[derive(Debug, PartialEq)]
pub struct ParseError(pub String);

#[derive(Debug, PartialEq)]
enum Token<'a> {
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Colon,
    Comma,
    Identifier(&'a [u8]),
    String(String),
    Number(f32),
}

struct Lexer<'a> {
    s: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(s: &[u8]) -> Lexer<'_> {
        Lexer { s: s, pos: 0 }
    }

    fn eof(&self) -> bool {
        self.pos >= self.s.len()
    }

    fn advance(&mut self) {
        if !self.eof() {
            self.pos += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.eof() {
            None
        } else {
            Some(self.s[self.pos])
        }
    }

    fn take_while<T>(&mut self, pred: T) -> &'a [u8]
    where
        T: Fn(u8) -> bool,
    {
        let start_pos = self.pos;
        while let Some(ch) = self.peek_byte() {
            if pred(ch) {
                self.advance();
            } else {
                break;
            }
        }

        &self.s[start_pos..self.pos]
    }

    fn skip_whitespace(&mut self) {
        self.take_while(|ch| ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n');
    }

    fn is_identifier_start(b: u8) -> bool {
        (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z') || b == b'_'
    }

    fn is_identifier_char(b: u8) -> bool {
        Self::is_identifier_start(b) || Self::is_digit(b)
    }

    fn is_digit(b: u8) -> bool {
        b >= b'0' && b <= b'9'
    }

    fn token(&mut self) -> Result<Token<'a>, ParseError> {
        self.skip_whitespace();

        if self.eof() {
            return Err(ParseError("Unexpected end of file".to_owned()));
        }

        let byte = self.peek_byte().unwrap();

        let result = match byte as char {
            '[' => {
                self.advance();
                Token::OpenBracket
            }
            ']' => {
                self.advance();
                Token::CloseBracket
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            ':' => {
                self.advance();
                Token::Colon
            }
            '{' => {
                self.advance();
                Token::OpenBrace
            }
            '}' => {
                self.advance();
                Token::CloseBrace
            }
            '-' => Token::Number(self.lex_number()?),
            d if d.is_digit(10) => Token::Number(self.lex_number()?),

            '"' => {
                // First, just find the extent of the string literal
                self.advance();
                let start_pos = self.pos;
                loop {
                    match self.peek_byte() {
                        None => {
                            return Err(ParseError(
                                "Unexpected end of file while parsing string literal".to_owned(),
                            ))
                        }
                        Some(b) => match b as char {
                            '\n' => {
                                return Err(ParseError(
                                    "Unexpected newline while parsing string literal".to_owned(),
                                ))
                            }
                            '\\' => {
                                self.advance();
                                if let None = self.peek_byte() {
                                    return Err(ParseError("Unexpected end of file while parsing string literal escape sequence".to_owned()));
                                }
                            }
                            '"' => break,
                            _ => self.advance(),
                        },
                    }
                }
                let end_pos = self.pos;

                let res = &self.s[start_pos..end_pos];

                self.advance();
                Token::String(Self::parse_string(res)?)
            }
            _ if Self::is_identifier_start(byte) => {
                Token::Identifier(self.take_while(Self::is_identifier_char))
            }
            _ => {
                return Err(ParseError(format!(
                    "Unexpected character '{}'",
                    byte as char
                )));
            }
        };

        self.skip_whitespace();

        Ok(result)
    }

    fn lex_number(&mut self) -> Result<f32, ParseError> {
        let negative = if self.peek_byte() == Some(b'-') {
            self.advance();
            true
        } else {
            false
        };

        if self.eof() {
            return Err(ParseError("Unexpected EOF while parsing number".to_owned()));
        }

        let start_offset = self.pos;

        // TODO: leading zeroes are not ok.  Only one leading zero before a decimal is allowed.

        self.take_while(&Self::is_digit);

        if self.peek_byte() == Some(b'.') {
            self.advance();

            self.take_while(&Self::is_digit);
        }

        if let Some(ch) = self.peek_byte() {
            if ch == b'e' || ch == b'E' {
                self.advance();

                let maybe_sign = self.peek_byte();
                if maybe_sign == Some(b'-') || maybe_sign == Some(b'+') {
                    self.advance();
                }

                self.take_while(&Self::is_digit);
            }
        }

        let end_effset = self.pos;

        let res = std::str::from_utf8(&self.s[start_offset..end_effset])
            .unwrap()
            .parse::<f32>()
            .unwrap();

        Ok(if negative { -(res as f32) } else { res as f32 })
    }

    fn parse_hex_digit(d: char) -> Result<usize, ParseError> {
        const DIGITS: &str = "01234567890ABCDEF";
        if let Some(i) = DIGITS.find(d.to_ascii_uppercase()) {
            Ok(i)
        } else {
            Err(ParseError(format!(
                "Bad hex digit '{}' in unicode escape",
                d
            )))
        }
    }

    fn parse_hex(d1: char, d2: char, d3: char, d4: char) -> Result<u32, ParseError> {
        let a1 = Self::parse_hex_digit(d1)?;
        let a2 = Self::parse_hex_digit(d2)?;
        let a3 = Self::parse_hex_digit(d3)?;
        let a4 = Self::parse_hex_digit(d4)?;
        Ok((a1 << 24 | a2 << 16 | a3 << 8 | a4) as u32)
    }

    fn parse_string(s: &[u8]) -> Result<String, ParseError> {
        let mut res = String::new();
        res.reserve_exact(s.len());

        let st = std::str::from_utf8(s).unwrap();

        let mut chars = st.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                let n = chars.next().unwrap(); // Should be ok.  Lexer should handle this.
                res.push(match n {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'u' => {
                        let gch = |ch: &mut std::str::Chars| match ch.next() {
                            None => Err(ParseError(
                                "Unexpected EOF when parsing unicode escape in string literal"
                                    .to_owned(),
                            )),
                            Some(c) => Ok(c),
                        };
                        let d1 = gch(&mut chars)?;
                        let d2 = gch(&mut chars)?;
                        let d3 = gch(&mut chars)?;
                        let d4 = gch(&mut chars)?;
                        char::from_u32(Self::parse_hex(d1, d2, d3, d4)?).unwrap()
                    }
                    c => c,
                });
            } else {
                res.push(ch);
            }
        }

        Ok(res)
    }

    fn rest(&self) -> &'a [u8] {
        &self.s[self.pos..self.s.len()]
    }
}

pub fn parse(s: &str) -> Result<Value, ParseError> {
    let mut lexer = Lexer::new(s.as_bytes());
    let v = parse_(&mut lexer)?;

    lexer.skip_whitespace();

    if !lexer.eof() {
        Err(ParseError(format!(
            "Extra goop at the end of the file: {:?}",
            lexer.rest()
        )))
    } else {
        Ok(v)
    }
}

const NULL_TOKEN: &'static [u8] = b"null";
const TRUE_TOKEN: &'static [u8] = b"true";
const FALSE_TOKEN: &'static [u8] = b"false";

fn parse_(lexer: &mut Lexer) -> Result<Value, ParseError> {
    let token = lexer.token()?;
    dbg!("token '{:?}'", &token);

    match token {
        Token::Identifier(i) if i == NULL_TOKEN => Ok(Value::Null),
        Token::Identifier(i) if i == TRUE_TOKEN => Ok(Value::Boolean(true)),
        Token::Identifier(i) if i == FALSE_TOKEN => Ok(Value::Boolean(false)),
        Token::String(s) => Ok(Value::String(s)),
        Token::Number(n) => Ok(Value::Number(n)),
        Token::OpenBracket => {
            let mut arr = Vec::new();
            loop {
                let val = parse_(lexer)?;
                arr.push(val);

                let next = lexer.token()?;
                match next {
                    Token::CloseBracket => break,
                    Token::Comma => continue,
                    _ => {
                        return Err(ParseError(format!(
                            "Expected ',' or ']' but got '{:?}'",
                            next
                        )));
                    }
                }
            }

            Ok(Value::Array(arr))
        }
        Token::OpenBrace => {
            let mut obj = HashMap::new();

            loop {
                let key = match parse_(lexer)? {
                    Value::String(s) => s,
                    other => {
                        return Err(ParseError(format!(
                            "Object keys must be strings.  Got {:?}",
                            other
                        )))
                    }
                };

                let colon = lexer.token()?;
                if Token::Colon != colon {
                    return Err(ParseError(format!("Expected colon but got '{:?}'", colon)));
                }

                let val = parse_(lexer)?;

                obj.insert(key, val);

                let comma_or_brace = lexer.token()?;
                if comma_or_brace == Token::CloseBrace {
                    break;
                } else if comma_or_brace != Token::Comma {
                    return Err(ParseError(format!(
                        "Expected comma or brace but got '{:?}'",
                        comma_or_brace
                    )));
                }
            }

            Ok(Value::Object(obj))
        }

        t => Err(ParseError(format!("Unknown token '{:?}'", t))),
    }
}
