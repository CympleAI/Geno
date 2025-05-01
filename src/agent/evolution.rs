use anyhow::{Result, anyhow};
use log::{info, debug, warn};
use serde_json::json;

use crate::agent::{AgentState, TaskResult};
use crate::prompts::upgrade::UPGRADE_PROMPT;

// 反思并升级 Agent
pub fn reflect_and_upgrade(state: &mut AgentState) -> Result<()> {
    info!("Starting self-upgrade for task: {}", state.task);

    // 检查任务历史
    if state.history.is_empty() {
        warn!("No task history available for reflection");
        return Ok(());
    }

    // 分析最近的任务结果
    let last_result = state.history.last().unwrap();
    debug!("Reflecting on task result: success={}, output={}", last_result.success, last_result.output);

    // 如果任务失败，生成改进策略
    if !last_result.success {
        let feedback = last_result.feedback.as_deref().unwrap_or("No specific feedback provided");
        let reflection_prompt = build_reflection_prompt(state, &last_result.output, feedback)?;
        let improvements = call_llm(&reflection_prompt)?;
        debug!("LLM suggested improvements: {}", improvements);

        // 应用改进（更新提示模板或知识库）
        apply_improvements(state, &improvements)?;
    }

    // 如果任务成功，更新知识库
    if last_result.success {
        update_knowledge_base(state, &last_result.task, &last_result.output)?;
    }

    info!("Self-upgrade completed");
    Ok(())
}

// 构建反思提示
fn build_reflection_prompt(state: &AgentState, output: &str, feedback: &str) -> Result<String> {
    let history = state.history.iter()
        .map(|r| format!("Task: {}, Success: {}, Output: {}", r.task, r.success, r.output))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "{}\n\nTask: {}\nLast Output: {}\nFeedback: {}\nHistory:\n{}\n\nAnalyze the failure and suggest improvements (e.g., better prompt, tool usage). Return JSON:\n```json\n{{\"prompt\": \"<new_prompt>\", \"tools\": [\"<tool_name>\"], \"strategy\": \"<strategy_description>\"}}\n```",
        UPGRADE_PROMPT, state.task, output, feedback, history
    );
    Ok(prompt)
}

// 应用改进
fn apply_improvements(state: &mut AgentState, improvements: &str) -> Result<()> {
    // 解析 LLM 返回的 JSON
    let json_start = improvements.find("```json").ok_or_else(|| anyhow!("No JSON block in improvements"))? + 7;
    let json_end = improvements.rfind("```").ok_or_else(|| anyhow!("No closing JSON block"))?;
    let json_str = &improvements[json_start..json_end].trim();

    let improvements: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Failed to parse improvements JSON: {}", e))?;

    // 更新提示模板
    if let Some(new_prompt) = improvements["prompt"].as_str() {
        state.prompt_template = new_prompt.to_string();
        info!("Updated prompt template: {}", new_prompt);
    }

    // 更新工具选择（暂存建议，实际工具更新需 tools.rs 实现）
    if let Some(tools) = improvements["tools"].as_array() {
        let tool_names = tools.iter()
            .filter_map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        debug!("Suggested tools: {}", tool_names);
        // TODO: 动态添加工具（需 tools.rs 支持）
    }

    // 记录策略
    if let Some(strategy) = improvements["strategy"].as_str() {
        state.knowledge_base.insert(
            format!("strategy:{}", state.task),
            strategy.to_string(),
        );
        info!("Stored strategy: {}", strategy);
    }

    Ok(())
}

// 更新知识库
fn update_knowledge_base(state: &mut AgentState, task: &str, output: &str) -> Result<()> {
    state.knowledge_base.insert(
        format!("success:{}", task),
        output.to_string(),
    );
    info!("Updated knowledge base with successful task: {}", task);
    Ok(())
}

// 模拟 LLM 调用（待 llm.rs 实现）
fn call_llm(prompt: &str) -> Result<String> {
    debug!("Simulated LLM call with prompt: {}", prompt);

    // 模拟改进建议
    let improvements = if prompt.contains("compile") {
        json!({
            "prompt": format!("{}\nFocus on generating valid Lumora code with proper types.", REACT_PROMPT),
            "tools": ["compile_lumora", "generate_code"],
            "strategy": "Ensure type annotations in Lumora code to avoid type errors."
        })
    } else {
        json!({
            "prompt": format!("{}\nPrioritize performance optimization.", REACT_PROMPT),
            "tools": ["run_wasm", "generate_code"],
            "strategy": "Optimize loops and reduce memory usage in generated code."
        })
    };

    Ok(format!("Reflection: Analyzing failure...\n```json\n{}\n```", improvements))
}
