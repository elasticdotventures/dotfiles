use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub enum WhatismyCommands {
    #[clap(about = "Detect current AI agent (claude, gemini, etc.)")]
    Agent {
        #[clap(long, help = "Ignore _B00T_Agent environment variable")]
        no_env: bool,
        #[clap(long, help = "Output in JSON format")]
        json: bool,
    },
    #[clap(about = "Detect current session information")]
    Session {
        #[clap(long, help = "Output in JSON format")]
        json: bool,
    },
    #[clap(about = "Detect current environment setup")]
    Environment {
        #[clap(long, help = "Output in JSON format")]
        json: bool,
    },
    #[clap(about = "Show session-aware system status with OODA context")]
    Status {
        #[clap(long, help = "Output in JSON format")]
        json: bool,
    },
    #[clap(about = "Export template in specified format")]
    Template {
        #[clap(help = "Template name (e.g., 'status')")]
        name: String,
        #[clap(help = "Output format: toml, yaml, json, or tera")]
        format: String,
    },
}

impl WhatismyCommands {
    pub fn execute(&self, _path: &str) -> Result<()> {
        match self {
            WhatismyCommands::Agent { no_env, json } => {
                use crate::session_memory::SessionMemory;
                let memory = SessionMemory::load()?;
                let agent = detect_agent(&memory, *no_env);
                
                if *json {
                    println!("{}", serde_json::to_string(&serde_json::json!({
                        "agent": agent,
                        "pid": std::process::id(),
                        "ppid": get_parent_pid(),
                    }))?);
                } else {
                    println!("{}", agent);
                }
                Ok(())
            }
            WhatismyCommands::Session { json } => {
                use crate::session_memory::SessionMemory;
                let memory = SessionMemory::load()?;
                
                if *json {
                    println!("{}", serde_json::to_string(&serde_json::json!({
                        "session_id": memory.metadata.session_id,
                        "pid": std::process::id(),
                        "created_at": memory.metadata.created_at,
                        "updated_at": memory.metadata.updated_at,
                        "branch": memory.metadata.initial_branch,
                    }))?);
                } else {
                    println!("{}", memory.get_summary());
                }
                Ok(())
            }
            WhatismyCommands::Environment { json } => {
                use crate::session_memory::SessionMemory;
                let memory = SessionMemory::load()?;
                let env_info = memory.collect_tracked_env();
                
                if *json {
                    println!("{}", serde_json::to_string(&env_info)?);
                } else {
                    println!("🌍 Environment: {:?}", env_info);
                }
                Ok(())
            }
            WhatismyCommands::Status { json } => {
                use crate::session_memory::SessionMemory;
                let mut memory = SessionMemory::load()?;
                
                if *json {
                    let context = memory.get_agent_context();
                    println!("{}", serde_json::to_string_pretty(&context)?);
                } else {
                    // Try template rendering first, fall back to detailed diagnostics
                    match memory.render_status_template() {
                        Ok(rendered) => println!("{}", rendered),
                        Err(_) => {
                            // Fallback: Run the enhanced diagnostics 
                            crate::commands::init::run_system_diagnostics(&mut memory)?;
                        }
                    }
                }
                Ok(())
            }
            WhatismyCommands::Template { name, format } => {
                use crate::session_memory::SessionMemory;
                let memory = SessionMemory::load()?;
                
                match name.as_str() {
                    "status" => {
                        match format.as_str() {
                            "tera" => {
                                // Output the raw Tera template
                                let template_content = memory.load_default_status_template().unwrap_or_else(|_| 
                                    "Error loading template".to_string());
                                println!("{}", template_content);
                            }
                            "toml" => {
                                // Output template config in TOML format
                                let context = memory.get_agent_context();
                                println!("[template.status]");
                                println!("# Agent context for template rendering");
                                println!("agent_name = \"{}\"", context.agent_name);
                                println!("session_id = \"{}\"", context.session_id);
                                println!("session_duration = {}", context.session_duration);
                                println!("current_branch = \"{}\"", context.current_branch);
                                println!("shell_count = {}", context.shell_count);
                                println!("build_count = {}", context.build_count);
                                println!("compile_count = {}", context.compile_count);
                                println!("test_count = {}", context.test_count);
                            }
                            "yaml" => {
                                // Output template config in YAML format
                                let context = memory.get_agent_context();
                                println!("template:");
                                println!("  status:");
                                println!("    # Agent context for template rendering");
                                println!("    agent_name: \"{}\"", context.agent_name);
                                println!("    session_id: \"{}\"", context.session_id);
                                println!("    session_duration: {}", context.session_duration);
                                println!("    current_branch: \"{}\"", context.current_branch);
                                println!("    shell_count: {}", context.shell_count);
                                println!("    build_count: {}", context.build_count);
                                println!("    compile_count: {}", context.compile_count);
                                println!("    test_count: {}", context.test_count);
                            }
                            "json" => {
                                // Output template config in JSON format
                                let context = memory.get_agent_context();
                                let template_data = serde_json::json!({
                                    "template": {
                                        "status": {
                                            "agent_name": context.agent_name,
                                            "session_id": context.session_id,
                                            "session_duration": context.session_duration,
                                            "current_branch": context.current_branch,
                                            "shell_count": context.shell_count,
                                            "build_count": context.build_count,
                                            "compile_count": context.compile_count,
                                            "test_count": context.test_count
                                        }
                                    }
                                });
                                println!("{}", serde_json::to_string_pretty(&template_data)?);
                            }
                            _ => {
                                return Err(anyhow::anyhow!("Unsupported format: {}. Use: toml, yaml, json, or tera", format));
                            }
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Unknown template: {}. Available: status", name));
                    }
                }
                Ok(())
            }
        }
    }
}

use crate::session_memory::SessionMemory;

pub fn detect_agent(memory: &SessionMemory, no_env: bool) -> String {
    // Check environment variable first (unless no_env flag is set)
    if !no_env {
        if let Some(agent) = memory.get_env_var("_B00T_Agent") {
            if !agent.is_empty() {
                return agent;
            }
        }
    }
    
    // Detect based on parent process and environment
    let pid = std::process::id();
    let ppid = get_parent_pid();
    
    // Check for Claude Code
    if memory.get_env_var("CLAUDECODE").as_deref() == Some("1") {
        return format!("🤖 Claude Code PID:{}", pid);
    }
    
    // Check for other AI environments using tracked variables
    if memory.get_env_var("ANTHROPIC_API_KEY").is_some() || 
       memory.get_env_var("CLAUDE_API_KEY").is_some() {
        return format!("🤖 Claude Agent PID:{}", pid);
    }
    
    // Check parent process name against configured patterns
    if let Some(parent_cmd) = get_parent_command() {
        for (pattern, display) in &memory.config.agent_patterns {
            if parent_cmd.contains(pattern) {
                return format!("{} PID:{}", display, pid);
            }
        }
    }
    
    // Default: not an agent
    format!("🧑‍💻 Human PID:{} PPID:{}", pid, ppid.unwrap_or(0))
}

fn get_parent_pid() -> Option<u32> {
    #[cfg(unix)]
    {
        unsafe {
            let ppid = libc::getppid();
            if ppid > 0 {
                Some(ppid as u32)
            } else {
                None
            }
        }
    }
    #[cfg(not(unix))]
    {
        None
    }
}

fn get_parent_command() -> Option<String> {
    if let Some(ppid) = get_parent_pid() {
        // Try to read the parent command from /proc on Linux
        #[cfg(target_os = "linux")]
        {
            if let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/comm", ppid)) {
                return Some(cmdline.trim().to_string());
            }
        }
        
        // Fallback: use ps command
        if let Ok(output) = duct::cmd!("ps", "-o", "comm=", "-p", ppid.to_string()).read() {
            return Some(output.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatismy_commands_exist() {
        let agent_cmd = WhatismyCommands::Agent {
            no_env: false,
            json: false,
        };
        
        assert!(agent_cmd.execute("test").is_ok());
    }
}