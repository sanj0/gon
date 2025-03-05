#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    None,
    Str(String),
    Num(String),
    Bool(bool),
    Obj(crate::MapT),
    List(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    inner: crate::MapT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    inner: Vec<Value>,
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        let Self::Num(num) = self else {
            return None;
        };
        num.parse().ok()
    }

    pub fn as_i128(&self) -> Option<i128> {
        let Self::Num(num) = self else {
            return None;
        };
        num.parse().ok()
    }
}
