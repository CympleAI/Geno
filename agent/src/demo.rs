use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tokio::{process::Command as AsyncCommand, fs};

// Agent configuration and state
struct Agent {
    client: Client,
    api_key: String,
    model: String,
    tools: HashMap<String, Box<dyn Tool>>,
    history: Vec<Message>,
    temp_dir: String,
}

// Message structures for conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCall {
    id: String,
    r#type: String, // "function" or "code_interpreter"
    function: Option<FunctionCall>,
    code_interpreter: Option<CodeInterpreter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeInterpreter {
    input: String,
    language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolResult {
    tool_call_id: String,
    output: String,
}

// Tool trait and implementations
#[async_trait]
trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Value) -> Result<String>;
}

impl Agent {
    async fn new(api_key: &str, model: &str) -> Result<Self> {
        let temp_dir = "./tmp".to_owned();
        fs::create_dir_all(&temp_dir).await?;

        let mut agent = Agent {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            tools: HashMap::new(),
            history: Vec::new(),
            temp_dir,
        };

        // Register default tools
        agent.register_tool(Box::new(WebSearchTool));
        agent.register_tool(Box::new(FileSystemTool));

        Ok(agent)
    }

    fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }

    async fn chat(&mut self, user_input: &str) -> Result<String> {
        // Add user message to history
        self.add_message(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
            tool_calls: None,
            name: None,
        });

        // Prepare messages for API call
        let messages = self.history.clone();

        // Define the tools available to the model
        let tools = vec![
            json!({
                "type": "code_interpreter",
                "code_interpreter": {
                    "languages": ["python", "javascript", "rust", "bash", "ruby"]
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "web_search",
                    "description": "Search the web for information",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            }
                        },
                        "required": ["query"]
                    }
                }
            }),
            // Add more tools as needed
        ];

        // Request to AI model
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": messages,
                "tools": tools,
                "tool_choice": "auto"
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        // Extract the assistant's message
        let assistant_message = response["choices"][0]["message"].clone();
        let content = assistant_message["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Check if there are any tool calls
        let mut tool_calls = None;
        if assistant_message["tool_calls"].is_array() {
            tool_calls = serde_json::from_value(assistant_message["tool_calls"].clone()).ok();
        }

        // Create and add the assistant message to history
        let mut message = Message {
            role: "assistant".to_string(),
            content,
            tool_calls: tool_calls.clone(),
            name: None,
        };
        self.add_message(message.clone());

        // Process tool calls if any
        if let Some(tool_calls) = tool_calls {
            for tool_call in tool_calls {
                let result = self.handle_tool_call(&tool_call).await?;

                // Add tool result to history
                self.add_message(Message {
                    role: "tool".to_string(),
                    content: result.output,
                    tool_calls: None,
                    name: Some(tool_call.id),
                });
            }

            // Get follow-up response after tool calls
            return Box::pin(self.chat("Please continue based on the tool results.")).await;
        }

        Ok(message.content)
    }

    async fn handle_tool_call(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        match tool_call.r#type.as_str() {
            "code_interpreter" => {
                if let Some(code_interpreter) = &tool_call.code_interpreter {
                    self.execute_code(&code_interpreter.input, &code_interpreter.language)
                        .await
                        .map(|output| ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            output,
                        })
                } else {
                    Err(anyhow!("Code interpreter data missing"))
                }
            }
            "function" => {
                if let Some(function_call) = &tool_call.function {
                    let tool_name = &function_call.name;
                    if let Some(tool) = self.tools.get(tool_name) {
                        let args: Value = serde_json::from_str(&function_call.arguments)?;
                        tool.execute(args).await.map(|output| ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            output,
                        })
                    } else {
                        Err(anyhow!("Tool not found: {}", tool_name))
                    }
                } else {
                    Err(anyhow!("Function call data missing"))
                }
            }
            _ => Err(anyhow!("Unknown tool call type: {}", tool_call.r#type)),
        }
    }

    async fn execute_code(&self, code: &str, language: &str) -> Result<String> {
        // Create a temporary file for the code
        let file_name = format!(
            "{}/code_{}.{}",
            self.temp_dir,
            "TODO",
            match language {
                "python" => "py",
                "javascript" => "js",
                "rust" => "rs",
                "ruby" => "rb",
                _ => language,
            }
        );

        fs::write(&file_name, code).await?;

        // Execute the code based on language
        let output = match language {
            "python" => AsyncCommand::new("python").arg(&file_name).output().await?,
            "javascript" => AsyncCommand::new("node").arg(&file_name).output().await?,
            "rust" => {
                // For Rust, compile first then run
                let compiled_file = format!("{}/code_exe", self.temp_dir);
                let compile_output = AsyncCommand::new("rustc")
                    .arg(&file_name)
                    .arg("-o")
                    .arg(&compiled_file)
                    .output()
                    .await?;

                if !compile_output.status.success() {
                    return Ok(format!(
                        "Compilation error: {}",
                        String::from_utf8_lossy(&compile_output.stderr)
                    ));
                }

                AsyncCommand::new(&compiled_file).output().await?
            }
            "bash" => AsyncCommand::new("bash").arg(&file_name).output().await?,
            "ruby" => AsyncCommand::new("ruby").arg(&file_name).output().await?,
            _ => return Err(anyhow!("Unsupported language: {}", language)),
        };

        // Combine stdout and stderr for the result
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            Ok(format!("Error: {}\nOutput: {}", stderr, stdout))
        }
    }
}

// Tool implementations
struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Query parameter missing"))?;

        // Implement web search functionality
        // This is a placeholder - you'd typically use a search API
        Ok(format!("Search results for: {}", query))
    }
}

struct FileSystemTool;

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "file_system"
    }

    fn description(&self) -> &str {
        "Read and write files on the system"
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| anyhow!("Operation parameter missing"))?;

        match operation {
            "read" => {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Path parameter missing"))?;
                let content = fs::read_to_string(path).await?;
                Ok(content)
            }
            "write" => {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Path parameter missing"))?;
                let content = args["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Content parameter missing"))?;
                fs::write(path, content).await?;
                Ok(format!("File written successfully: {}", path))
            }
            "list" => {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Path parameter missing"))?;
                let mut entries = Vec::new();
                let mut dir = fs::read_dir(path).await?;
                while let Some(entry) = dir.next_entry().await? {
                    entries.push(entry.file_name().to_string_lossy().to_string());
                }
                Ok(serde_json::to_string(&entries)?)
            }
            _ => Err(anyhow!("Unsupported operation: {}", operation)),
        }
    }
}

// Main function to run the agent
#[tokio::main]
async fn main() -> Result<()> {
    // Read API key from environment or config
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let model = "gpt-4-turbo"; // or your preferred model that supports tool calls

    let mut agent = Agent::new(&api_key, model).await?;

    println!("AI Agent initialized. Enter your query (or 'exit' to quit):");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.to_lowercase() == "exit" {
            break;
        }

        match agent.chat(input).await {
            Ok(response) => println!("Agent: {}", response),
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
