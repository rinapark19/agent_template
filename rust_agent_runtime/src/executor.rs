use pyo3::prelude::*;
use crate::graph::Graph;
use crate::state::State;

pub struct Executor<'a> {
    pub graph: &'a Graph,
}

impl<'a> Executor<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    pub fn run(&self, py:Python<'_>, start: String, state: State) -> State {
        let mut current = start;
        let mut state = state;

        loop {
            let node = self
                .graph
                .nodes
                .get(&current)
                .expect("node not found");

            state = node.run(py, &state);

            match self.graph.edges.get(&current) {
                Some(next) if !next.is_empty() => {
                    current = next[0].clone();
                }
                _ => break,
            }
        }

        state
    }
}