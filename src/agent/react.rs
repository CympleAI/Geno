use anyhow::{Result, anyhow};
use log::{info, debug, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::agent::{Agent, Message, TaskResult};
use crate::prompts::react::REACT_PROMPT;

impl Agent {
    /// Handle the task with ReAct
    pub fn run_with_react(&mut self, input_content: Option<String>, output_path: &str) -> Result<String> {
        info!("Starting ReAct loop for task: {}", self.task);

        // init task context
        let initial_context = input_content.unwrap_or_default();
        self.messages.push(Message {
            role: "system".to_string(),
            content: format!("Task: {}\nContext: {}", self.task, initial_context),
        });

        // ReAct
        for iteration in 0..self.max_iterations {
            info!("ReAct iteration {}/{}", iteration + 1, self.max_iterations);

            // Thought: generate the step
            let thought_prompt = build_thought_prompt(&self)?;

            let thought = call_llm(&thought_prompt)?;

            self.messages.push(Message {
                role: "thought".to_string(),
                content: thought.clone(),
            });
            debug!("Thought: {}", thought);

            // Action: parse and action
            let action = parse_action(&thought)?;

            let tool_output = invoke_tool(&action.tool, &action.input, &self)?;

            self.messages.push(Message {
                role: "action".to_string(),
                content: format!("Tool: {}, Input: {}, Output: {}", action.tool, action.input, tool_output),
            });
            debug!("Action: tool={}, input={}, output={}", action.tool, action.input, tool_output);

            // Observation: collect the result
            self.messages.push(Message {
                role: "observation".to_string(),
                content: tool_output.clone(),
            });

            // check task is finished
            if is_task_complete(&self, &tool_output, output_path)? {
                info!("Task completed successfully");
                self.history.push(TaskResult {
                    task: self.task.clone(),
                    success: true,
                    output: tool_output.clone(),
                    feedback: None,
                });
                return Ok(tool_output);
            }
        }

        // if task not finished in the limit times, record it
        warn!("Task failed to complete within {} iterations", self.max_iterations);
        self.history.push(TaskResult {
            task: self.task.clone(),
            success: false,
            output: "Failed to complete task".to_string(),
            feedback: Some(format!("Reached maximum iterations: {}", self.max_iterations)),
        });

        Err(anyhow!("Task failed to complete within {} iterations", self.max_iterations))
    }
}

// Build Thought prompt
fn build_thought_prompt(state: &Agent) -> Result<String> {
    // collect history
    let messages = state.messages.iter()
        .map(|m| format!("[{}] {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    // aviable tools
    let tools = state.tools.iter()
        .map(|t| format!("- {}: {}", t.name, t.description))
        .collect::<Vec<_>>()
        .join("\n");

    // build prompt
    let prompt = format!(
        "{}\n\nCurrent Task: {}\nAvailable Tools:\n{}\nMessages:\n{}\n\nGenerate the next reasoning step (Thought) and suggest an Action in JSON format:\n```json\n{{\"tool\": \"<tool_name>\", \"input\": \"<input_string>\"}}\n```",
        REACT_PROMPT, state.task, tools, messages
    );
    Ok(prompt)
}

// parse Action（LLM response JSON）
#[derive(Debug, Deserialize, Serialize)]
struct Action {
    tool: String,
    input: String,
}

fn parse_action(thought: &str) -> Result<Action> {
    //  JSON
    let json_start = thought.find("```json").ok_or_else(|| anyhow!("No JSON block found in thought"))? + 7;
    let json_end = thought.rfind("```").ok_or_else(|| anyhow!("No closing JSON block found"))?;
    let json_str = &thought[json_start..json_end].trim();

    // parse JSON
    let action: Action = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Failed to parse action JSON: {}", e))?;

    // check tool
    debug!("Parsed action: tool={}, input={}", action.tool, action.input);
    Ok(action)
}

// call the tool
fn invoke_tool(tool: &str, input: &str, state: &mut AgentState) -> Result<String> {
    // find the tool
    let tool = state.tools.iter()
        .find(|t| t.name == tool)
        .ok_or_else(|| anyhow!("Tool '{}' not found", tool))?;

    // tool call
    (tool.invoke)(input, state)
}

// check task is finished
fn is_task_complete(state: &AgentState, output: &str, output_path: &str) -> Result<bool> {
    // condition：
    // 1. output with "success" tag
    // 2. WASM compiled file
    // 3. custom user condition
    if output.contains("success") || Path::new(output_path).exists() {
        info!("Completion condition met: output='{}', file_exists={}", output, output_path);
        return Ok(true);
    }

    // check if has failure tag
    if output.contains("error") || output.contains("failed") {
        state.history.push(TaskResult {
            task: state.task.clone(),
            success: false,
            output: output.to_string(),
            feedback: Some("Tool reported an error".to_string()),
        });
        return Ok(false);
    }

    Ok(false)
}

// mock LLM call
fn call_llm(prompt: &str) -> Result<String> {
    debug!("Simulated LLM call with prompt: {}", prompt);

    let action = if prompt.contains("Compile") {
        json!({
            "tool": "compile_lumora",
            "input": "(module Math (fn add (a int) (b int) -> int ((return (+ a b)))) (export add))"
        })
    } else if prompt.contains("Generate") {
        json!({
            "tool": "generate_code",
            "input": "Generate a Lumora factorial function"
        })
    } else {
        json!({
            "tool": "search_knowledge",
            "input": "Lumora best practices for performance"
        })
    };

    Ok(format!("Thought: Analyzing task...\n```json\n{}\n```", action))
}
