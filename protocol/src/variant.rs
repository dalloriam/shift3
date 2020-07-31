use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Variant {
    Int(i32),
    Str(String),
    Bool(bool),
    Map(HashMap<String, Variant>),
}
