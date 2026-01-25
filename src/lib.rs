///! Parser for a simple JSON-like format that doesn't require quotes around keys and
///! allows trailing commas (but requires non at all).

#[cfg(feature = "json")]
pub mod json;
pub mod parser;
pub mod value;

pub use parser::{parse, parse_str};
pub use value::{List, Object, Value, SpellConfig};

use std::collections::HashMap;

use klex::{Loc, Token};
use thiserror::Error;

/// The map type when the `preserve_order` feature is on
#[cfg(feature = "preserve_order")]
pub type MapT = indexmap::IndexMap<String, Value>;
/// The map type when the `preserve_order` feature is off
#[cfg(not(feature = "preserve_order"))]
pub type MapT = HashMap<String, Value>;

/// Something went wrong?!
#[derive(Debug, Error, PartialEq)]
pub enum GonError {
    /// An error from the tokenizer `klex`
    #[error("couldn't tokenize")]
    LexerErr(#[from] klex::KlexError),
    /// Attempted to parse the empty string
    #[error("no value present")]
    NoValueErr,
    /// An invalid value
    #[error(
        "invalid value: '{0}' at {1}\n\tExpected one of: None, \"...\", <number>, true/false, [values], {{key: value}}"
    )]
    InvalidValue(String, Loc),
    /// An unexpected token
    #[error("unexpected token: '{0:?}' at {1}")]
    UnexpectedToken(Token, Loc),
    /// A missing : after an object key
    #[error("missing colon : after key '{0}' at {1}")]
    MissingColon(String, Loc),
    /// A key within an object without a value
    #[error("missing value after '{0}:' at {1}")]
    MissingValue(String, Loc),
    /// A delimiter wasn't closed
    #[error("unclosed delimiter: missing '{0:?}' which was opened at {1}")]
    UnclosedDelimiter(char, Loc),
    /// There are leftover tokens after parsing everything
    #[error("leftover tokens starting with '{0:?}' at {1}")]
    LeftoverTokens(Token, Loc),
}

#[cfg(test)]
mod tests {
    use super::parser::*;
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(parse_str(""), Err(GonError::NoValueErr));
    }

    #[test]
    fn single_value_none() {
        assert_eq!(parse_str("None"), Ok(Value::None));
        assert_eq!(parse_str("none"), Ok(Value::None));
        assert_eq!(parse_str("NONE"), Ok(Value::None));
        assert_eq!(parse_str("Null"), Ok(Value::None));
        assert_eq!(parse_str("null"), Ok(Value::None));
        assert_eq!(parse_str("NULL"), Ok(Value::None));
    }

    #[test]
    fn single_value_str() {
        assert_eq!(parse_str("\"hello\""), Ok(Value::Str{ s: "hello".into(), raw: false }));
    }

    #[test]
    fn single_value_num() {
        assert_eq!(parse_str("3.14"), Ok(Value::Num("3.14".into())));
        assert_eq!(parse_str("0"), Ok(Value::Num("0".into())));
        assert_eq!(parse_str("-99999"), Ok(Value::Num("-99999".into())));
    }

    #[test]
    fn single_value_bool() {
        assert_eq!(parse_str("true"), Ok(Value::Bool(true)));
        assert_eq!(parse_str("false"), Ok(Value::Bool(false)));
    }

    #[test]
    fn single_value_obj() {
        assert_eq!(parse_str("{}"), Ok(Value::Obj(MapT::new())));
        let a = Value::Obj(HashMap::from([(
            String::from("pi"),
            Value::Num(String::from("3.14")),
        )]));
        assert_eq!(parse_str("{pi: 3.14}"), Ok(a));
        let b = Value::Obj(HashMap::from([(
            String::from("name"),
            Value::Str{ s: "gon".into(), raw: false },
        )]));
        assert_eq!(
            parse_str("  {\n    name:\n\t\"gon\"\n\n\n\t\t}"),
            Ok(b)
        );
    }
    #[test]
    fn single_value_list() {
        assert_eq!(parse_str("[]"), Ok(Value::List(Vec::new())));
        assert_eq!(
            parse_str("[2.71]"),
            Ok(Value::List(vec![Value::Num(String::from("2.71"))]))
        );
        assert_eq!(
            parse_str("[\n\nfalse\t,]"),
            Ok(Value::List(vec![Value::Bool(false)]))
        );
    }

    #[test]
    fn many_values() {
        let name = Value::Obj(HashMap::from([
            (String::from("first"), Value::Str{ s: "John".into(), raw: false }),
            (String::from("last"), Value::Str{ s: "Doe".into(), raw: false }),
        ]));
        let address = Value::Obj(HashMap::from([
            (String::from("street"), Value::Str{ s: "Wood Way".into(), raw: false }),
            (String::from("house"), Value::Num(String::from("-9_000"))),
        ]));
        let friends = Value::List(vec![
            Value::Obj(HashMap::from([
                (String::from("name"), Value::Str{ s: "Alice".into(), raw: false }),
            ])),
            Value::Obj(HashMap::from([
                (String::from("name"), Value::Str{ s: "Bob".into(), raw: false }),
            ])),
        ]);
        let obj = Value::Obj(HashMap::from([
            (String::from("id"), Value::Num(String::from("456"))),
            (String::from("name"), name),
            (String::from("address"), address),
            (String::from("alive"), Value::Bool(true)),
            (String::from("friends"), friends),
        ]));
        assert_eq!(
            parse_str(
                r#"{
            id: 456,
            name: {
                first: "John",
                last: "Doe",
            },
            address: {
                street: "Wood Way",
                house: -9_000,
            },
            alive: true,
            friends: [
                {name: "Alice",},
                {
                    name: "Bob"
                },
            ]
        }"#
            ),
            Ok(obj)
        );
    }
}
