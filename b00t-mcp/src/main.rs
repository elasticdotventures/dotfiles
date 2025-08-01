use anyhow::Result;
use clap::{Arg, Command};
use rmcp::{ServiceExt, transport::io::stdio};
use std::path::Path;

use b00t_mcp::B00tMcpServerRusty;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("b00t-mcp")
        .version(env!("GIT_REPO_TAG_VERSION"))
        .author("b00t-mcp contributors")
        .about("MCP Server for b00t-cli command proxy with ACL filtering")
        .arg(
            Arg::new("working-dir")
                .short('d')
                .long("directory")
                .value_name("DIR")
                .help("Working directory for the MCP server")
                .default_value("."),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to ACL configuration file")
                .default_value("~/.dotfiles/b00t-mcp-acl.toml"),
        )
        .arg(
            Arg::new("stdio")
                .long("stdio")
                .help("Run as MCP server using stdio transport")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mode")
                .help("Transport mode (stdio)")
                .value_parser(["stdio"])
                .index(1),
        )
        .get_matches();

    let working_dir = matches.get_one::<String>("working-dir").unwrap();
    let config_path = matches.get_one::<String>("config").unwrap();
    let working_path = Path::new(working_dir);

    let is_stdio_mode = matches.get_flag("stdio") || matches.get_one::<String>("mode").map_or(false, |m| m == "stdio");
    
    if is_stdio_mode {
        // Run as MCP server
        // eprintln!(
        //     "Starting b00t-mcp MCP server in directory: {} with config: {}",
        //     working_path.display(),
        //     config_path
        // );

        // No stderr output in stdio mode as it breaks the MCP protocol
        let server = B00tMcpServerRusty::new(working_path, config_path)?;
        let running_service = server.serve(stdio()).await?;

        // Keep the server running
        running_service.waiting().await?;
    } else {
        // Show usage information
        println!("b00t-mcp v{}", env!("CARGO_PKG_VERSION"));
        println!("MCP Server for b00t-cli command proxy with ACL filtering");
        println!();
        println!("Usage:");
        println!(
            "  {} stdio                             Run as MCP server with stdio transport",
            env!("CARGO_PKG_NAME")
        );
        println!(
            "  {} --stdio                           Run as MCP server with stdio transport",
            env!("CARGO_PKG_NAME")
        );
        println!(
            "  {} --directory <DIR> stdio           Run MCP server in specific directory",
            env!("CARGO_PKG_NAME")
        );
        println!(
            "  {} --directory <DIR> --stdio         Run MCP server in specific directory",
            env!("CARGO_PKG_NAME")
        );
        println!(
            "  {} --config <FILE> stdio             Run MCP server with custom ACL config",
            env!("CARGO_PKG_NAME")
        );
        println!(
            "  {} --config <FILE> --stdio           Run MCP server with custom ACL config",
            env!("CARGO_PKG_NAME")
        );
        println!();
        println!("🦀 Rusty MCP Tools:");
        println!("  Tools are compile-time generated from b00t-cli CLAP structures");
        println!("  Type-safe execution with zero runtime parsing failures");
        println!("  Examples: b00t_mcp_list, b00t_cli_detect, b00t_whoami");
        println!();
        println!("Example usage with MCP client:");
        println!("  Configure in .mcp.json or MCP client settings");
    }

    Ok(())
}