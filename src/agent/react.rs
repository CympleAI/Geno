use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent::{Agent, Message, TaskResult, TaskContext};
use crate::agent::prompt::REACT_PROMPT;

/// Handle the task with ReAct
pub async fn run_with_react(agent: &Agent, context: &mut TaskContext) -> Result<String> {
    info!("Starting ReAct loop for task: {}", context.task);

    // init task context
    context.memory.push(Message {
        role: "system".to_string(),
        content: format!("Task: {}", context.task),
    });

    // ReAct
    for step in 0..agent.max_step {
        info!("ReAct step {}/{}", step + 1, agent.max_step);

        // Thought: generate the step
        let thought_prompt = build_thought_prompt(agent, context)?;

        let thought = call_llm(&thought_prompt)?;

        context.memory.push(Message {
            role: "thought".to_string(),
            content: thought.clone(),
        });
        debug!("Thought: {}", thought);

        // Action: parse and action
        let action = parse_action(&thought)?;

        let tool_output = agent.tool_call(&action.tool, &action.input).await?;

        context.memory.push(Message {
            role: "action".to_string(),
            content: format!("Tool: {}, Input: {}, Output: {}", action.tool, action.input, tool_output),
        });
        debug!("Action: tool={}, input={}, output={}", action.tool, action.input, tool_output);

        // Observation: collect the result
        context.memory.push(Message {
            role: "observation".to_string(),
            content: tool_output.clone(),
        });

        // check task is finished
        if is_task_complete(context, &tool_output)? {
            info!("Task completed successfully");
            context.history.push(TaskResult {
                task: context.task.clone(),
                success: true,
                output: tool_output.clone(),
                feedback: None,
            });
            return Ok(tool_output);
        }
    }

    // if task not finished in the limit times, record it
    warn!("Task failed to complete within {} steps", agent.max_step);
    context.history.push(TaskResult {
        task: context.task.clone(),
        success: false,
        output: "Failed to complete task".to_string(),
        feedback: Some(format!("Reached maximum steps: {}", agent.max_step)),
    });

    Err(anyhow!("Task failed to complete within {} steps", agent.max_step))
}


// Build Thought prompt
fn build_thought_prompt(agent: &Agent, context: &TaskContext) -> Result<String> {
    // collect history
    let messages = context.memory.iter()
        .map(|m| format!("[{}] {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    // aviable tools
    let tools = agent.tools.iter()
        .map(|t| format!("- {}: {}", t.name, t.description))
        .collect::<Vec<_>>()
        .join("\n");

    // build prompt
    let prompt = format!(
        "{}\n\nCurrent Task: {}\nAvailable Tools:\n{}\nMessages:\n{}\n\nGenerate the next reasoning step (Thought) and suggest an Action in JSON format:\n```json\n{{\"tool\": \"<tool_name>\", \"input\": \"<input_string>\"}}\n```",
        REACT_PROMPT, context.task, tools, messages
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

// check task is finished
fn is_task_complete(context: &mut TaskContext, output: &str) -> Result<bool> {
    // condition：
    // 1. output with "success" tag
    // 2. WASM compiled file
    // 3. custom user condition
    if output.contains("success") {
        info!("Completion condition met: output='{}'", output);
        return Ok(true);
    }

    // check if has failure tag
    if output.contains("error") || output.contains("failed") {
        context.history.push(TaskResult {
            task: context.task.clone(),
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
