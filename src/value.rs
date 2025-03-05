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
