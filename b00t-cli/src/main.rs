
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use regex::Regex;
use duct::cmd;
use semver::Version;
use anyhow::{Result, Context};
use b00t_cli::{McpServer, McpConfig, normalize_mcp_json};

mod integration_tests;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long, default_value = "~/.dotfiles/_b00t_")]
    path: String,
}

#[derive(Parser)]
enum Commands {
    #[clap(about = "Detect the version of a command")]
    Detect {
        #[clap(help = "The command to detect")]
        command: String,
    },
    #[clap(about = "Show the desired version of a command")]
    Desires {
        #[clap(help = "The command to check")]
        command: String,
    },
    #[clap(about = "Install a command")]
    Install {
        #[clap(help = "The command to install")]
        command: String,
    },
    #[clap(about = "Update a command")]
    Update {
        #[clap(help = "The command to update")]
        command: String,
    },
    #[clap(name = ".", about = "Compare installed and desired versions")]
    Dot {
        #[clap(help = "The command to check")]
        command: String,
    },
    #[clap(about = "Update all commands")]
    Up,
    #[clap(about = "MCP (Model Context Protocol) server management")]
    Mcp {
        #[clap(subcommand)]
        mcp_command: McpCommands,
    },
    #[clap(about = "VSCode integration")]
    Vscode {
        #[clap(subcommand)]
        vscode_command: VscodeCommands,
    },
    #[clap(about = "Claude Code integration")]
    ClaudeCode {
        #[clap(subcommand)]
        claude_command: ClaudeCodeCommands,
    },
}

#[derive(Parser)]
enum McpCommands {
    #[clap(about = "Add MCP server configuration")]
    Add {
        #[clap(help = "MCP server JSON configuration")]
        json: String,
        #[clap(long, help = "Do What I Want - auto-cleanup and format JSON")]
        dwiw: bool,
    },
    #[clap(about = "List available MCP server configurations")]
    List,
}

#[derive(Parser)]
enum VscodeCommands {
    #[clap(about = "Install MCP server to VSCode")]
    Install {
        #[clap(subcommand)]
        install_command: VscodeInstallCommands,
    },
}

#[derive(Parser)]
enum VscodeInstallCommands {
    #[clap(about = "Install MCP server by name")]
    Mcp {
        #[clap(help = "Name of the MCP server to install")]
        name: String,
    },
}

#[derive(Parser)]
enum ClaudeCodeCommands {
    #[clap(about = "Install MCP server to Claude Code")]
    Install {
        #[clap(subcommand)]
        install_command: ClaudeCodeInstallCommands,
    },
}

#[derive(Parser)]
enum ClaudeCodeInstallCommands {
    #[clap(about = "Install MCP server by name")]
    Mcp {
        #[clap(help = "Name of the MCP server to install")]
        name: String,
    },
}

#[derive(Deserialize, Debug)]
struct Config {
    b00t: BootConfig,
}

#[derive(Deserialize, Debug)]
struct BootConfig {
    name: String,
    desires: String,
    install: String,
    update: Option<String>,
    version: String,
    version_regex: Option<String>,
    hint: String,
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Detect { command } => detect(command, &cli.path),
        Commands::Desires { command } => desires(command, &cli.path),
        Commands::Install { command } => install(command, &cli.path),
        Commands::Update { command } => update(command, &cli.path),
        Commands::Dot { command } => dot(command, &cli.path),
        Commands::Up => up(&cli.path),
        Commands::Mcp { mcp_command } => match mcp_command {
            McpCommands::Add { json, dwiw } => {
                if let Err(e) = mcp_add(json, *dwiw, &cli.path) {
                    eprintln!("Error adding MCP server: {}", e);
                    std::process::exit(1);
                }
            }
            McpCommands::List => {
                if let Err(e) = mcp_list(&cli.path) {
                    eprintln!("Error listing MCP servers: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Vscode { vscode_command } => match vscode_command {
            VscodeCommands::Install { install_command } => match install_command {
                VscodeInstallCommands::Mcp { name } => {
                    if let Err(e) = vscode_install_mcp(name, &cli.path) {
                        eprintln!("Error installing MCP server to VSCode: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::ClaudeCode { claude_command } => match claude_command {
            ClaudeCodeCommands::Install { install_command } => match install_command {
                ClaudeCodeInstallCommands::Mcp { name } => {
                    if let Err(e) = claude_code_install_mcp(name, &cli.path) {
                        eprintln!("Error installing MCP server to Claude Code: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}

fn get_config(command: &str, path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut path_buf = PathBuf::new();
    path_buf.push(shellexpand::tilde(path).to_string());
    path_buf.push(format!("{}.toml", command));

    if !path_buf.exists() {
        eprintln!("{} UNDEFINED", command);
        std::process::exit(100);
    }

    let content = fs::read_to_string(&path_buf)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn detect(command: &str, path: &str) {
    if let Some(version) = get_installed_version(command, path) {
        println!("{}", version);
    } else {
        eprintln!("🥾😱 {} MISSING", command);
        std::process::exit(2);
    }
}

fn desires(command: &str, path: &str) {
    match get_config(command, path) {
        Ok(config) => {
            println!("{}", config.b00t.desires);
        }
        Err(e) => eprintln!("Error reading config for {}: {}", command, e),
    }
}

fn install(command: &str, path: &str) {
    match get_config(command, path) {
        Ok(config) => {
            println!("Installing {}...", command);
            let install_cmd = &config.b00t.install;
            let result = cmd!("bash", "-c", install_cmd).run();
            match result {
                Ok(_) => println!("Installation complete."),
                Err(e) => eprintln!("Installation failed: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading config for {}: {}", command, e),
    }
}

fn update(command: &str, path: &str) {
    match get_config(command, path) {
        Ok(config) => {
            let update_cmd = config.b00t.update.as_ref().unwrap_or(&config.b00t.install);
            println!("Updating {}...", command);
            let result = cmd!("bash", "-c", update_cmd).run();
            match result {
                Ok(_) => println!("Update complete."),
                Err(e) => eprintln!("Update failed: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading config for {}: {}", command, e),
    }
}

fn get_installed_version(command: &str, path: &str) -> Option<String> {
    if let Ok(config) = get_config(command, path) {
        let version_cmd = &config.b00t.version;
        if let Ok(output) = cmd!("bash", "-c", version_cmd).read() {
            let re = Regex::new(config.b00t.version_regex.as_deref().unwrap_or("\\d+\\.\\d+\\.\\d+")).unwrap();
            if let Some(caps) = re.captures(&output) {
                return Some(caps[0].to_string());
            }
        }
    }
    None
}

fn dot(command: &str, path: &str) {
    let desired_version_str = match get_config(command, path) {
        Ok(config) => config.b00t.desires,
        Err(e) => {
            eprintln!("Error reading config for {}: {}", command, e);
            std::process::exit(1);
        }
    };

    let desired_version = Version::parse(&desired_version_str).unwrap();

    if let Some(installed_version_str) = get_installed_version(command, path) {
        let installed_version = Version::parse(&installed_version_str).unwrap();

        if installed_version == desired_version {
            println!("🥾👍🏻 {}", command);
            std::process::exit(0);
        } else if installed_version > desired_version {
            println!("🥾🐣 {} {}", command, installed_version);
            std::process::exit(0);
        } else {
            println!("🥾😭 {} IS {} WANTS {}", command, installed_version, desired_version);
            std::process::exit(1);
        }
    } else {
        println!("🥾😱 {} MISSING", command);
        std::process::exit(2);
    }
}

fn up(path: &str) {
    let expanded_path = shellexpand::tilde(path).to_string();
    let entries = match fs::read_dir(&expanded_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory {}: {}", &expanded_path, e);
            return;
        }
    };

    for entry in entries {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            if entry_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(command) = entry_path.file_stem().and_then(|s| s.to_str()) {
                    let desired_version_str = match get_config(command, path) {
                        Ok(config) => config.b00t.desires,
                        Err(_) => continue,
                    };

                    let desired_version = Version::parse(&desired_version_str).unwrap();

                    if let Some(installed_version_str) = get_installed_version(command, path) {
                        let installed_version = Version::parse(&installed_version_str).unwrap();

                        if installed_version < desired_version {
                            println!("Updating {}...", command);
                            update(command, path);
                        }
                    } else {
                        println!("Installing {}...", command);
                        install(command, path);
                    }
                }
            }
        }
    }
}


fn create_mcp_toml_config(server: &McpServer, path: &str) -> Result<()> {
    let config = McpConfig {
        mcp: server.clone(),
    };

    let toml_content = toml::to_string(&config)
        .context("Failed to serialize MCP config to TOML")?;

    let mut path_buf = PathBuf::new();
    path_buf.push(shellexpand::tilde(path).to_string());
    path_buf.push(format!("{}.mcp-json.toml", server.name));

    fs::write(&path_buf, toml_content)
        .context(format!("Failed to write MCP config to {}", path_buf.display()))?;

    println!("Created MCP config: {}", path_buf.display());
    Ok(())
}


fn mcp_add(json: &str, dwiw: bool, path: &str) -> Result<()> {
    let server = normalize_mcp_json(json, dwiw)?;
    
    create_mcp_toml_config(&server, path)?;
    
    println!("MCP server '{}' configuration saved.", server.name);
    println!("To install to VSCode: b00t-cli vscode install mcp {}", server.name);
    
    Ok(())
}

fn mcp_list(path: &str) -> Result<()> {
    let expanded_path = shellexpand::tilde(path).to_string();
    let entries = match fs::read_dir(&expanded_path) {
        Ok(entries) => entries,
        Err(e) => {
            anyhow::bail!("Error reading directory {}: {}", &expanded_path, e);
        }
    };

    let mut mcp_servers = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            if let Some(file_name) = entry_path.file_name().and_then(|s| s.to_str()) {
                if file_name.ends_with(".mcp-json.toml") {
                    if let Some(server_name) = file_name.strip_suffix(".mcp-json.toml") {
                        // Try to read the config to get details
                        match get_mcp_config(server_name, path) {
                            Ok(server) => {
                                mcp_servers.push((server_name.to_string(), Some(server)));
                            }
                            Err(_) => {
                                mcp_servers.push((server_name.to_string(), None));
                            }
                        }
                    }
                }
            }
        }
    }

    if mcp_servers.is_empty() {
        println!("No MCP server configurations found in {}", expanded_path);
        println!("Use 'b00t-cli mcp add <json>' to add MCP server configurations.");
    } else {
        println!("Available MCP servers in {}:", expanded_path);
        println!();
        for (name, server) in mcp_servers {
            match server {
                Some(s) => {
                    println!("📋 {} ({})", name, s.command);
                    if !s.args.is_empty() {
                        println!("   args: {}", s.args.join(" "));
                    }
                }
                None => {
                    println!("❌ {} (error reading config)", name);
                }
            }
        }
        println!();
        println!("To install to VSCode: b00t-cli vscode install mcp <name>");
        println!("To install to Claude Code: b00t-cli claude-code install mcp <name>");
    }

    Ok(())
}

fn get_mcp_config(name: &str, path: &str) -> Result<McpServer> {
    let mut path_buf = PathBuf::new();
    path_buf.push(shellexpand::tilde(path).to_string());
    path_buf.push(format!("{}.mcp-json.toml", name));

    if !path_buf.exists() {
        anyhow::bail!("MCP server '{}' not found. Use 'b00t-cli mcp add' to create it first.", name);
    }

    let content = fs::read_to_string(&path_buf)
        .context(format!("Failed to read MCP config from {}", path_buf.display()))?;
    
    let config: McpConfig = toml::from_str(&content)
        .context("Failed to parse MCP config TOML")?;

    Ok(config.mcp)
}

fn vscode_install_mcp(name: &str, path: &str) -> Result<()> {
    let server = get_mcp_config(name, path)?;
    
    let vscode_json = serde_json::json!({
        "name": server.name,
        "command": server.command,
        "args": server.args
    });

    let json_str = serde_json::to_string(&vscode_json)
        .context("Failed to serialize JSON for VSCode")?;

    let result = cmd!("code", "--add-mcp", &json_str).run();
    
    match result {
        Ok(_) => {
            println!("Successfully installed MCP server '{}' to VSCode", server.name);
            println!("VSCode command: code --add-mcp '{}'", json_str);
        },
        Err(e) => {
            eprintln!("Failed to install MCP server to VSCode: {}", e);
            eprintln!("Manual command: code --add-mcp '{}'", json_str);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn claude_code_install_mcp(name: &str, path: &str) -> Result<()> {
    let server = get_mcp_config(name, path)?;
    
    // Claude Code uses claude-code config add-mcp command
    let claude_json = serde_json::json!({
        "name": server.name,
        "command": server.command,
        "args": server.args
    });

    let json_str = serde_json::to_string(&claude_json)
        .context("Failed to serialize JSON for Claude Code")?;

    let result = cmd!("claude-code", "config", "add-mcp", &json_str).run();
    
    match result {
        Ok(_) => {
            println!("Successfully installed MCP server '{}' to Claude Code", server.name);
            println!("Claude Code command: claude-code config add-mcp '{}'", json_str);
        },
        Err(e) => {
            eprintln!("Failed to install MCP server to Claude Code: {}", e);
            eprintln!("Manual command: claude-code config add-mcp '{}'", json_str);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
