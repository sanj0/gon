use std::iter::Peekable;

use klex::{Lexer, Loc, RichToken, Token};

use crate::{GonError, List, Object, Value};

struct TokenIter {
    inner: Peekable<std::vec::IntoIter<RichToken>>,
    loc: Loc,
}

pub fn parse_str(src: &str) -> Result<Option<Value>, GonError> {
    parse(src.chars())
}

pub fn parse<I: Iterator<Item = char>>(src: I) -> Result<Option<Value>, GonError> {
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

fn next_value(tokens: &mut TokenIter) -> Result<Option<Value>, GonError> {
    let Some(first_token) = tokens.next() else {
        return Ok(None);
    };
    match first_token.inner {
        Token::Sym(sym) => {
            let sym_lower = sym.to_lowercase();
            if sym_lower == "none" || sym_lower == "null" {
                Ok(Some(Value::None))
            } else if sym_lower == "true" {
                Ok(Some(Value::Bool(true)))
            } else if sym_lower == "false" {
                Ok(Some(Value::Bool(false)))
            } else {
                Err(GonError::InvalidValue(sym, first_token.loc))
            }
        }
        Token::Str(string) => Ok(Some(Value::Str(string))),
        Token::Num(num) => Ok(Some(Value::Num(num))),
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
            Ok(Some(Value::Obj(map)))
        }
        Token::LBrack => {
            let mut list = Vec::new();
            let opening_loc = tokens.loc;
            loop {
                if matches![tokens.peek().map(|t| &t.inner), Some(Token::RBrack)] {
                    tokens.next();
                    break;
                }
                let Some(value) = next_value(tokens)? else {
                    return Err(GonError::UnclosedDelimiter(']', opening_loc));
                };
                list.push(value);
                consume_optional_comma(tokens);
            }
            Ok(Some(Value::List(list)))
        }
        token => Err(GonError::UnexpectedToken(token, first_token.loc)),
    }
}

fn consume_required_comma(tokens: &mut TokenIter) -> Result<(), GonError> {
    if let Some(rt) = tokens.next() {
        if matches![rt.inner, Token::Comma] {
            return Ok(());
        }
    }
    Err(GonError::MissingComma(tokens.loc))
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
    let Token::Sym(key) = token else {
        return Err(GonError::UnexpectedToken(token, tokens.loc));
    };
    let Some(Token::Colon) = tokens.next().map(|t| t.inner) else {
        return Err(GonError::MissingColon(key, tokens.loc));
    };
    let Some(value) = next_value(tokens)? else {
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
