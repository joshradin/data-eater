use std::rc::Weak;
use indexmap::IndexMap;

use serde::{Deserialize, Serialize};

use crate::snowflake::Snowflake;

/// A document is a single entity stored within data eater
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: Snowflake,
    pub fields: IndexMap<String, Document>
}

/// A value in a document
#[derive(Default, Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    #[default]
    Empty,
    Boolean(bool),
    Float(f64),
    Integer(i64),
    String(String),
    Blob(Box<[u8]>),
    List(Vec<Value>),
    Reference(DocumentRef)
}

/// A reference to a document
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DocumentRef(pub Snowflake);