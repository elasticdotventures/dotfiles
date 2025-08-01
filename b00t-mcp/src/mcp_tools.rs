use clap::Parser;
use crate::clap_reflection::{McpReflection, McpCommandRegistry};
use crate::impl_mcp_tool;

// Re-export b00t-cli command structures for MCP use
// This creates a compile-time dependency but ensures type safety

/// MCP command for listing MCP servers
#[derive(Parser, Clone)]
pub struct McpListCommand {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

impl_mcp_tool!(McpListCommand, "b00t_mcp_list", ["mcp", "list"]);

/// MCP command for adding MCP servers
#[derive(Parser, Clone)]
pub struct McpAddCommand {
    #[arg(help = "MCP server configuration JSON or '-' for stdin")]
    pub json: String,
    
    #[arg(long, help = "Enable Do What I Want mode for enhanced parsing")]
    pub dwiw: bool,
    
    #[arg(long, help = "Server hint/description")]
    pub hint: Option<String>,
}

impl_mcp_tool!(McpAddCommand, "b00t_mcp_add", ["mcp", "add"]);

/// MCP command for MCP server output generation
#[derive(Parser, Clone)]
pub struct McpOutputCommand {
    #[arg(help = "Comma-separated list of server names")]
    pub servers: String,
    
    #[arg(long, help = "Output raw JSON instead of mcpServers wrapper")]
    pub json: bool,
}

impl_mcp_tool!(McpOutputCommand, "b00t_mcp_output", ["mcp", "output"]);

/// CLI detect command
#[derive(Parser, Clone)]
pub struct CliDetectCommand {
    #[arg(help = "Command to detect version for")]
    pub command: String,
}

impl_mcp_tool!(CliDetectCommand, "b00t_cli_detect", ["cli", "detect"]);

/// CLI desires command  
#[derive(Parser, Clone)]
pub struct CliDesiresCommand {
    #[arg(help = "Command to show desired version for")]
    pub command: String,
}

impl_mcp_tool!(CliDesiresCommand, "b00t_cli_desires", ["cli", "desires"]);

/// CLI check command
#[derive(Parser, Clone)]
pub struct CliCheckCommand {
    #[arg(help = "Command to check version alignment for")]
    pub command: String,
}

impl_mcp_tool!(CliCheckCommand, "b00t_cli_check", ["cli", "check"]);

/// CLI install command
#[derive(Parser, Clone)]
pub struct CliInstallCommand {
    #[arg(help = "Command to install")]
    pub command: String,
}

impl_mcp_tool!(CliInstallCommand, "b00t_cli_install", ["cli", "install"]);

/// CLI update command
#[derive(Parser, Clone)]
pub struct CliUpdateCommand {
    #[arg(help = "Command to update")]
    pub command: String,
}

impl_mcp_tool!(CliUpdateCommand, "b00t_cli_update", ["cli", "update"]);

/// CLI up command (update all)
#[derive(Parser, Clone)]  
pub struct CliUpCommand {
    #[arg(long, help = "Dry run - show what would be updated")]
    pub dry_run: bool,
}

impl_mcp_tool!(CliUpCommand, "b00t_cli_up", ["cli", "up"]);

/// Whoami command
#[derive(Parser, Clone)]
pub struct WhoamiCommand;

impl_mcp_tool!(WhoamiCommand, "b00t_whoami", ["whoami"]);

/// Status command
// 🤓 ENTANGLED: b00t-cli/src/main.rs Commands::Status
// When this changes, update b00t-cli Status command structure
#[derive(Parser, Clone)]
pub struct StatusCommand {
    #[arg(long, help = "Filter by subsystem")]
    pub filter: Option<String>,
    
    #[arg(long, help = "Show only installed tools")]
    pub installed: bool,
    
    #[arg(long, help = "Show only available tools")]
    pub available: bool,
}

impl_mcp_tool!(StatusCommand, "b00t_status", ["status"]);

/// AI list command
#[derive(Parser, Clone)]
pub struct AiListCommand {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
}

impl_mcp_tool!(AiListCommand, "b00t_ai_list", ["ai", "list"]);

/// AI output command
#[derive(Parser, Clone)]
pub struct AiOutputCommand {
    #[arg(help = "Comma-separated list of AI provider names")]
    pub providers: String,
    
    #[arg(long, help = "Output key-value pairs")]
    pub kv: bool,
    
    #[arg(long, help = "Output b00t format")]
    pub b00t: bool,
}

impl_mcp_tool!(AiOutputCommand, "b00t_ai_output", ["ai", "output"]);

/// App VSCode MCP install command
#[derive(Parser, Clone)]
pub struct AppVscodeMcpInstallCommand {
    #[arg(help = "MCP server name to install")]
    pub name: String,
}

impl_mcp_tool!(AppVscodeMcpInstallCommand, "b00t_app_vscode_mcp_install", ["app", "vscode", "mcp", "install"]);

/// App Claude Code MCP install command
#[derive(Parser, Clone)]
pub struct AppClaudecodeMcpInstallCommand {
    #[arg(help = "MCP server name to install")]
    pub name: String,
}

impl_mcp_tool!(AppClaudecodeMcpInstallCommand, "b00t_app_claudecode_mcp_install", ["app", "claudecode", "mcp", "install"]);

/// Session init command
// 🤓 ENTANGLED: b00t-cli/src/commands/session.rs SessionCommands::Init
// When this changes, update b00t-cli SessionCommands::Init structure
#[derive(Parser, Clone)]
pub struct SessionInitCommand {
    #[arg(long, help = "Budget limit in dollars")]
    pub budget: Option<f64>,
    
    #[arg(long, help = "Time limit in minutes")]
    pub time_limit: Option<u32>,
    
    #[arg(long, help = "Agent name")]
    pub agent: Option<String>,
}

impl_mcp_tool!(SessionInitCommand, "b00t_session_init", ["session", "init"]);

/// Session status command
#[derive(Parser, Clone)]
pub struct SessionStatusCommand;

impl_mcp_tool!(SessionStatusCommand, "b00t_session_status", ["session", "status"]);

/// Session end command
#[derive(Parser, Clone)]
pub struct SessionEndCommand;

impl_mcp_tool!(SessionEndCommand, "b00t_session_end", ["session", "end"]);

/// Learn command
// 🤓 ENTANGLED: b00t-cli/src/main.rs Commands::Learn
// When this changes, update b00t-cli Learn command structure
#[derive(Parser, Clone)]
pub struct LearnCommand {
    #[arg(help = "Topic to learn about")]
    pub topic: Option<String>,
}

impl_mcp_tool!(LearnCommand, "b00t_learn", ["learn"]);

/// Checkpoint command
// 🤓 ENTANGLED: b00t-cli/src/main.rs Commands::Checkpoint
// When this changes, update b00t-cli Checkpoint command structure
#[derive(Parser, Clone)]
pub struct CheckpointCommand {
    #[arg(short, long, help = "Commit message")]
    pub message: Option<String>,
    
    #[arg(long, help = "Skip running tests")]
    pub skip_tests: bool,
}

impl_mcp_tool!(CheckpointCommand, "b00t_checkpoint", ["checkpoint"]);

/// Create and populate a registry with all available MCP tools
pub fn create_mcp_registry() -> McpCommandRegistry {
    let mut builder = McpCommandRegistry::builder();
    
    // Register all MCP tools
    builder
        .register::<McpListCommand>()
        .register::<McpAddCommand>()
        .register::<McpOutputCommand>()
        .register::<CliDetectCommand>()
        .register::<CliDesiresCommand>()
        .register::<CliCheckCommand>()
        .register::<CliInstallCommand>()
        .register::<CliUpdateCommand>()
        .register::<CliUpCommand>()
        .register::<WhoamiCommand>()
        .register::<StatusCommand>()
        .register::<AiListCommand>()
        .register::<AiOutputCommand>()
        .register::<AppVscodeMcpInstallCommand>()
        .register::<AppClaudecodeMcpInstallCommand>()
        .register::<SessionInitCommand>()
        .register::<SessionStatusCommand>()
        .register::<SessionEndCommand>()
        .register::<LearnCommand>()
        .register::<CheckpointCommand>();
        
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clap_reflection::McpExecutor;
    use std::collections::HashMap;
    
    #[test]
    fn test_registry_creation() {
        let registry = create_mcp_registry();
        let tools = registry.get_tools();
        
        // Should have all registered tools
        assert!(!tools.is_empty());
        
        // Check specific tools exist
        let tool_names: Vec<&str> = tools.iter()
            .map(|t| t.name.as_ref())
            .collect();
        
        assert!(tool_names.contains(&"b00t_mcp_list"));
        assert!(tool_names.contains(&"b00t_cli_detect"));
        assert!(tool_names.contains(&"b00t_whoami"));
        assert!(tool_names.contains(&"b00t_status"));
    }
    
    #[test]
    fn test_tool_schema_generation() {
        let tool = McpListCommand::to_mcp_tool();
        assert_eq!(tool.name.as_ref(), "b00t_mcp_list");
        
        // Check schema has expected properties
        let schema = tool.input_schema.as_ref();
        assert!(schema.contains_key("type"));
        assert!(schema.contains_key("properties"));
        
        let properties = schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("json"));
    }
    
    #[test]
    fn test_params_conversion() {
        let mut params = HashMap::new();
        params.insert("json".to_string(), serde_json::json!(true));
        
        let args = McpListCommand::params_to_args(&params);
        assert!(args.contains(&"--json".to_string()));
    }
}