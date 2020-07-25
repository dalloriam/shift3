use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;

use serde::{Deserialize, Serialize};

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum VariantError {
    InvalidType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Variant {
    Str(String),
    Int(i32),
    Bool(bool),
    List(Vec<Variant>),
    Map(HashMap<String, Variant>),
}

impl From<String> for Variant {
    fn from(v: String) -> Self {
        Variant::Str(v)
    }
}

impl From<bool> for Variant {
    fn from(b: bool) -> Self {
        Variant::Bool(b)
    }
}

impl From<i32> for Variant {
    fn from(b: i32) -> Self {
        Variant::Int(b)
    }
}

impl<T> From<Vec<T>> for Variant
where
    Variant: From<T>,
{
    fn from(v: Vec<T>) -> Self {
        Variant::List(v.into_iter().map(Variant::from).collect())
    }
}

impl Variant {
    pub fn cast<T>(self) -> T
    where
        Variant: From<T>,
    {
        unimplemented!();
    }
}

impl<T> From<HashMap<String, T>> for Variant
where
    Variant: From<T>,
{
    fn from(v: HashMap<String, T>) -> Variant {
        Variant::Map(HashMap::from_iter(
            v.into_iter().map(|(k, v)| (k, Variant::from(v))),
        ))
    }
}

impl TryFrom<Variant> for String {
    type Error = VariantError;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Str(s) => Ok(s),
            _ => Err(VariantError::InvalidType),
        }
    }
}

impl TryFrom<Variant> for i32 {
    type Error = VariantError;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Int(i) => Ok(i),
            _ => Err(VariantError::InvalidType),
        }
    }
}

impl TryFrom<Variant> for bool {
    type Error = VariantError;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Bool(i) => Ok(i),
            _ => Err(VariantError::InvalidType),
        }
    }
}
