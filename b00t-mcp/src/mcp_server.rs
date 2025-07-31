use anyhow::{Context, Result};
use rmcp::{
    handler::server::{ServerHandler, router::tool::ToolRouter, tool::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    schemars::{self, JsonSchema},
};
use std::path::Path;

use crate::acl::AclFilter;
use crate::command_dispatcher::{ToolRegistry, GenericParams};

// B00tOutput struct is now defined in command_dispatcher.rs

#[derive(Clone)]
pub struct B00tMcpServer {
    working_dir: std::path::PathBuf,
    acl_filter: AclFilter,
    tool_router: ToolRouter<Self>,
    tool_registry: ToolRegistry,
}

impl B00tMcpServer {
    pub fn new<P: AsRef<Path>>(working_dir: P, config_path: &str) -> Result<Self> {
        let acl_filter = AclFilter::load_from_file(config_path)
            .context("Failed to load ACL configuration")?;

        let tool_registry = ToolRegistry::new(
            working_dir.as_ref().to_path_buf(),
            acl_filter.clone()
        );

        Ok(Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            acl_filter,
            tool_router: Self::tool_router(),
            tool_registry,
        })
    }

}

// Delegate all tools to the ToolRegistry - this exposes the complete generic command dispatcher
#[tool_router]
impl B00tMcpServer {
    // CLI tools
    #[tool(description = "Detect currently installed version of a CLI tool")]
    async fn b00t_cli_detect(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_cli_detect(params).await
    }

    #[tool(description = "Show desired version of a CLI tool from configuration")]
    async fn b00t_cli_desires(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_cli_desires(params).await
    }

    #[tool(description = "Install a CLI tool")]
    async fn b00t_cli_install(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_cli_install(params).await
    }

    #[tool(description = "Update a CLI tool")]
    async fn b00t_cli_update(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_cli_update(params).await
    }

    #[tool(description = "Update all outdated CLI tools")]
    async fn b00t_cli_up(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_cli_up(params).await
    }

    // MCP tools  
    #[tool(description = "Add MCP server configuration")]
    async fn b00t_mcp_add(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_mcp_add(params).await
    }

    #[tool(description = "List MCP server configurations")]
    async fn b00t_mcp_list(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_mcp_list(params).await
    }

    // Status and system tools
    #[tool(description = "Show status dashboard of all tools and services")]
    async fn b00t_status(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_status(params).await
    }

    #[tool(description = "Initialize hello world protocol - wake up all systems")]
    async fn b00t_init_hello_world(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_init_hello_world(params).await
    }

    #[tool(description = "Create checkpoint: commit all files and run tests")]
    async fn b00t_checkpoint(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_checkpoint(params).await
    }

    // Session management
    #[tool(description = "Get session memory value")]
    async fn b00t_session_get(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_session_get(params).await
    }

    #[tool(description = "Set session memory value")]
    async fn b00t_session_set(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_session_set(params).await
    }

    #[tool(description = "Increment session memory counter")]
    async fn b00t_session_incr(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_session_incr(params).await
    }

    // App integrations
    #[tool(description = "Install MCP server to VSCode")]
    async fn b00t_app_vscode_install_mcp(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_app_vscode_install_mcp(params).await
    }

    #[tool(description = "Install MCP server to Claude Code")]
    async fn b00t_app_claude_code_install_mcp(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_app_claude_code_install_mcp(params).await
    }

    // Learn system
    #[tool(description = "Show learning resources for a topic")]
    async fn b00t_learn(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_learn(params).await
    }

    // AI tools
    #[tool(description = "List AI provider configurations")]
    async fn b00t_ai_list(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_ai_list(params).await
    }

    #[tool(description = "Add AI provider configuration")]
    async fn b00t_ai_add(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_ai_add(params).await
    }

    // K8s tools
    #[tool(description = "List Kubernetes resources")]
    async fn b00t_k8s_list(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_k8s_list(params).await
    }

    #[tool(description = "Deploy to Kubernetes")]
    async fn b00t_k8s_deploy(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_k8s_deploy(params).await
    }

    // Identity tools
    #[tool(description = "Show agent identity and context information")]
    async fn b00t_whoami(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_whoami(params).await
    }

    #[tool(description = "Show IP address information")]
    async fn b00t_whatismy_ip(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_whatismy_ip(params).await
    }

    #[tool(description = "Show hostname information")]
    async fn b00t_whatismy_hostname(
        &self,
        params: Parameters<GenericParams>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        self.tool_registry.b00t_whatismy_hostname(params).await
    }
}

#[tool_handler]
impl ServerHandler for B00tMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation::from_build_env(),
            instructions: Some("MCP server for b00t-cli command proxy with ACL filtering. Provides secure access to b00t-cli commands.".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
        }
    }
}