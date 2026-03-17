use pyo3::prelude::*;
use crate::state::State;

pub struct Node {
    pub name: String,
    pub py_func: PyObject
}

impl Node {
    
    pub fn run(&self, py: Python<'_>, state: &State) -> State {

        let state_json = serde_json::to_string(&state.data).unwrap();

        let result = self
            .py_func
            .call1(py,(state_json,))
            .unwrap();

        let result_str:String = result.extract(py).unwrap();

        let map:std::collections::HashMap<String,String> =
            serde_json::from_str(&result_str).unwrap();
        
        State{ data:map }
    }
}