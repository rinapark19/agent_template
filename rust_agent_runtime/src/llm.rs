use pyo3::prelude::*;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

#[pyclass]
#[derive(Clone)]
pub struct ChatModel {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

#[pymethods]
impl ChatModel {
    #[new]
    pub fn new(model: String, base_url: String, api_key: String) -> Self {
        Self {
            model,
            base_url,
            api_key,
        }
    }

    pub fn invoke_messages(&self, messages_json: String) -> PyResult<String> {
        let messages: Value = serde_json::from_str(&messages_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let client = Client::new();

        let response = client
            .post(format!("{}/chat/completions", self.base_url.trim_end_matches("/")))
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "messages": messages
            }))
            .send()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let status = response.status();
        let value: Value = response
            .json()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        if !status.is_success() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "llm request failed: status={}, body={}",
                status,
                value
            )));
        }

        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("missing choices[0].message.content")
            })?
            .to_string();

        Ok(content)

    }
}

#[pyclass]
#[derive(Clone)]
pub struct PromptTemplate {
    pub system_template: String,
    pub user_template: Option<String>,
}

#[pymethods]
impl PromptTemplate {
    #[new]
    pub fn new(system_template: String, user_template: Option<String>) -> Self {
        Self {
            system_template,
            user_template,
        }
    }

    pub fn format_messages(&self, variables_json: String) -> PyResult<String> {
        let variables: HashMap<String, String> = serde_json::from_str(&variables_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let mut messages = vec![];

        messages.push(json!({
            "role": "system",
            "content": render_template(&self.system_template, &variables)
        }));

        if let Some(user_template) = &self.user_template {
            messages.push(json!({
                "role": "user",
                "content": render_template(user_template, &variables)
            })); 
        }

        serde_json::to_string(&messages)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    for (k, v) in variables {
        let placeholder = format!("{{{}}}", k);
        result = result.replace(&placeholder, v);
    }

    result
}

#[pyclass]
pub struct TextOutputParser;

#[pymethods]
impl TextOutputParser {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, text: String) -> PyResult<String> {
        Ok(text)
    }
}

#[pyclass]
pub struct JsonOutputParser;

#[pymethods]
impl JsonOutputParser {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, text: String) -> PyResult<String> {
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("json parse failed: {}", e)))?;

        serde_json::to_string(&value)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[pyclass]
pub struct LLMChain {
    pub model: ChatModel,
    pub prompt: PromptTemplate,
    pub parser_kind: String,
}

#[pymethods]
impl LLMChain {
    #[new]
    pub fn new(
        model: PyRef<'_, ChatModel>,
        prompt: PyRef<'_, PromptTemplate>,
        parser_kind: Option<String>
    ) -> Self {
        Self {
            model: model.clone(),
            prompt: prompt.clone(),
            parser_kind: parser_kind.unwrap_or_else(|| "text".to_string()),
        }
    }

    pub fn invoke(&self, variables_json: String) -> PyResult<String> {
        let messages_json = self.prompt.format_messages(variables_json)?;
        let raw = self.model.invoke_messages(messages_json)?;

        match self.parser_kind.as_str() {
            "json" => JsonOutputParser::new().parse(raw),
            _ => TextOutputParser::new().parse(raw),
        }
    }
}