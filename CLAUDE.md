# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Geno is an open and free future personal assistant system built in Rust. Users can deploy their self-hosted service and connect third-party services (Gmail, Telegram, Slack, etc.) to enable intelligent message handling, event creation, and privacy-focused automation.

The project is structured as a Cargo workspace with multiple components:

- `app/`: Web frontend, mobile and desktop applications
- `agent/`: Core AI agent system with ReAct (Reason+Act) capabilities
- `proxy/`: HTTP gateway server coordinating database, frontend, agent, and third-party AI services
- `integrations/`: Third-party service connectors (Gmail, Telegram, Slack, calendar, etc.)
- `types/`: Shared type definitions and data structures

## Common Commands

### Building the Project
```bash
cargo build
```

### Running Components
```bash
# Run the agent service
cargo run --bin agent -- --ask "your question here"

# Run the proxy gateway
cargo run --bin proxy
```

### Testing
```bash
cargo test
```

### Checking Code Quality
```bash
cargo check
cargo clippy
cargo fmt
```

## Architecture Overview

### System Design

**Data Flow:**
1. Third-party services → `integrations/` → `proxy/` → `agent/`
2. Agent processes messages, extracts actions (calendar events, bill storage, etc.)
3. Privacy-focused LLM communication ensures no personal data leakage
4. Results flow back through proxy to frontend applications

### Core Components

**Agent System (`agent/src/`)**
- `Agent`: Main agent struct with knowledge base, tools, and ReAct loop functionality
- `react.rs`: Implements the ReAct (Reasoning and Acting) pattern with Thought-Action-Observation cycles
- `action.rs`: Defines action types (Code, Tool, Unknown) that the agent can perform
- `prompt.rs`: Contains prompts for the ReAct system

**Proxy Gateway (`proxy/src/`)**
- HTTP server coordinating all system components
- Database connections and data persistence
- Third-party AI service integration (STT, TTS, LLM)
- Frontend API endpoints

**Integrations (`integrations/`)**
- Service-specific connectors for Gmail, Telegram, Slack, calendars
- Standardized message format conversion
- Authentication and API management

**Applications (`app/`)**
- Web frontend for system management
- Mobile and desktop applications
- User interface for agent interactions

**Key Structures**
- `TaskContext`: Manages task state, memory, and history
- `Message`: Represents communication in the system (system, thought, action, observation roles)
- `Tool`: Defines available tools with async function calls
- `TaskResult`: Stores task execution results with success status and feedback

### Execution Flow

1. Third-party services send events/messages to integrations
2. Proxy receives standardized messages and routes to agent
3. Agent processes with ReAct loop:
   - **Thought**: Generate reasoning step using privacy-filtered prompts
   - **Action**: Execute actions (create calendar events, store bills, etc.)
   - **Observation**: Collect results and check for completion
4. Results stored in database and/or forwarded to relevant services

### Current Implementation Status

- **Implemented**: Basic agent ReAct system, workspace structure
- **In Progress**: Proxy gateway placeholder, type definitions
- **Planned**: Integration connectors, web/mobile apps, database layer
- **Privacy Features**: LLM prompt sanitization system (planned)

## Development Notes

- Uses `tracing` for structured logging with DEBUG level
- Async/await pattern throughout with `tokio` runtime
- Configuration managed through `AgentConfig` struct
- Task completion detected by "success" keyword in output
- Error handling uses `anyhow` for error propagation
