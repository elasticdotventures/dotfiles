use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::{Result, Context};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub hint: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct McpConfig {
    pub mcp: McpServer,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct UnifiedConfig {
    pub b00t: BootDatum,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BootDatum {
    pub name: String,
    #[serde(rename = "type")]
    pub datum_type: Option<DatumType>,
    pub desires: Option<String>,
    pub hint: String,
    
    pub install: Option<String>,
    pub update: Option<String>,
    pub version: Option<String>,
    pub version_regex: Option<String>,
    
    // MCP server fields
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    
    // VSCode extension fields
    pub vsix_id: Option<String>,
    
    // Bash script fields
    pub script: Option<String>,
    
    // Docker fields
    pub image: Option<String>,
    pub docker_args: Option<Vec<String>>,
    
    pub package_name: Option<String>,
    
    // Environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
    
    // Require constraints
    pub require: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatumType {
    Unknown,
    Mcp,
    Bash,
    Vscode,
    Docker,
    Apt,
    Nix,
    Ai,
}

#[derive(Serialize, Debug)]
pub struct McpListOutput {
    pub servers: Vec<McpListItem>,
    pub path: String,
}

#[derive(Serialize, Debug)]
pub struct McpListItem {
    pub name: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AiConfig {
    pub b00t: BootDatum,
    pub models: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Debug)]
pub struct AiListOutput {
    pub providers: Vec<AiListItem>,
    pub path: String,
}

#[derive(Serialize, Debug)]
pub struct AiListItem {
    pub name: String,
    pub models: Option<Vec<String>>,
    pub env_keys: Option<Vec<String>>,
    pub error: Option<String>,
}

pub fn extract_comments_and_clean_json(input: &str) -> (String, Option<String>) {
    let comment_regex = Regex::new(r"^\s*//\s*(.*)$").unwrap();
    
    let mut comments = Vec::new();
    let mut cleaned_lines = Vec::new();
    
    for line in input.lines() {
        if let Some(caps) = comment_regex.captures(line) {
            let comment = caps[1].trim();
            if !comment.is_empty() {
                comments.push(comment.to_string());
            }
        } else {
            cleaned_lines.push(line);
        }
    }
    
    let cleaned_json = cleaned_lines.join("\n").trim().to_string();
    let hint = if comments.is_empty() {
        None
    } else {
        Some(comments.join(" "))
    };
    
    (cleaned_json, hint)
}

pub fn clean_json_for_dwiw(input: &str) -> String {
    extract_comments_and_clean_json(input).0
}

pub fn normalize_mcp_json(input: &str, dwiw: bool) -> Result<BootDatum> {
    let (cleaned_input, hint) = if dwiw {
        extract_comments_and_clean_json(input)
    } else {
        (input.to_string(), None)
    };

    let json_value: serde_json::Value = serde_json::from_str(&cleaned_input)?;

    // Handle direct format: {"name": "...", "command": "...", "args": [...]}
    if let Some(name) = json_value.get("name") {
        let server = BootDatum {
            name: name.as_str().unwrap_or("unknown").to_string(),
            datum_type: Some(DatumType::Mcp),
            desires: None,
            hint: hint.clone().unwrap_or_else(|| "MCP server".to_string()),
            install: None,
            update: None,
            version: None,
            version_regex: None,
            command: Some(json_value.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("npx")
                .to_string()),
            args: Some(json_value.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(|| vec![])),
            vsix_id: None,
            script: None,
            image: None,
            docker_args: None,
            package_name: None,
            env: json_value.get("env")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()),
            require: json_value.get("require")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()),
        };
        return Ok(server);
    }

    // Handle mcpServers wrapper format: {"mcpServers": {"server_name": {...}}}
    if let Some(mcp_servers) = json_value.get("mcpServers") {
        let keys: Vec<_> = mcp_servers.as_object()
            .map(|obj| obj.keys().collect())
            .unwrap_or_default();
        
        if keys.len() == 1 {
            let server_name = &keys[0];
            let server_config = &mcp_servers[server_name];
            
            let server = BootDatum {
                name: server_name.to_string(),
                datum_type: Some(DatumType::Mcp),
                desires: None,
                hint: hint.clone().unwrap_or_else(|| "MCP server".to_string()),
                install: None,
                update: None,
                version: None,
                version_regex: None,
                command: Some(server_config.get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("npx")
                    .to_string()),
                args: Some(server_config.get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_else(|| vec![])),
                vsix_id: None,
                script: None,
                image: None,
                docker_args: None,
                package_name: None,
                env: server_config.get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()),
                require: server_config.get("require")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()),
            };
            return Ok(server);
        }
    }

    // Handle single server format: {"server_name": {...}}
    let keys: Vec<_> = json_value.as_object()
        .map(|obj| obj.keys().collect())
        .unwrap_or_default();
    
    if keys.len() == 1 {
        let server_name = &keys[0];
        let server_config = &json_value[server_name];
        
        let server = BootDatum {
            name: server_name.to_string(),
            datum_type: Some(DatumType::Mcp),
            desires: None,
            hint: hint.unwrap_or_else(|| "MCP server".to_string()),
            install: None,
            update: None,
            version: None,
            version_regex: None,
            command: Some(server_config.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("npx")
                .to_string()),
            args: Some(server_config.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(|| vec![])),
            vsix_id: None,
            script: None,
            image: None,
            docker_args: None,
            package_name: None,
            env: server_config.get("env")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()),
            require: server_config.get("require")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()),
        };
        return Ok(server);
    }

    anyhow::bail!("Unable to parse MCP server configuration from JSON");
}

pub fn create_ai_toml_config(config: &AiConfig, path: &str) -> Result<()> {
    let toml_content = toml::to_string(config)
        .context("Failed to serialize AI config to TOML")?;

    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(shellexpand::tilde(path).to_string());
    path_buf.push(format!("{}.ai.toml", config.b00t.name));

    std::fs::write(&path_buf, toml_content)
        .context(format!("Failed to write AI config to {}", path_buf.display()))?;

    println!("Created AI config: {}", path_buf.display());
    Ok(())
}

pub fn create_unified_toml_config(datum: &BootDatum, path: &str) -> Result<()> {
    let config = UnifiedConfig {
        b00t: datum.clone(),
    };

    let toml_content = toml::to_string(&config)
        .context("Failed to serialize config to TOML")?;

    // Use explicit datum_type or default to Unknown
    let datum_type = datum.datum_type.clone().unwrap_or(DatumType::Unknown);
    let suffix = match datum_type {
        DatumType::Mcp => ".mcp.toml",
        DatumType::Bash => ".bash.toml",
        DatumType::Vscode => ".vscode.toml",
        DatumType::Docker => ".docker.toml",
        DatumType::Apt => ".apt.toml",
        DatumType::Nix => ".nix.toml",
        DatumType::Ai => ".ai.toml",
        DatumType::Unknown => ".toml",
    };

    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(shellexpand::tilde(path).to_string());
    path_buf.push(format!("{}{}", datum.name, suffix));

    std::fs::write(&path_buf, toml_content)
        .context(format!("Failed to write config to {}", path_buf.display()))?;

    println!("Created {} config: {}", datum_type.to_string(), path_buf.display());
    Ok(())
}

impl std::fmt::Display for DatumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatumType::Unknown => write!(f, "unknown"),
            DatumType::Mcp => write!(f, "MCP"),
            DatumType::Bash => write!(f, "bash"),
            DatumType::Vscode => write!(f, "VSCode"),
            DatumType::Docker => write!(f, "docker"),
            DatumType::Apt => write!(f, "apt"),
            DatumType::Nix => write!(f, "nix"),
            DatumType::Ai => write!(f, "AI"),
        }
    }
}

impl DatumType {
    pub fn from_filename_extension(filename: &str) -> DatumType {
        if filename.ends_with(".cli.toml") {
            DatumType::Unknown
        } else if filename.ends_with(".mcp.toml") {
            DatumType::Mcp
        } else if filename.ends_with(".bash.toml") {
            DatumType::Bash
        } else if filename.ends_with(".vscode.toml") {
            DatumType::Vscode
        } else if filename.ends_with(".docker.toml") {
            DatumType::Docker
        } else if filename.ends_with(".apt.toml") {
            DatumType::Apt
        } else if filename.ends_with(".nix.toml") {
            DatumType::Nix
        } else if filename.ends_with(".ai.toml") {
            DatumType::Ai
        } else {
            DatumType::Unknown // Default fallback for .toml files
        }
    }
}

impl BootDatum {
    pub fn get_datum_type(&self, filename: Option<&str>) -> DatumType {
        self.datum_type.clone().unwrap_or_else(|| {
            filename.map(DatumType::from_filename_extension)
                   .unwrap_or(DatumType::Unknown)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_for_dwiw() {
        let input = r#"// This is a comment
{
  "name": "test",
  // Another comment
  "command": "npx"
}"#;
        
        let expected = r#"{
  "name": "test",
  "command": "npx"
}"#;
        
        assert_eq!(clean_json_for_dwiw(input), expected);
    }

    #[test]
    fn test_normalize_mcp_json_direct_format() {
        let input = r#"{"name": "filesystem", "command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem"]}"#;
        
        let result = normalize_mcp_json(input, false).unwrap();
        
        assert_eq!(result.name, "filesystem");
        assert_eq!(result.command, Some("npx".to_string()));
        assert_eq!(result.args, Some(vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()]));
        assert_eq!(result.hint, "MCP server");
    }

    #[test]
    fn test_normalize_mcp_json_nested_format() {
        let input = r#"{"playwright": {"command": "npx", "args": ["-y", "@executeautomation/playwright-mcp-server"]}}"#;
        
        let result = normalize_mcp_json(input, false).unwrap();
        
        assert_eq!(result.name, "playwright");
        assert_eq!(result.command, Some("npx".to_string()));
        assert_eq!(result.args, Some(vec!["-y".to_string(), "@executeautomation/playwright-mcp-server".to_string()]));
        assert_eq!(result.hint, "MCP server");
    }

    #[test]
    fn test_normalize_mcp_json_with_dwiw() {
        let input = r#"// GitHub MCP server
{
  "github": {
    // Provides GitHub API access
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"]
  }
}"#;
        
        let result = normalize_mcp_json(input, true).unwrap();
        
        assert_eq!(result.name, "github");
        assert_eq!(result.command, Some("npx".to_string()));
        assert_eq!(result.args, Some(vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()]));
        assert_eq!(result.hint, "GitHub MCP server Provides GitHub API access");
    }

    #[test]
    fn test_normalize_mcp_json_defaults() {
        let input = r#"{"test": {}}"#;
        
        let result = normalize_mcp_json(input, false).unwrap();
        
        assert_eq!(result.name, "test");
        assert_eq!(result.command, Some("npx".to_string()));
        assert_eq!(result.args, Some(Vec::<String>::new()));
        assert_eq!(result.hint, "MCP server");
    }

    #[test]
    fn test_normalize_mcp_json_mcpservers_format() {
        let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join("mcp")
            .join("mcpservers-format.json");
        let input = std::fs::read_to_string(sample_path).unwrap();
        
        let result = normalize_mcp_json(&input, false).unwrap();
        
        assert_eq!(result.name, "lsp");
        assert_eq!(result.command, Some("docker".to_string()));
        assert_eq!(result.args, Some(vec!["run".to_string(), "-i".to_string(), "--rm".to_string(), "docker.io/jonrad/lsp-mcp:0.3.1".to_string()]));
        assert_eq!(result.hint, "MCP server");
    }

    #[test]
    fn test_normalize_mcp_json_invalid() {
        let input = r#"{"multiple": {}, "keys": {}}"#;
        
        let result = normalize_mcp_json(input, false);
        
        assert!(result.is_err());
    }
}