use std::collections::HashMap;

#[derive(Clone)]
pub struct State {
    pub data: HashMap<String, String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}