# async-shell

A headless terminal multiplexer and Model Context Protocol (MCP) server designed specifically for AI agents.

## The Problem
When an AI agent (like Claude, Devin, or Gemini) runs a long-lived command like `npm run dev` or a compiler watcher using standard shell tools, the agent gets stuck waiting for `stdout` to close. The entire reasoning loop hangs. 

## The Solution
`async-shell` acts as a "Headless Tmux for AI". It allows agents to spawn commands in native OS pseudo-terminals (PTYs) in the background. The agent can then continue reasoning, writing code, and periodically poll the background processes for their scrollback logs without ever blocking.

## Features
- **True Non-Blocking Execution**: Spawn any shell command asynchronously.
- **Agent Isolation**: Uses Ed25519 cryptography to assign unique session IDs to different agents, ensuring their read cursors never overlap.
- **Smart Paging**: Agents can page through up to 100,000 lines of scrollback history to find stack traces without blowing out their LLM context window.
- **Interactive Stdin**: Agents can send keystrokes (like `Ctrl+C` or "yes\n") to running background processes.

## Usage (MCP)

Configure your agent harness (Claude Desktop, Cursor, etc.) to use `async-shell` as an MCP server.

```json
{
  "mcpServers": {
    "async-shell": {
      "command": "npx",
      "args": ["-y", "async-shell"]
    }
  }
}
```

### Protocol Flow
1. Agent calls `register_agent` to receive a cryptographic `agent_id`.
2. Agent calls `spawn` with a `session_id` and the `command` to run.
3. Agent calls `read_history` using their `agent_id` to read the logs. If `start_line` is omitted, the server automatically resumes reading exactly where the agent last left off.
4. Agent calls `write_stdin` to interact with the process.

## License
MIT
