use std::iter::Peekable;

use klex::{Lexer, Loc, RichToken, Token};

use crate::{GonError, List, Object, Value};

struct TokenIter {
    inner: Peekable<std::vec::IntoIter<RichToken>>,
    loc: Loc,
}

/// Try to parse the given `&str` into a gon [`Value`]. This is just a short-hand:
/// `parse_str(s) = parse(s.chars())`. See [`parse`].
/// # Usage example
/// ```rust
/// use gon::{parse_str, Value};
/// let src = "[1, 2, 3]";
/// assert_eq!(
///     Ok(Value::List(vec![
///         Value::Num(1.to_string()),
///         Value::Num(2.to_string()),
///         Value::Num(3.to_string()),
///     ])),
///     parse_str(src),
/// );
/// ```
pub fn parse_str(src: &str) -> Result<Value, GonError> {
    parse(src.chars())
}

/// Try to parse the given char iterator into a gon [`Value`].
/// # Usage example
/// ```rust
/// use gon::{MapT, parse, Value};
/// let src = "{a: 1, b: []}".chars();
/// assert_eq!(
///     Ok(Value::Obj(MapT::from([
///         ("a".to_string(), Value::Num(1.to_string())),
///         ("b".to_string(), Value::List(vec![])),
///     ]))),
///     parse(src),
/// );
/// ```
pub fn parse<I: Iterator<Item = char>>(src: I) -> Result<Value, GonError> {
    let tokens = Lexer::from_iter(src, 0)
        .lex()
        .map_err(|e| GonError::LexerErr(e))?
        .into_iter()
        .peekable();
    let mut token_iter = TokenIter {
        inner: tokens,
        loc: Loc::start_of_file(0),
    };
    let value = next_value(&mut token_iter)?;
    if let Some(tok) = token_iter.next() {
        Err(GonError::LeftoverTokens(tok.inner, token_iter.loc))
    } else {
        Ok(value)
    }
}

fn next_value(tokens: &mut TokenIter) -> Result<Value, GonError> {
    let Some(first_token) = tokens.next() else {
        return Err(GonError::NoValueErr);
    };
    match first_token.inner {
        Token::Sym(sym) => {
            let sym_lower = sym.to_lowercase();
            if sym_lower == "none" || sym_lower == "null" {
                Ok(Value::None)
            } else if sym_lower == "true" {
                Ok(Value::Bool(true))
            } else if sym_lower == "false" {
                Ok(Value::Bool(false))
            } else if sym_lower == "r" {
                if let Some(Token::Str(string)) = tokens.peek().map(|rt| &rt.inner) {
                    let value = Value::Str {
                        s: string.to_owned(),
                        raw: true,
                    };
                    tokens.next();
                    Ok(value)
                } else {
                    Err(GonError::InvalidValue(sym, first_token.loc))
                }
            } else {
                Err(GonError::InvalidValue(sym, first_token.loc))
            }
        }
        Token::Str(string) => Ok(Value::Str {
            s: string,
            raw: false,
        }),
        Token::Num(num) => Ok(Value::Num(num)),
        Token::Dash => {
            if let Some(Token::Num(ns)) = tokens.peek().map(|t| &t.inner) {
                let value = Value::Num(format!("-{ns}"));
                tokens.next();
                Ok(value)
            } else {
                Err(GonError::UnexpectedToken(Token::Dash, first_token.loc))
            }
        }
        Token::LBrace => {
            let mut map = crate::MapT::new();
            let opening_loc = tokens.loc;
            loop {
                if matches![tokens.peek().map(|t| &t.inner), Some(Token::RBrace)] {
                    tokens.next();
                    break;
                }
                let Some((key, value)) = next_key_value_pair(tokens)? else {
                    return Err(GonError::UnclosedDelimiter('}', opening_loc));
                };
                map.insert(key, value);
                consume_optional_comma(tokens);
            }
            Ok(Value::Obj(map))
        }
        Token::LBrack => {
            let mut list = Vec::new();
            let opening_loc = tokens.loc;
            loop {
                if matches![tokens.peek().map(|t| &t.inner), Some(Token::RBrack)] {
                    tokens.next();
                    break;
                }
                let Ok(value) = next_value(tokens) else {
                    return Err(GonError::UnclosedDelimiter(']', opening_loc));
                };
                list.push(value);
                consume_optional_comma(tokens);
            }
            Ok(Value::List(list))
        }
        token => Err(GonError::UnexpectedToken(token, first_token.loc)),
    }
}

fn consume_optional_comma(tokens: &mut TokenIter) {
    if let Some(rt) = tokens.peek() {
        if matches![rt.inner, Token::Comma] {
            tokens.next();
        }
    }
}

fn next_key_value_pair(tokens: &mut TokenIter) -> Result<Option<(String, Value)>, GonError> {
    let Some(token) = tokens.next().map(|t| t.inner) else {
        return Ok(None);
    };
    let key = match token {
        Token::Str(s) | Token::Num(s) | Token::Sym(s) => s,
        Token::Comment(_) => return Err(GonError::UnexpectedToken(token, tokens.loc)),
        otherwise => otherwise.spelling(),
    };
    let Some(Token::Colon) = tokens.next().map(|t| t.inner) else {
        return Err(GonError::MissingColon(key, tokens.loc));
    };
    let Ok(value) = next_value(tokens) else {
        return Err(GonError::MissingValue(key, tokens.loc));
    };
    Ok(Some((key, value)))
}

impl TokenIter {
    pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        self.inner.peek()
    }
}

impl Iterator for TokenIter {
    type Item = RichToken;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rt) = self.inner.next() {
            self.loc = rt.loc;
            Some(rt)
        } else {
            None
        }
    }
}
