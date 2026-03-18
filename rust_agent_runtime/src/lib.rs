mod state;
mod node;
mod graph;
mod executor;
mod llm;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::Value;

use crate::executor::Executor;
use crate::graph::Graph;
use crate::node::Node;

#[pyclass]
struct PyGraph {
    graph: Graph,
}

#[pymethods]
impl PyGraph {
    #[new]
    fn new() -> Self {
        Self {
            graph: Graph::new()
        }
    }

    fn add_state_field(
        &mut self,
        name: String,
        default_json: Option<String>,
        required: Option<bool>,
    ) -> PyResult<()> {
        let default_value: Option<Value> = match default_json {
            Some(s) => Some(
                serde_json::from_str(&s)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?
            ),
            None => None,
        };

        self.graph
            .add_state_field(name, default_value, required.unwrap_or(false));

        Ok(())
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
        let input: std::collections::HashMap<String, Value> = serde_json::from_str(&state_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let executor = Executor::new(&self.graph);
        let result_state = executor.run(py, start, input)?;

        serde_json::to_string(&result_state.data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[pymodule]
fn rust_agent_runtime(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    m.add_class::<crate::llm::ChatModel>()?;
    m.add_class::<crate::llm::PromptTemplate>()?;
    m.add_class::<crate::llm::TextOutputParser>()?;
    m.add_class::<crate::llm::JsonOutputParser>()?;
    m.add_class::<crate::llm::LLMChain>()?;
    Ok(())
}