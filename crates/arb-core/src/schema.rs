use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    Object,
    String,
    Number,
    Boolean,
    List,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchemaNode {
    #[serde(rename = "type")]
    pub node_type: SchemaType,

    #[serde(default)]
    pub required: Vec<String>,

    #[serde(default)]
    pub properties: HashMap<String, SchemaNode>,

    #[serde(default)]
    pub items: Option<Box<SchemaNode>>,

    #[serde(default)]
    pub description: Option<String>,
}
