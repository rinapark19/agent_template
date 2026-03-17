use std::collections::HashMap;
use crate::node::Node;

pub struct Graph {

    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Vec<String>>,

}

impl Graph {

    pub fn new() -> Self {

        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
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
}