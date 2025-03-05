use serde_json::Value as JsonValue;

use crate::Value;

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        match value {
            Value::None => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(b),
            // FIXME
            Value::Num(n) => {
                JsonValue::Number(serde_json::Number::from_f64(n.parse().unwrap()).unwrap())
            }
            Value::Str(s) => JsonValue::String(s),
            Value::List(xs) => JsonValue::Array(xs.into_iter().map(Value::into).collect()),
            Value::Obj(obj) => {
                JsonValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}
