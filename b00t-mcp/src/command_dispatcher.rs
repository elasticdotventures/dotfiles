use anyhow::{Context, Result};
use rmcp::{
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router,
    schemars::{self, JsonSchema},
};
use serde::Deserialize;
use serde_json::{Value, Map};
use std::process::Command;

use crate::acl::AclFilter;

/// Generic parameter structure for all b00t-cli commands
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenericParams {
    /// Command arguments as key-value pairs
    #[serde(flatten)]
    pub args: Map<String, Value>,
}

/// Command dispatcher that routes MCP calls to b00t-cli dynamically
#[derive(Clone)]
pub struct CommandDispatcher {
    working_dir: std::path::PathBuf,
    acl_filter: AclFilter,
}

impl CommandDispatcher {
    pub fn new<P: AsRef<std::path::Path>>(working_dir: P, acl_filter: AclFilter) -> Self {
        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            acl_filter,
        }
    }

    /// Execute b00t-cli command with dynamic argument mapping
    pub fn execute_command(&self, command_path: &[&str], params: &GenericParams) -> Result<String> {
        let mut args = Vec::new();
        
        // Build command path (e.g., ["cli", "detect"] -> "cli detect")
        for segment in command_path {
            args.push(segment.to_string());
        }
        
        // Convert parameters to command arguments
        self.params_to_args(&mut args, params)?;
        
        // Validate with ACL
        let full_command = command_path.join(" ");
        if !self.acl_filter.is_allowed(&full_command, &args) {
            anyhow::bail!("Command '{}' with args {:?} is not allowed by ACL policy", full_command, args);
        }

        // Execute b00t-cli
        let mut cmd = Command::new("b00t-cli");
        cmd.current_dir(&self.working_dir);
        
        for arg in &args {
            cmd.arg(arg);
        }

        let output = cmd.output()
            .with_context(|| format!("Failed to execute b00t-cli {}", args.join(" ")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("b00t-cli command failed: {}", stderr);
        }
    }

    /// Convert GenericParams to command line arguments
    fn params_to_args(&self, args: &mut Vec<String>, params: &GenericParams) -> Result<()> {
        for (key, value) in &params.args {
            match value {
                Value::Bool(true) => {
                    // Convert boolean flags: {"flag": true} -> ["--flag"]
                    if key.len() == 1 {
                        args.push(format!("-{}", key));
                    } else {
                        args.push(format!("--{}", key));
                    }
                }
                Value::Bool(false) => {
                    // Skip false boolean flags
                }
                Value::String(s) => {
                    // Handle positional vs named arguments
                    if self.is_positional_arg(key) {
                        args.push(s.clone());
                    } else {
                        if key.len() == 1 {
                            args.push(format!("-{}", key));
                        } else {
                            args.push(format!("--{}", key));
                        }
                        args.push(s.clone());
                    }
                }
                Value::Number(n) => {
                    if key.len() == 1 {
                        args.push(format!("-{}", key));
                    } else {
                        args.push(format!("--{}", key));
                    }
                    args.push(n.to_string());
                }
                Value::Array(arr) => {
                    // Handle array arguments
                    for item in arr {
                        if let Value::String(s) = item {
                            args.push(s.clone());
                        }
                    }
                }
                Value::Null => {
                    // Skip null values
                }
                Value::Object(_) => {
                    // Flatten nested objects if needed
                    anyhow::bail!("Nested objects not supported in parameters: {}", key);
                }
            }
        }
        Ok(())
    }

    /// Determine if argument is positional based on common patterns
    fn is_positional_arg(&self, key: &str) -> bool {
        matches!(key, "tool" | "command" | "name" | "value" | "key" | "server" | "topic" | "message")
    }

    /// Create MCP tool result from command execution
    pub async fn execute_mcp_tool(&self, command_path: &[&str], params: GenericParams) -> Result<CallToolResult, rmcp::model::ErrorData> {
        #[derive(serde::Serialize)]
        struct B00tOutput {
            output: String,
            success: bool,
        }
        
        match self.execute_command(command_path, &params) {
            Ok(output) => {
                let result = B00tOutput {
                    output,
                    success: true,
                };
                let content = serde_json::to_string_pretty(&result).unwrap();
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            Err(e) => {
                let result = B00tOutput {
                    output: e.to_string(),
                    success: false,
                };
                let content = serde_json::to_string_pretty(&result).unwrap();
                Ok(CallToolResult::error(vec![Content::text(content)]))
            }
        }
    }
}

/// Macro to generate MCP tools that route to b00t-cli commands
macro_rules! generate_b00t_tool {
    ($method_name:ident, $description:expr, $command_path:expr) => {
        #[tool(description = $description)]
        pub async fn $method_name(
            &self,
            Parameters(params): Parameters<GenericParams>,
        ) -> Result<CallToolResult, rmcp::model::ErrorData> {
            self.dispatcher.execute_mcp_tool(&$command_path, params).await
        }
    };
}

/// Tool registry for dynamic MCP tool registration
#[derive(Clone)]
pub struct ToolRegistry {
    pub dispatcher: CommandDispatcher,
}

impl ToolRegistry {
    pub fn new(working_dir: std::path::PathBuf, acl_filter: AclFilter) -> Self {
        Self {
            dispatcher: CommandDispatcher::new(working_dir, acl_filter),
        }
    }
}

#[tool_router]
impl ToolRegistry {
    // CLI tools
    generate_b00t_tool!(b00t_cli_detect, "Detect currently installed version of a CLI tool", ["cli", "detect"]);
    generate_b00t_tool!(b00t_cli_desires, "Show desired version of a CLI tool from configuration", ["cli", "desires"]);  
    generate_b00t_tool!(b00t_cli_install, "Install a CLI tool", ["cli", "install"]);
    generate_b00t_tool!(b00t_cli_update, "Update a CLI tool", ["cli", "update"]);
    generate_b00t_tool!(b00t_cli_up, "Update all outdated CLI tools", ["cli", "up"]);

    // MCP tools  
    generate_b00t_tool!(b00t_mcp_add, "Add MCP server configuration", ["mcp", "add"]);
    generate_b00t_tool!(b00t_mcp_list, "List MCP server configurations", ["mcp", "list"]);

    // Status and system tools
    generate_b00t_tool!(b00t_status, "Show status dashboard of all tools and services", ["status"]);
    generate_b00t_tool!(b00t_init_hello_world, "Initialize hello world protocol - wake up all systems", ["init", "hello-world"]);
    generate_b00t_tool!(b00t_checkpoint, "Create checkpoint: commit all files and run tests", ["checkpoint"]);

    // Session management
    generate_b00t_tool!(b00t_session_get, "Get session memory value", ["session", "get"]);
    generate_b00t_tool!(b00t_session_set, "Set session memory value", ["session", "set"]);
    generate_b00t_tool!(b00t_session_incr, "Increment session memory counter", ["session", "incr"]);

    // App integrations
    generate_b00t_tool!(b00t_app_vscode_install_mcp, "Install MCP server to VSCode", ["app", "vscode", "mcp", "install"]);
    generate_b00t_tool!(b00t_app_claude_code_install_mcp, "Install MCP server to Claude Code", ["app", "claude-code", "mcp", "install"]);

    // Learn system
    generate_b00t_tool!(b00t_learn, "Show learning resources for a topic", ["learn"]);

    // AI tools
    generate_b00t_tool!(b00t_ai_list, "List AI provider configurations", ["ai", "list"]);
    generate_b00t_tool!(b00t_ai_add, "Add AI provider configuration", ["ai", "add"]);

    // K8s tools
    generate_b00t_tool!(b00t_k8s_list, "List Kubernetes resources", ["k8s", "list"]);
    generate_b00t_tool!(b00t_k8s_deploy, "Deploy to Kubernetes", ["k8s", "deploy"]);

    // Identity tools
    generate_b00t_tool!(b00t_whoami, "Show agent identity and context information", ["whoami"]);
    generate_b00t_tool!(b00t_whatismy_ip, "Show IP address information", ["whatismy", "ip"]);
    generate_b00t_tool!(b00t_whatismy_hostname, "Show hostname information", ["whatismy", "hostname"]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::acl::{AclConfig, Policy};

    fn create_test_dispatcher() -> (CommandDispatcher, TempDir) {
        use std::collections::HashMap;
        
        let temp_dir = TempDir::new().unwrap();
        let acl_config = AclConfig {
            default_policy: Policy::Allow,
            commands: HashMap::new(),
            patterns: None,
        };
        let acl_filter = AclFilter::new(acl_config).unwrap();
        let dispatcher = CommandDispatcher::new(temp_dir.path(), acl_filter);
        (dispatcher, temp_dir)
    }

    #[test]
    fn test_params_to_args_conversion() {
        let (dispatcher, _temp) = create_test_dispatcher();
        let mut args = vec!["cli".to_string(), "detect".to_string()];
        
        let mut params_map = Map::new();
        params_map.insert("tool".to_string(), Value::String("node".to_string()));
        params_map.insert("verbose".to_string(), Value::Bool(true));
        params_map.insert("count".to_string(), Value::Number(serde_json::Number::from(5)));
        
        let params = GenericParams { args: params_map };
        
        dispatcher.params_to_args(&mut args, &params).unwrap();
        
        // Should contain: ["cli", "detect", "node", "--verbose", "--count", "5"]
        assert!(args.contains(&"node".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--count".to_string()));
        assert!(args.contains(&"5".to_string()));
    }

    #[test]
    fn test_positional_arg_detection() {
        let (dispatcher, _temp) = create_test_dispatcher();
        
        assert!(dispatcher.is_positional_arg("tool"));
        assert!(dispatcher.is_positional_arg("name"));
        assert!(dispatcher.is_positional_arg("value"));
        assert!(!dispatcher.is_positional_arg("verbose"));
        assert!(!dispatcher.is_positional_arg("count"));
    }

    #[tokio::test]
    async fn test_mcp_tool_execution() {
        let (dispatcher, _temp) = create_test_dispatcher();
        
        let mut params_map = Map::new();
        params_map.insert("tool".to_string(), Value::String("nonexistent".to_string()));
        let params = GenericParams { args: params_map };
        
        // This should return an error result (not panic) since b00t-cli will fail
        let result = dispatcher.execute_mcp_tool(&["cli", "detect"], params).await;
        assert!(result.is_ok()); // The MCP call succeeds, but b00t-cli command may fail
    }
}