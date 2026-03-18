use std::collections::HashMap;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct State {
    pub data: HashMap<String, Value>,
}

impl State {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn merge_update(&mut self, update: HashMap<String, Value>) {
        for (k, v) in update {
            self.data.insert(k, v);
        }
    }
}

#[derive(Clone, Debug)]
pub struct StateField {
    pub name: String,
    pub default: Option<Value>,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct StateSchema {
    pub fields: HashMap<String, StateField>,
}

impl StateSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: String, default: Option<Value>, required: bool) {
        self.fields.insert(
            name.clone(),
            StateField {
                name,
                default,
                required,
            },
        );
    }

    pub fn initialize_state(
        &self,
        input: HashMap<String, Value>,
    ) -> Result<State, String> {
        let mut merged = HashMap::new();

        for (field_name, field) in &self.fields {
            if let Some(value) = input.get(field_name) {
                merged.insert(field_name.clone(), value.clone());
            } else if let Some(default_value) = &field.default {
                merged.insert(field_name.clone(), default_value.clone());
            } else if field.required {
                return Err(format!("missing required state field: {}", field_name));
            }
        }

        for (k, v) in input {
            merged.entry(k).or_insert(v);
        }

        Ok(State::from_map(merged))
    }

    pub fn validate_update(&self, update: &HashMap<String, Value>) -> Result<(), String> {
        for key in update.keys() {
            if !self.fields.is_empty() && !self.fields.contains_key(key) {
                return Err(format!("unknown state field in update: {}", key));
            }
        }
        Ok(())
    }
}