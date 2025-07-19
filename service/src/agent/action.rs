use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result};

pub enum Action {
    Code(String),
    Tool(String, String),
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonAction {
    tool: Option<String>,
    input: Option<String>,
    code: Option<String>
}

impl Action {
    pub fn from_str(s: &str) -> Result<Self> {
        let json: JsonAction = serde_json::from_str(s)
            .map_err(|e| anyhow!("Failed to parse action JSON: {}", e))?;

        if let Some(code) = json.code {
            if !code.is_empty() {
                return Ok(Action::Code(code));
            }
        }

        match (json.tool, json.input) {
            (Some(tool), Some(input)) => {
                Ok(Action::Tool(tool, input))
            }
            _ => Ok(Action::Unknown),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Action::Code(code) => format!("Code: {code}"),
            Action::Tool(tool, input) => format!("Tool: {tool}, Input: {input}"),
            Action::Unknown => "Unknown action".to_owned(),
        }
    }
}
