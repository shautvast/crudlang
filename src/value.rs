use anyhow::anyhow;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructValue {
    definition: StructDefinition,
    fields: Vec<Value>,
}

impl StructValue {
    pub fn new(definition: StructDefinition) -> Self {
        Self {
            definition,
            fields: Vec::new(),
        }
    }
}

impl Display for StructValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, field) in self.definition.fields.iter().enumerate() {
            write!(f, "{}: {}", field, self.fields[i])?;
        }
        Ok(())
    }
}

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
    Date(DateTime<Utc>),
    Enum,
    List(Vec<Value>),
    Map(HashMap<Value, Value>),
    Struct(StructValue),
    Error(String),
    Void,
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

impl Into<Value> for DateTime<Utc> {
    fn into(self) -> Value {
        Value::Date(self)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            &Value::U32(v) => write!(f, "{}", v),
            &Value::U64(v) => write!(f, "{}", v),
            &Value::String(v) => write!(f, "{}", v),
            &Value::Bool(v) => write!(f, "{}", v),
            &Value::I32(v) => write!(f, "{}", v),
            &Value::I64(v) => write!(f, "{}", v),
            &Value::F32(v) => write!(f, "{}", v),
            &Value::F64(v) => write!(f, "{}", v),
            &Value::Char(v) => write!(f, "{}", v),
            &Value::Date(v) => write!(f, "{}", v),
            &Value::Enum => write!(f, "enum"),
            &Value::Struct(v) => write!(f, "{}", v),
            &Value::List(v) => write!(f, "{:?}", v),
            &Value::Map(v) => write!(f, "map"),
            &Value::Error(v) => write!(f, "{}", v),
            &Value::Void => write!(f, "()"),
        }
    }
}

impl Neg for &Value {
    type Output = anyhow::Result<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::I32(i) => Ok(Value::I32(-i)),
            Value::I64(i) => Ok(Value::I64(-i)),
            Value::F32(i) => Ok(Value::F32(-i)),
            Value::F64(i) => Ok(Value::F64(-i)),
            _ => Err(anyhow!("Cannot negate")),
        }
    }
}

impl Add<&Value> for &Value {
    type Output = anyhow::Result<Value>;

    fn add(self, rhs: &Value) -> Self::Output {
        if let Value::List(s) = self {
            let mut copy = s.clone();
            copy.push(rhs.clone());
            Ok(Value::List(copy))
        } else if let Value::List(rhs) = rhs {
            let mut copy = rhs.clone();
            copy.push(self.clone());
            Ok(Value::List(copy))
        } else {
            match (self, rhs) {
                (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b)),
                (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a + b)),
                (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a + b)),
                (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a + b)),
                (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
                (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
                (Value::String(s), Value::I32(i)) => Ok(Value::String(format!("{}{}", s, i))),
                (Value::String(s), Value::I64(i)) => Ok(Value::String(format!("{}{}", s, i))),
                (Value::String(s), Value::U32(u)) => Ok(Value::String(format!("{}{}", s, u))),
                (Value::String(s), Value::U64(u)) => Ok(Value::String(format!("{}{}", s, u))),
                (Value::String(s), Value::F32(f)) => Ok(Value::String(format!("{}{}", s, f))),
                (Value::String(s), Value::F64(f)) => Ok(Value::String(format!("{}{}", s, f))),
                (Value::String(s), Value::Bool(b)) => Ok(Value::String(format!("{}{}", s, b))),
                (Value::String(s), Value::Char(c)) => Ok(Value::String(format!("{}{}", s, c))),
                (Value::String(s1), Value::String(s2)) => {
                    Ok(Value::String(format!("{}{}", s1, s2)))
                }
                (Value::String(s1), Value::Map(m)) => Ok(Value::String(format!("{}{:?}", s1, m))),
                //enum?
                _ => Err(anyhow!("Cannot add")),
            }
        }
    }
}

impl Sub<&Value> for &Value {
    type Output = anyhow::Result<Value>;

    fn sub(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a - b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a - b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a - b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a - b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a - b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            //enum?
            _ => Err(anyhow!("Cannot subtract")),
        }
    }
}

impl Mul<&Value> for &Value {
    type Output = anyhow::Result<Value>;

    fn mul(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a * b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a * b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a * b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a * b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a * b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            _ => Err(anyhow!("Cannot multiply")),
        }
    }
}

impl Div<&Value> for &Value {
    type Output = anyhow::Result<Value>;

    fn div(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a / b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a / b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a / b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a / b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a / b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),
            _ => Err(anyhow!("Cannot divide")),
        }
    }
}

impl BitAnd<&Value> for &Value {
    type Output = anyhow::Result<Value>;
    fn bitand(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a & b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a & b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a & b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a & b)),
            _ => Err(anyhow!("Cannot do bitwise-and on")),
        }
    }
}

impl BitOr<&Value> for &Value {
    type Output = anyhow::Result<Value>;
    fn bitor(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a | b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a | b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a | b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a | b)),
            _ => Err(anyhow!("Cannot do bitwise-or on")),
        }
    }
}

impl BitXor<&Value> for &Value {
    type Output = anyhow::Result<Value>;
    fn bitxor(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a ^ b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a ^ b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a ^ b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a ^ b)),
            _ => Err(anyhow!("Cannot do bitwise-xor on")),
        }
    }
}

impl Not for &Value {
    type Output = anyhow::Result<Value>;

    fn not(self) -> Self::Output {
        match (self) {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::I32(i32) => Ok(Value::I32(!i32)),
            Value::I64(i64) => Ok(Value::I64(!i64)),
            Value::U32(u32) => Ok(Value::U32(!u32)),
            Value::U64(u64) => Ok(Value::U64(!u64)),
            _ => Err(anyhow!("Cannot calculate not")),
        }
    }
}

impl Shl<&Value> for &Value {
    type Output = anyhow::Result<Value>;
    fn shl(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a << b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a << b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a << b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a << b)),
            _ => Err(anyhow!("Cannot shift left on")),
        }
    }
}

impl Shr<&Value> for &Value {
    type Output = anyhow::Result<Value>;
    fn shr(self, rhs: &Value) -> Self::Output {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a >> b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a >> b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a >> b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a >> b)),
            _ => Err(anyhow!("Cannot shift right on")),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::F32(a), Value::F32(b)) => a == b,
            (Value::F64(a), Value::F64(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            // (Value::List(a), Value::List(b)) => a == b,
            // (Value::Map(a), Value::Map(b)) => {
            //     let mut equal = true;
            //     for (k, v) in a.iter() {
            //         if !b.contains_key(k) || b.get(k).unwrap() != v {
            //             //safe unwrap
            //             equal = false;
            //             break;
            //         }
            //     }
            //     equal
            // }
            // struct?
            _ => false, //?
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match (self, rhs) {
            (Value::I32(a), Value::I32(b)) => Some(a.partial_cmp(b)?),
            (Value::I64(a), Value::I64(b)) => Some(a.partial_cmp(b)?),
            (Value::U32(a), Value::U32(b)) => Some(a.partial_cmp(b)?),
            (Value::U64(a), Value::U64(b)) => Some(a.partial_cmp(b)?),
            (Value::F32(a), Value::F32(b)) => Some(a.partial_cmp(b)?),
            (Value::F64(a), Value::F64(b)) => Some(a.partial_cmp(b)?),
            (Value::String(a), Value::String(b)) => Some(a.partial_cmp(b)?),
            (Value::Char(a), Value::Char(b)) => Some(a.partial_cmp(b)?),
            (Value::Date(a), Value::Date(b)) => Some(a.partial_cmp(b)?),
            _ => None,
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        // Then hash the fields
        match self {
            Value::I32(i32) => i32.hash(state),
            Value::I64(i64) => i64.hash(state),
            Value::U32(u32) => u32.hash(state),
            Value::U64(u64) => u64.hash(state),
            Value::F32(f32) => f32.to_bits().hash(state),
            Value::F64(f64) => f64.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Char(c) => c.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Date(d) => d.hash(state),
            Value::List(l) => l.hash(state),
            _ => {}
        }
    }
}

// impl Ord for Value {
//     fn cmp(&self, rhs: &Self) -> Ordering {
//         self.partial_cmp(rhs).unwrap()
//     }
// }
