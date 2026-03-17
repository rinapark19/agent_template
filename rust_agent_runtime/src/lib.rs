mod state;
mod node;
mod graph;
mod executor;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use crate::state::State;
use crate::graph::Graph;
use crate::node::Node;
use crate::executor::Executor;

#[pyclass]
struct PyGraph {
    graph: Graph,
}

#[pymethods]
impl PyGraph {
    #[new]
    fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    fn add_node(&mut self, name: String, func: PyObject) {
        let node = Node {
            name: name.clone(),
            py_func: func,
        };
        
        self.graph.add_node(node);
    }

    fn add_edge(&mut self, from: String, to: String) {
        self.graph.add_edge(from, to);
    }

    fn run(&self, py: Python<'_>, start: String, state_json: String) -> PyResult<String> {
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&state_json)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        let state = State { data: map };

        let executor = Executor::new(&self.graph);

        let result = executor.run(py, start, state);

        let result_json = serde_json::to_string(&result.data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(result_json)
    }
}

#[pymodule]
fn rust_agent_runtime(_py:Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    Ok(())
}