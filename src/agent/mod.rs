mod react;
mod action;
mod interpreter;
mod prompt;
mod evolution;

pub mod tools;
pub mod llm;

use anyhow::{Result, anyhow};
use log::{info, debug};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::prompts::react::REACT_PROMPT;

/// Agent with ReAct mode
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    /// store Thought, Action, Observation (memeory)
    pub messages: Vec<Message>,
    /// Successful cases and functions
    pub knowledge_base: HashMap<String, String>,
    /// Template for ReAct
    pub prompt_template: String,
    /// All tools
    pub tools: Vec<Tool>,
    /// Current task info
    pub task: String,
    /// History task result
    pub history: Vec<TaskResult>,
    /// Max times for ReAct
    pub max_iterations: usize,
}

impl Agent {
    /// create a new Agent
    pub fn new(task: &str, max_iterations: usize) -> Result<Self> {
        info!("Initializing Agent for task: {}", task);

        Ok(Agent{
            messages: vec![],
            knowledge_base: HashMap::new(),
            prompt_template: REACT_PROMPT.to_string(),
            tools: tools::init_tools(),
            task: task.to_string(),
            history: vec![],
            max_iterations,
        })
    }

    /// start a new task
    pub fn run(&mut self, input_content: Option<String>, output_path: &str) -> Result<String> {
        info!("Running task: {}", self.state.task);

        // do ReAct loop
        let result = react::run_with_react(&mut self, input_content, output_path)?;

        // do evolution
        evolution::reflect_and_upgrade(&mut self.state)?;

        Ok(result)
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
    pub invoke: fn(&str, &mut AgentState) -> Result<String>, // 工具调用函数
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
