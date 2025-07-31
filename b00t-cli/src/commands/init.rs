use anyhow::Result;
use clap::Parser;
use duct::cmd;
use std::process::{Command, Stdio};
use crate::session_memory::SessionMemory;
use crate::whoami;
use crate::{get_mcp_toml_files, get_mcp_config};

#[derive(Parser)]
pub enum InitCommands {
    #[clap(
        about = "Initialize command aliases",
        long_about = "Initialize command aliases for CLI tools.\n\nExamples:\n  b00t-cli init aliases"
    )]
    Aliases,
    #[clap(
        about = "👋 Initialize hello world protocol - wake up all systems",
        long_about = "Execute the b00t hello_world protocol:\n  • Verify and start Redis server\n  • Load MCP server configurations\n  • Run system diagnostics\n  • Configure agent preferences\n  • Display interactive system tour\n\nExamples:\n  b00t-cli init hello-world\n  b00t-cli init hello-world --skip-redis"
    )]
    HelloWorld {
        #[clap(long, help = "Skip Redis server startup")]
        skip_redis: bool,
        #[clap(long, help = "Skip system diagnostics")]
        skip_diagnostics: bool,
        #[clap(long, help = "Skip interactive tour")]
        skip_tour: bool,
    },
}

/// Execute the comprehensive b00t hello_world protocol
fn execute_hello_world_protocol(path: &str, skip_redis: bool, skip_diagnostics: bool, skip_tour: bool) -> Result<()> {
    println!("👋 Initializing b00t hello_world protocol...\n");
    
    // Initialize session memory and agent identity
    let mut memory = SessionMemory::load()?;
    let agent = whoami::detect_agent(false);
    
    if !agent.is_empty() {
        memory.set("detected_agent", &agent)?;
        println!("🤖 Agent detected: {}", agent);
    } else {
        println!("🤖 No specific agent detected, running in generic mode");
    }

    // Phase 1: Redis Server Startup and Verification
    if !skip_redis {
        println!("\n🔧 Phase 1: Redis Server Management");
        verify_and_start_redis(&mut memory)?;
    } else {
        println!("\n⏭️  Skipping Redis server startup");
    }

    // Phase 2: MCP Server Introspection
    println!("\n🔌 Phase 2: MCP Server Configuration");
    introspect_mcp_servers(path, &mut memory)?;

    // Phase 3: System Diagnostics
    if !skip_diagnostics {
        println!("\n⚕️  Phase 3: System Diagnostics");
        run_system_diagnostics(&mut memory)?;
    } else {
        println!("\n⏭️  Skipping system diagnostics");
    }

    // Phase 4: System Preferences Configuration
    println!("\n⚙️  Phase 4: System Preferences");
    configure_system_preferences(&mut memory)?;

    // Phase 5: Interactive Documentation Tour
    if !skip_tour {
        println!("\n📚 Phase 5: Interactive Documentation Tour");
        interactive_documentation_tour(path, &mut memory)?;
    } else {
        println!("\n⏭️  Skipping interactive tour");
    }

    // Final status report
    memory.incr("hello_world_completions")?;
    println!("\n✅ b00t hello_world protocol completed successfully!");
    println!("📊 Session summary: {}", memory.get_summary());
    
    Ok(())
}

/// Verify Redis server is running and start if needed
fn verify_and_start_redis(memory: &mut SessionMemory) -> Result<()> {
    println!("  🔍 Checking Redis server status...");
    
    // Check if Redis is already running
    let redis_running = Command::new("redis-cli")
        .args(&["ping"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    
    if redis_running {
        println!("  ✅ Redis server is already running");
        memory.set_flag("redis_running", true)?;
        return Ok(());
    }
    
    println!("  🚀 Starting Redis server...");
    
    // Try to start Redis using different methods
    let redis_started = try_start_redis_server()?;
    
    if redis_started {
        // Wait a moment for Redis to fully start
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        // Verify it's running
        let ping_result = Command::new("redis-cli")
            .args(&["ping"])
            .output();
            
        match ping_result {
            Ok(output) if output.status.success() => {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.trim() == "PONG" {
                    println!("  ✅ Redis server started and verified (PONG received)");
                    memory.set_flag("redis_running", true)?;
                    memory.incr("redis_start_count")?;
                } else {
                    println!("  ⚠️  Redis server started but ping response unexpected: {}", response.trim());
                    memory.set_flag("redis_running", false)?;
                }
            }
            _ => {
                println!("  ❌ Redis server may have started but ping verification failed");
                memory.set_flag("redis_running", false)?;
            }
        }
    } else {
        println!("  ❌ Failed to start Redis server");
        memory.set_flag("redis_running", false)?;
        memory.incr("redis_start_failures")?;
    }
    
    Ok(())
}

/// Try different methods to start Redis server
fn try_start_redis_server() -> Result<bool> {
    // Method 1: systemctl (systemd)
    if let Ok(status) = Command::new("systemctl")
        .args(&["start", "redis-server"])
        .status() 
    {
        if status.success() {
            println!("  📦 Started Redis via systemctl");
            return Ok(true);
        }
    }
    
    // Method 2: service command
    if let Ok(status) = Command::new("service")
        .args(&["redis-server", "start"])
        .status()
    {
        if status.success() {
            println!("  📦 Started Redis via service command");
            return Ok(true);
        }
    }
    
    // Method 3: direct redis-server command
    if let Ok(_child) = Command::new("redis-server")
        .args(&["--daemonize", "yes"])
        .spawn()
    {
        println!("  📦 Started Redis server directly");
        return Ok(true);
    }
    
    // Method 4: Docker fallback
    if let Ok(status) = Command::new("docker")
        .args(&["run", "-d", "--name", "b00t-redis", "-p", "6379:6379", "redis:alpine"])
        .status()
    {
        if status.success() {
            println!("  🐳 Started Redis via Docker");
            return Ok(true);
        }
    }
    
    println!("  ⚠️  All Redis startup methods failed");
    Ok(false)
}

/// Simple MCP server info for hello_world protocol
#[derive(Debug)]
struct McpServerInfo {
    name: String,
    command: String,
    hint: String,
}

/// Get MCP server configurations for introspection
fn get_mcp_servers_info(path: &str) -> Result<Vec<McpServerInfo>> {
    let mcp_files = get_mcp_toml_files(path)?;
    let mut servers = Vec::new();
    
    for server_name in mcp_files {
        if let Ok(datum) = get_mcp_config(&server_name, path) {
            servers.push(McpServerInfo {
                name: server_name,  // server_name is already String
                command: datum.command.unwrap_or_else(|| "unknown".to_string()),
                hint: datum.hint,
            });
        }
    }
    
    Ok(servers)
}

/// Introspect MCP server configurations and report status
fn introspect_mcp_servers(path: &str, memory: &mut SessionMemory) -> Result<()> {
    println!("  🔍 Scanning MCP server configurations...");
    
    let mcp_servers = get_mcp_servers_info(path)?;
    
    if mcp_servers.is_empty() {
        println!("  ℹ️  No MCP servers configured");
        memory.set_num("mcp_server_count", 0)?;
        return Ok(());
    }
    
    println!("  📋 Found {} MCP server(s):", mcp_servers.len());
    memory.set_num("mcp_server_count", mcp_servers.len() as i64)?;
    
    for server in &mcp_servers {
        println!("    • {} - {}", server.name, server.hint);
        
        // Test if the command is available
        let command_available = Command::new("which")
            .arg(&server.command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
            
        if command_available {
            println!("      ✅ Command '{}' is available", server.command);
        } else {
            println!("      ❌ Command '{}' not found", server.command);
            memory.incr("mcp_missing_commands")?;
        }
    }
    
    // Show sample b00t syntax for MCP usage
    println!("  💡 b00t MCP syntax examples:");
    if let Some(first_server) = mcp_servers.first() {
        println!("    b00t app claude-code mcp install {}", first_server.name);
        println!("    b00t app vscode mcp install {}", first_server.name);
    }
    
    Ok(())
}

/// Run system diagnostics and health checks
fn run_system_diagnostics(memory: &mut SessionMemory) -> Result<()> {
    println!("  🩺 Running system health checks...");
    
    let mut diagnostic_results = Vec::new();
    
    // Check Git status
    if let Ok(output) = cmd!("git", "--version").read() {
        println!("  ✅ Git: {}", output.trim());
        diagnostic_results.push(("git", true));
    } else {
        println!("  ❌ Git: Not available");
        diagnostic_results.push(("git", false));
    }
    
    // Check Rust/Cargo
    if let Ok(output) = cmd!("cargo", "--version").read() {
        println!("  ✅ Cargo: {}", output.trim());
        diagnostic_results.push(("cargo", true));
    } else {
        println!("  ❌ Cargo: Not available");
        diagnostic_results.push(("cargo", false));
    }
    
    // Check Node.js
    if let Ok(output) = cmd!("node", "--version").read() {
        println!("  ✅ Node.js: {}", output.trim());
        diagnostic_results.push(("node", true));
    } else {
        println!("  ❌ Node.js: Not available");
        diagnostic_results.push(("node", false));
    }
    
    // Check Docker
    if let Ok(output) = cmd!("docker", "--version").read() {
        println!("  ✅ Docker: {}", output.lines().next().unwrap_or("").trim());
        diagnostic_results.push(("docker", true));
    } else {
        println!("  ❌ Docker: Not available");
        diagnostic_results.push(("docker", false));
    }
    
    // Check available package managers
    let package_managers = vec!["npm", "pnpm", "yarn", "bun"];
    for pm in package_managers {
        if cmd!(pm, "--version").read().is_ok() {
            println!("  ✅ Package Manager: {} available", pm);
            diagnostic_results.push((pm, true));
        }
    }
    
    // Store diagnostic results in session memory
    let passing_count = diagnostic_results.iter().filter(|(_, passed)| *passed).count();
    memory.set_num("diagnostic_passing", passing_count as i64)?;
    memory.set_num("diagnostic_total", diagnostic_results.len() as i64)?;
    
    println!("  📊 Diagnostics: {}/{} systems operational", passing_count, diagnostic_results.len());
    
    Ok(())
}

/// Configure system preferences in session memory
fn configure_system_preferences(memory: &mut SessionMemory) -> Result<()> {
    println!("  ⚙️  Configuring system preferences...");
    
    // Detect preferred container runtime
    let container_runtime = if cmd!("podman", "--version").read().is_ok() {
        "podman"
    } else if cmd!("docker", "--version").read().is_ok() {
        "docker"  
    } else {
        "none"
    };
    
    memory.set("preferred_container_runtime", container_runtime)?;
    println!("  🐳 Container runtime: {}", container_runtime);
    
    // Detect Kubernetes environment
    let k8s_env = if cmd!("minikube", "status").read().is_ok() {
        "minikube"
    } else if cmd!("kubectl", "cluster-info").read().is_ok() {
        "kubectl"
    } else {
        "none"
    };
    
    memory.set("k8s_environment", k8s_env)?;
    println!("  ☸️  Kubernetes: {}", k8s_env);
    
    // Store initialization timestamp
    memory.set("last_hello_world", &chrono::Utc::now().to_rfc3339())?;
    
    println!("  ✅ System preferences configured");
    
    Ok(())
}

/// Interactive documentation tour
fn interactive_documentation_tour(_path: &str, memory: &mut SessionMemory) -> Result<()> {
    println!("  📖 Starting interactive documentation tour...");
    
    // Check for README.md
    let git_root = crate::utils::get_workspace_root();
    let readme_path = std::path::PathBuf::from(&git_root).join("README.md");
    
    if readme_path.exists() {
        if !memory.is_readme_read() {
            println!("  📋 README.md found - marking as available for reading");
            println!("  💡 Remember to run 'b00t-cli session mark-readme-read' after reading");
        } else {
            println!("  ✅ README.md already marked as read this session");
        }
    } else {
        println!("  ℹ️  No README.md found in git root");
    }
    
    // Show sample b00t commands for different scenarios
    println!("  🎓 b00t Command Reference:");
    println!("    # Session management");
    println!("    b00t-cli session get <key>           # Get session value");
    println!("    b00t-cli session incr <key>          # Increment counter");
    println!("    b00t-cli session keys                # List all keys");
    println!("");
    println!("    # MCP server management");
    println!("    b00t-cli mcp list                    # List MCP servers");
    println!("    b00t-cli app claude-code mcp install <server>  # Install to Claude Code");
    println!("");
    println!("    # System status");
    println!("    b00t-cli status                      # Show all tools status");
    println!("    b00t-cli checkpoint                  # Create git checkpoint");
    
    memory.set_flag("tour_completed", true)?;
    println!("  ✅ Interactive tour completed");
    
    Ok(())
}

impl InitCommands {
    pub fn execute(&self, path: &str) -> Result<()> {
        match self {
            InitCommands::Aliases => {
                println!("🔗 Init aliases functionality coming soon...");
                Ok(())
            }
            InitCommands::HelloWorld { skip_redis, skip_diagnostics, skip_tour } => {
                execute_hello_world_protocol(path, *skip_redis, *skip_diagnostics, *skip_tour)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_commands_exist() {
        let aliases_cmd = InitCommands::Aliases;
        assert!(aliases_cmd.execute("test").is_ok());
    }
}