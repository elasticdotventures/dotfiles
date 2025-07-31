use anyhow::{Context, Result};
use rmcp::{
    handler::server::{ServerHandler, tool::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    tool,
};
use std::path::Path;
use std::process::Command;

use crate::acl::AclFilter;
use crate::params::*;

#[derive(Clone)]
pub struct B00tMcpServer {
    working_dir: std::path::PathBuf,
    acl_filter: AclFilter,
}

impl B00tMcpServer {
    pub fn new<P: AsRef<Path>>(working_dir: P, config_path: &str) -> Result<Self> {
        let acl_filter = AclFilter::load_from_file(config_path)
            .context("Failed to load ACL configuration")?;

        Ok(Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            acl_filter,
        })
    }

    /// Execute b00t-cli command with ACL validation
    fn execute_b00t_command(&self, command_args: &[&str]) -> Result<String> {
        // Validate with ACL
        let command_str = command_args.join(" ");
        let args: Vec<String> = command_args.iter().map(|s| s.to_string()).collect();

        if !self.acl_filter.is_allowed(&command_str, &args) {
            anyhow::bail!("Command '{}' is not allowed by ACL policy", command_str);
        }

        // Execute b00t-cli
        let mut cmd = Command::new("b00t-cli");
        cmd.current_dir(&self.working_dir);

        for arg in command_args {
            cmd.arg(arg);
        }

        let output = cmd.output()
            .with_context(|| format!("Failed to execute b00t-cli {}", command_args.join(" ")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("b00t-cli command failed: {}", stderr);
        }
    }

    /// Create successful MCP tool result
    fn success_result(&self, output: String) -> CallToolResult {
        #[derive(serde::Serialize)]
        struct B00tOutput {
            output: String,
            success: bool,
        }

        let result = B00tOutput {
            output,
            success: true,
        };
        let content = serde_json::to_string_pretty(&result).unwrap();
        CallToolResult::success(vec![Content::text(content)])
    }

    /// Create error MCP tool result
    fn error_result(&self, error: String) -> CallToolResult {
        #[derive(serde::Serialize)]
        struct B00tOutput {
            output: String,
            success: bool,
        }

        let result = B00tOutput {
            output: error,
            success: false,
        };
        let content = serde_json::to_string_pretty(&result).unwrap();
        CallToolResult::error(vec![Content::text(content)])
    }

    /// Detect currently installed version of a CLI tool
    #[tool(description = "Detect currently installed version of a CLI tool")]
    pub async fn b00t_cli_detect(
        &self,
        Parameters(params): Parameters<DetectParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["cli", "detect", &params.tool];
        if params.verbose {
            args.push("--verbose");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Show desired version of a CLI tool from configuration
    #[tool(description = "Show desired version of a CLI tool from configuration")]
    pub async fn b00t_cli_desires(
        &self,
        Parameters(params): Parameters<DesiresParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["cli", "desires", &params.tool];
        if params.verbose {
            args.push("--verbose");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Install a CLI tool
    #[tool(description = "Install a CLI tool")]
    pub async fn b00t_cli_install(
        &self,
        Parameters(params): Parameters<InstallParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["cli", "install", &params.tool];

        if let Some(ref version) = params.version {
            args.extend(vec!["--version", version.as_str()]);
        }
        if params.force {
            args.push("--force");
        }
        if params.verbose {
            args.push("--verbose");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Show status dashboard of all tools and services
    #[tool(description = "Show status dashboard of all tools and services")]
    pub async fn b00t_status(
        &self,
        Parameters(params): Parameters<StatusParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["status"];

        if params.verbose {
            args.push("--verbose");
        }
        if params.detailed {
            args.push("--detailed");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Show agent identity and context information
    #[tool(description = "Show agent identity and context information")]
    pub async fn b00t_whoami(
        &self,
        Parameters(params): Parameters<WhoamiParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["whoami"];

        if params.verbose {
            args.push("--verbose");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Update a CLI tool to latest or specified version
    #[tool(description = "Update a CLI tool to latest or specified version")]
    pub async fn b00t_cli_update(
        &self,
        Parameters(params): Parameters<UpdateParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["cli", "update", &params.tool];

        if let Some(ref version) = params.version {
            args.extend(vec!["--version", version.as_str()]);
        }
        if params.force {
            args.push("--force");
        }
        if params.verbose {
            args.push("--verbose");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Update all tools and services to their desired versions
    #[tool(description = "Update all tools and services to their desired versions")]
    pub async fn b00t_up(
        &self,
        Parameters(params): Parameters<UpParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["up"];

        if params.verbose {
            args.push("--verbose");
        }
        if params.dry_run {
            args.push("--dry-run");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Add a new MCP server configuration
    #[tool(description = "Add a new MCP server configuration")]
    pub async fn b00t_mcp_add(
        &self,
        Parameters(params): Parameters<McpAddParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["mcp".to_string(), "add".to_string(), params.name.clone(), params.command.clone()];

        // Add arguments
        for arg in &params.args {
            args.push(arg.clone());
        }

        // Add environment variables
        for (key, value) in &params.env {
            args.push("--env".to_string());
            args.push(format!("{}={}", key, value));
        }

        let string_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.execute_b00t_command(&string_refs) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// List configured MCP servers
    #[tool(description = "List configured MCP servers")]
    pub async fn b00t_mcp_list(
        &self,
        Parameters(params): Parameters<McpListParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["mcp", "list"];

        if params.verbose {
            args.push("--verbose");
        }
        if params.enabled_only {
            args.push("--enabled-only");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Get value from session memory
    #[tool(description = "Get value from session memory")]
    pub async fn b00t_session_get(
        &self,
        Parameters(params): Parameters<SessionGetParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let args = vec!["session", "get", &params.key];

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Set value in session memory
    #[tool(description = "Set value in session memory")]
    pub async fn b00t_session_set(
        &self,
        Parameters(params): Parameters<SessionSetParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let args = vec!["session", "set", &params.key, &params.value];

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Increment numeric value in session memory
    #[tool(description = "Increment numeric value in session memory")]
    pub async fn b00t_session_incr(
        &self,
        Parameters(params): Parameters<SessionIncrParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["session".to_string(), "incr".to_string(), params.key.clone()];

        if params.amount != 1 {
            args.push("--amount".to_string());
            args.push(params.amount.to_string());
        }

        let string_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match self.execute_b00t_command(&string_refs) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Learn about a topic using b00t knowledge system
    #[tool(description = "Learn about a topic using b00t knowledge system")]
    pub async fn b00t_learn(
        &self,
        Parameters(params): Parameters<LearnParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["learn", &params.topic];

        if params.detailed {
            args.push("--detailed");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Show current IP address
    #[tool(description = "Show current IP address")]
    pub async fn b00t_whatismyip(
        &self,
        Parameters(params): Parameters<WhatismyIpParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["whatismyip"];

        if params.detailed {
            args.push("--detailed");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }

    /// Show current hostname
    #[tool(description = "Show current hostname")]
    pub async fn b00t_whatismyhostname(
        &self,
        Parameters(params): Parameters<WhatismyHostnameParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let mut args = vec!["whatismyhostname"];

        if params.detailed {
            args.push("--detailed");
        }

        match self.execute_b00t_command(&args) {
            Ok(output) => Ok(self.success_result(output)),
            Err(e) => Ok(self.error_result(e.to_string())),
        }
    }
}

impl ServerHandler for B00tMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation::from_build_env(),
            instructions: Some("MCP server for b00t-cli command proxy with ACL filtering. Provides secure access to b00t-cli commands with type-safe parameters.".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
        }
    }
}