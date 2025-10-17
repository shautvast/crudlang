use chrono::Utc;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone)]
pub enum Value {
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Char(char),
    Bool(bool),
    Date(Utc),
    Enum,
    Struct,
    List(Vec<Value>),
    Map(HashMap<Value, Value>),
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::I32(self)
    }
}

impl Into<Value> for i64 {
    fn into(self) -> Value {
        Value::I64(self)
    }
}

impl Into<Value> for u32 {
    fn into(self) -> Value {
        Value::U32(self)
    }
}

impl Into<Value> for u64 {
    fn into(self) -> Value {
        Value::U64(self)
    }
}

impl Into<Value> for f32 {
    fn into(self) -> Value {
        Value::F32(self)
    }
}

impl Into<Value> for f64 {
    fn into(self) -> Value {
        Value::F64(self)
    }
}

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value::String(self.to_string())
    }
}
impl Into<Value> for char {
    fn into(self) -> Value {
        Value::Char(self)
    }
}

impl Into<Value> for bool {
    fn into(self) -> Value {
        Value::Bool(self)
    }
}

impl Into<Value> for Utc {
    fn into(self) -> Value {
        Value::Date(self)
    }
}

impl Into<Value> for Vec<Value> {
    fn into(self) -> Value {
        Value::List(self)
    }
}

impl Into<Value> for HashMap<Value, Value> {
    fn into(self) -> Value {
        Value::Map(self)
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::I32(i) => Value::I32(-i),
            Value::I64(i) => Value::I64(-i),
            Value::F32(i) => Value::F32(-i),
            Value::F64(i) => Value::F64(-i),
            _ => panic!("Cannot negate {:?}", self),
        }
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
            (Value::I64(a), Value::I64(b)) => Value::I64(a + b),
            (Value::U32(a), Value::U32(b)) => Value::U32(a + b),
            (Value::U64(a), Value::U64(b)) => Value::U64(a + b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a + b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a + b),
            (Value::String(s), Value::I32(i)) => Value::String(format!("{}{}", s, i)),
            (Value::String(s), Value::I64(i)) => Value::String(format!("{}{}", s, i)),
            (Value::String(s), Value::U32(u)) => Value::String(format!("{}{}", s, u)),
            (Value::String(s), Value::U64(u)) => Value::String(format!("{}{}", s, u)),
            (Value::String(s), Value::F32(f)) => Value::String(format!("{}{}", s, f)),
            (Value::String(s), Value::F64(f)) => Value::String(format!("{}{}", s, f)),
            (Value::String(s), Value::Bool(b)) => Value::String(format!("{}{}", s, b)),
            (Value::String(s), Value::Char(c)) => Value::String(format!("{}{}", s, c)),
            (Value::String(s1), Value::String(s2)) => Value::String(format!("{}{}", s1, s2)),
            (Value::String(s1), Value::List(l)) => Value::String(format!("{}{:?}", s1, l)),
            (Value::String(s1), Value::Map(m)) => Value::String(format!("{}{:?}", s1, m)),
            //enum?
            _ => panic!("Cannot add"),
        }
    }
}

impl Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a - b),
            (Value::I64(a), Value::I64(b)) => Value::I64(a - b),
            (Value::U32(a), Value::U32(b)) => Value::U32(a - b),
            (Value::U64(a), Value::U64(b)) => Value::U64(a - b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a - b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a - b),
            //enum?
            _ => panic!("Cannot subtract"),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a * b),
            (Value::I64(a), Value::I64(b)) => Value::I64(a * b),
            (Value::U32(a), Value::U32(b)) => Value::U32(a * b),
            (Value::U64(a), Value::U64(b)) => Value::U64(a * b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a * b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a * b),
            //enum?
            _ => panic!("Cannot multiply"),
        }
    }
}

impl Div<Value> for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a / b),
            (Value::I64(a), Value::I64(b)) => Value::I64(a / b),
            (Value::U32(a), Value::U32(b)) => Value::U32(a / b),
            (Value::U64(a), Value::U64(b)) => Value::U64(a / b),
            (Value::F32(a), Value::F32(b)) => Value::F32(a / b),
            (Value::F64(a), Value::F64(b)) => Value::F64(a / b),
            //enum?
            _ => panic!("Cannot divide"),
        }
    }
}
