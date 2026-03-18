use std::collections::HashMap;
use serde_json::Value;
use crate::node::Node;
use crate::state::StateSchema;

pub struct Graph {

    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Vec<String>>,
    pub state_schema: StateSchema,

}

impl Graph {

    pub fn new() -> Self {

        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            state_schema: StateSchema::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.name.clone(),node);
    
    }

    pub fn add_edge(&mut self, from:String,to:String){

        self.edges
        .entry(from)
        .or_default()
        .push(to);
    }

    pub fn add_state_field(
        &mut self,
        name: String,
        default: Option<Value>,
        required: bool,
    ) {
        self.state_schema.add_field(name, default, required);
    }
}