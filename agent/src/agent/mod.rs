mod react;
mod action;
// mod interpreter;
mod prompt;
// mod evolution;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::config::AgentConfig;

/// Agent with ReAct mode
pub struct Agent {
    /// Successful cases and functions
    pub knowledge_base: HashMap<String, String>,
    /// Code examples
    pub codes: Vec<Code>,
    /// All avaiable tools
    pub tools: HashMap<String, Tool>,
    /// Max times for ReAct
    pub max_step: usize,
}

/// The task context
pub struct TaskContext {
    /// the task info
    pub task: String,
    /// store the task memory (Thought, Action, Observation)
    pub memory: Vec<Message>,
    /// result history
    pub history: Vec<TaskResult>,
}

/// Task result
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub task: String,
    pub success: bool,
    pub output: String,
    /// for user feedback and evolution
    pub feedback: Option<String>,
}

impl Agent {
    /// create a new Agent
    pub fn new(config: AgentConfig) -> Result<Self> {
        Ok(Agent{
            knowledge_base: HashMap::new(),
            codes: vec![],
            tools: HashMap::new(),
            max_step: config.max_step,
        })
    }

    /// start a new task
    pub async fn run(&mut self, task: String) -> Result<String> {
        info!("Running task: {}", task);

        let mut context = TaskContext {
            task,
            memory: vec![],
            history: vec![],
        };

        // do ReAct loop for task
        let result = react::run_with_react(&self, &mut context).await?;

        // do self-evolution
        // evolution::reflect_and_upgrade(&mut self, context)?;

        Ok(result)
    }

    pub async fn action(&self, action: &action::Action) -> Result<String> {
        match action {
            Action::Code(code) => {
                // TODO
                Ok("code".to_owned())
            }
            Action::Tool(name, input) => {
                if let Some(tool) = self.tools.get(name) {
                    tool.call(input).await
                }
                // TODO
                Ok("success".to_owned())
            }
            Action::Unknown => {
                Ok("unknown".to_owned())
            }
        }
    }
}

/// Short-Term Message（Thought、Action、Observation）
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    /// "system", "thought", "action", "observation"
    pub role: String,
    /// content
    pub content: String,
}

/// Tool call
#[derive(Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub call: async fn(&str) -> Result<String>
}
