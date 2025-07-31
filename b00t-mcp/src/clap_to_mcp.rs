use quote::quote;
use syn::{Fields, Ident, Attribute, Meta};

/// Code generation for MCP tools from clap Commands enum
/// 
/// This module provides compile-time code generation to automatically
/// create MCP tool methods from clap command definitions, avoiding
/// the need for manual duplication of command structures.

/// Generate MCP tool implementations for all b00t-cli commands
pub fn generate_b00t_cli_tools() -> proc_macro2::TokenStream {
    // For now, generate the core b00t-cli commands manually
    // Future: Parse actual clap enums from b00t-cli at build time
    
    quote! {
        // Parameter structs for b00t-cli commands
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CliDetectParams {
            pub tool: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CliDesiresParams {
            pub tool: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CliInstallParams {
            pub tool: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CliUpdateParams {
            pub tool: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CliUpParams {}
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct McpAddParams {
            pub json_config: String,
            #[serde(default)]
            pub dwiw: bool,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct McpListParams {
            #[serde(default)]
            pub json: bool,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct StatusParams {
            pub filter: Option<String>,
            #[serde(default)]
            pub installed: bool,
            #[serde(default)]
            pub available: bool,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct InitHelloWorldParams {
            #[serde(default)]
            pub skip_redis: bool,
            #[serde(default)]
            pub skip_diagnostics: bool,
            #[serde(default)]
            pub skip_tour: bool,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct CheckpointParams {
            pub message: Option<String>,
            #[serde(default)]
            pub skip_tests: bool,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct SessionGetParams {
            pub key: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct SessionSetParams {
            pub key: String,
            pub value: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct SessionIncrParams {
            pub key: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        pub struct AppVscodeInstallMcpParams {
            pub server: String,
        }
        
        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]  
        pub struct AppClaudeCodeInstallMcpParams {
            pub server: String,
        }
    }
}

/// Generate MCP tool method implementations
pub fn generate_mcp_tool_methods() -> proc_macro2::TokenStream {
    quote! {
        #[rmcp::tool(description = "Detect currently installed version of a CLI tool")]
        async fn b00t_cli_detect(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<CliDetectParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.tool];
            self.execute_mcp_command("cli", "detect", &args).await
        }
        
        #[rmcp::tool(description = "Show desired version of a CLI tool from configuration")]  
        async fn b00t_cli_desires(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<CliDesiresParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.tool];
            self.execute_mcp_command("cli", "desires", &args).await
        }
        
        #[rmcp::tool(description = "Install a CLI tool")]
        async fn b00t_cli_install(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<CliInstallParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.tool];
            self.execute_mcp_command("cli", "install", &args).await
        }
        
        #[rmcp::tool(description = "Update a CLI tool")]
        async fn b00t_cli_update(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<CliUpdateParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.tool];
            self.execute_mcp_command("cli", "update", &args).await
        }
        
        #[rmcp::tool(description = "Update all outdated CLI tools")]
        async fn b00t_cli_up(
            &self,
            rmcp::handler::server::tool::Parameters(_params): rmcp::handler::server::tool::Parameters<CliUpParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![];
            self.execute_mcp_command("cli", "up", &args).await
        }
        
        #[rmcp::tool(description = "Add MCP server configuration")]
        async fn b00t_mcp_add(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<McpAddParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let mut args = vec![params.json_config];
            if params.dwiw {
                args.insert(0, "--dwiw".to_string());
            }
            self.execute_mcp_command("mcp", "add", &args).await
        }
        
        #[rmcp::tool(description = "List MCP server configurations")]
        async fn b00t_mcp_list(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<McpListParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = if params.json {
                vec!["--json".to_string()]
            } else {
                vec![]
            };
            self.execute_mcp_command("mcp", "list", &args).await
        }
        
        #[rmcp::tool(description = "Show status dashboard of all tools and services")]
        async fn b00t_status(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<StatusParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let mut args = vec![];
            if let Some(filter) = params.filter {
                args.extend(vec!["--filter".to_string(), filter]);
            }
            if params.installed {
                args.push("--installed".to_string());
            }
            if params.available {
                args.push("--available".to_string());
            }
            self.execute_mcp_command("status", "", &args).await
        }
        
        #[rmcp::tool(description = "Initialize hello world protocol - wake up all systems")]
        async fn b00t_init_hello_world(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<InitHelloWorldParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let mut args = vec![];
            if params.skip_redis {
                args.push("--skip-redis".to_string());
            }
            if params.skip_diagnostics {
                args.push("--skip-diagnostics".to_string());
            }
            if params.skip_tour {
                args.push("--skip-tour".to_string());
            }
            self.execute_mcp_command("init", "hello-world", &args).await
        }
        
        #[rmcp::tool(description = "Create checkpoint: commit all files and run tests")]
        async fn b00t_checkpoint(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<CheckpointParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let mut args = vec![];
            if let Some(message) = params.message {
                args.extend(vec!["--message".to_string(), message]);
            }
            if params.skip_tests {
                args.push("--skip-tests".to_string());
            }
            self.execute_mcp_command("checkpoint", "", &args).await
        }
        
        #[rmcp::tool(description = "Get session memory value")]
        async fn b00t_session_get(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<SessionGetParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.key];
            self.execute_mcp_command("session", "get", &args).await
        }
        
        #[rmcp::tool(description = "Set session memory value")]
        async fn b00t_session_set(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<SessionSetParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.key, params.value];
            self.execute_mcp_command("session", "set", &args).await
        }
        
        #[rmcp::tool(description = "Increment session memory counter")]
        async fn b00t_session_incr(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<SessionIncrParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.key];
            self.execute_mcp_command("session", "incr", &args).await
        }
        
        #[rmcp::tool(description = "Install MCP server to VSCode")]
        async fn b00t_app_vscode_install_mcp(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<AppVscodeInstallMcpParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.server];
            self.execute_mcp_command("app", "vscode mcp install", &args).await
        }
        
        #[rmcp::tool(description = "Install MCP server to Claude Code")]
        async fn b00t_app_claude_code_install_mcp(
            &self,
            rmcp::handler::server::tool::Parameters(params): rmcp::handler::server::tool::Parameters<AppClaudeCodeInstallMcpParams>,
        ) -> Result<rmcp::model::CallToolResult, rmcp::model::ErrorData> {
            let args = vec![params.server];
            self.execute_mcp_command("app", "claude-code mcp install", &args).await
        }
    }
}

/// Helper function to generate a complete MCP tool implementation
pub fn generate_complete_implementation() -> proc_macro2::TokenStream {
    let param_structs = generate_b00t_cli_tools();
    let _tool_methods = generate_mcp_tool_methods();
    
    quote! {
        use rmcp::{
            handler::server::{ServerHandler, router::tool::ToolRouter, tool::Parameters},
            model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
            tool_handler, tool_router,
        };
        use serde::{Deserialize, Serialize};
        use schemars::JsonSchema;
        
        // Generated parameter structs
        #param_structs
        
        // Generated tool methods would go here in the actual implementation
        // #tool_methods (commented out for now due to complexity)
    }
}

// Helper functions for parsing clap structures (future implementation)
pub fn extract_clap_about(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("clap") {
            if let Meta::List(_meta_list) = &attr.meta {
                // Parse about = "description" from clap attributes
                // This would need proper parsing of the token stream
                // For now, return a placeholder
                return Some("Generated from clap command".to_string());
            }
        }
    }
    None
}

pub fn field_to_param_type(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_type = &field.ty;
    
    quote! {
        pub #field_name: #field_type,
    }
}

pub fn generate_param_struct(variant_name: &Ident, fields: &Fields) -> proc_macro2::TokenStream {
    let struct_name = Ident::new(&format!("{}Params", variant_name), variant_name.span());
    
    match fields {
        Fields::Named(fields_named) => {
            let field_definitions: Vec<_> = fields_named.named.iter()
                .map(field_to_param_type)
                .collect();
                
            quote! {
                #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
                pub struct #struct_name {
                    #(#field_definitions)*
                }
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
                pub struct #struct_name {
                    pub value: String,
                }
            }
        }
        Fields::Unit => {
            quote! {
                #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
                pub struct #struct_name {}
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub params: Vec<ParamInfo>,
}

#[derive(Debug)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
    pub optional: bool,
}