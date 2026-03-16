mod grid;
mod mcp;
mod pty;
mod session;

use mcp::McpServer;

fn main() {
    // Initialize standard tracing for local debugging (if not talking over MCP stdin/stdout)
    // We disable this by default in MCP mode to avoid polluting stdout
    // tracing_subscriber::fmt::init();

    let server = McpServer::new();
    server.run();
}
