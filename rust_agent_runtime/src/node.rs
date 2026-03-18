use pyo3::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct Node {
    pub name: String,
    pub py_func: PyObject
}

impl Node {
    
    pub fn run(
        &self,
        py: Python<'_>,
        state_json: &str,
    ) -> PyResult<HashMap<String, Value>> {
        let result = self.py_func.call1(py, (state_json,))?;
        let result_str: String = result.extract(py)?;

        let update: HashMap<String, Value> = serde_json::from_str(&result_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(update)
    }
}