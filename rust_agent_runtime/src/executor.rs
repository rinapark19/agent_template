use pyo3::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::graph::Graph;
use crate::state::State;

pub struct Executor<'a> {
    pub graph: &'a Graph,
}

impl<'a> Executor<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    pub fn run(
        &self,
        py: Python<'_>,
        start: String,
        input: HashMap<String, Value>,
    ) -> PyResult<State> {
        let mut current = start;

        let mut state = self
            .graph
            .state_schema
            .initialize_state(input)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
        
        loop {
            let node = self
                .graph
                .nodes
                .get(&current)
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err(format!("node not found: {}", current))
                })?;
            
            let state_json = serde_json::to_string(&state.data)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                
            let update = node.run(py, &state_json)?;

            self.graph
                .state_schema
                .validate_update(&update)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;

            state.merge_update(update);

            match self.graph.edges.get(&current) {
                Some(next) if !next.is_empty() => {
                    current = next[0].clone();
                }
                _ => break,
            }
        }

        Ok(state)
    }
}