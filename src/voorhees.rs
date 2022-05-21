#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    // Number(f32)
    String(String),
    Array(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub struct ParseError(String);

#[derive(Debug)]
enum Token<'a> {
    Identifier(&'a [u8]),
    Symbol(u8),
    String(String),
}

struct Lexer<'a> {
    s: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(s: &[u8]) -> Lexer {
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
        self.take_while(|ch| {
            ch == ' ' as u8 || ch == '\t' as u8 || ch == '\r' as u8 || ch == '\n' as u8
        });
    }

    fn is_identifier_start(b: u8) -> bool {
        (b >= 'a' as u8 && b <= 'z' as u8) || (b >= 'A' as u8 && b <= 'Z' as u8) || b == '_' as u8
    }

    fn is_identifier_char(b: u8) -> bool {
        Self::is_identifier_start(b) || (b >= '0' as u8 && b <= '9' as u8)
    }

    fn token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();

        if self.eof() {
            return Err(ParseError("Unexpected end of file".to_owned()));
        }

        let next_char = |lexer: &mut Self| {
            let start_pos = lexer.pos;
            lexer.advance();
            Token::Symbol(lexer.s[start_pos])
        };

        let byte = self.peek_byte().unwrap();

        let result = match byte as char {
            '[' => next_char(self),
            ']' => next_char(self),
            ',' => next_char(self),
            ':' => next_char(self),
            '{' => next_char(self),
            '}' => next_char(self),
            '"' => {
                // std::str::from_utf8(self.s)

                // TODO: Scan for non-escaped end quote.
                // Parse utf8.
                // Process escape sequences in-place?

                // Alternate: Can we parse utf8 into a &str without a copy?
                // Then walk chars to parse escape sequences

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

                let res = std::str::from_utf8(&self.s[start_pos..end_pos])
                    .unwrap()
                    .to_owned();
                self.advance();
                Token::String(res)
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
        Token::Symbol(t) if t == '[' as u8 => {
            let mut arr = Vec::new();
            loop {
                let val = parse_(lexer)?;
                arr.push(val);

                let next = lexer.token()?;
                match next {
                    Token::Symbol(t) if t == ']' as u8 => break,
                    Token::Symbol(t) if t == ',' as u8 => continue,
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
        Token::String(s) => Ok(Value::String(s)),
        t => Err(ParseError(format!("Unknown token '{:?}'", t))),
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
