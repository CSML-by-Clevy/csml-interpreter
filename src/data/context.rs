use crate::data::{
    primitive::{PrimitiveObject, PrimitiveType},
    Client, Hold, Interval, Literal,
};

use crate::interpreter::{json_to_literal, memory_to_literal};

use nom::lib::std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiInfo {
    pub client: Client,
    pub fn_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct ContextJson {
    pub current: serde_json::Value,
    pub metadata: serde_json::Value,
    pub api_info: Option<ApiInfo>,
    pub hold: Option<Hold>,
}

impl Default for ContextJson {
    fn default() -> Self {
        Self {
            current: serde_json::json!({}),
            metadata: serde_json::json!({}),
            api_info: None,
            hold: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub current: HashMap<String, Literal>,
    pub metadata: HashMap<String, Literal>,
    pub api_info: Option<ApiInfo>,
    pub hold: Option<Hold>,
}

pub fn get_hashmap_from_mem(lit: &serde_json::Value) -> HashMap<String, Literal> {
    match memory_to_literal(lit, Interval { line: 0, column: 0 }) {
        Ok(vars) if vars.primitive.get_type() == PrimitiveType::PrimitiveObject => {
            match vars.primitive.as_any().downcast_ref::<PrimitiveObject>() {
                Some(map) => map.value.clone(),
                None => HashMap::new(),
            }
        }
        _ => HashMap::new(),
    }
}

pub fn get_hashmap_from_json(lit: &serde_json::Value) -> HashMap<String, Literal> {
    match json_to_literal(lit, Interval { line: 0, column: 0 }) {
        Ok(vars) if vars.primitive.get_type() == PrimitiveType::PrimitiveObject => {
            match vars.primitive.as_any().downcast_ref::<PrimitiveObject>() {
                Some(map) => map.value.clone(),
                None => HashMap::new(),
            }
        }
        _ => HashMap::new(),
    }
}

impl ContextJson {
    pub fn to_literal(&self) -> Context {
        let current = get_hashmap_from_mem(&self.current);
        let metadata = get_hashmap_from_json(&self.metadata);

        Context {
            current,
            metadata,
            api_info: self.api_info.to_owned(),
            hold: self.hold.to_owned(),
        }
    }
}
