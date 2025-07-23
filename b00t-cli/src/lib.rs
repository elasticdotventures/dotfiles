use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct McpConfig {
    pub mcp: McpServer,
}

pub fn clean_json_for_dwiw(input: &str) -> String {
    let comment_regex = Regex::new(r"^\s*//.*$").unwrap();
    
    input
        .lines()
        .filter(|line| !comment_regex.is_match(line))
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn normalize_mcp_json(input: &str, dwiw: bool) -> Result<McpServer> {
    let cleaned_input = if dwiw {
        clean_json_for_dwiw(input)
    } else {
        input.to_string()
    };

    let json_value: serde_json::Value = serde_json::from_str(&cleaned_input)?;

    if let Some(name) = json_value.get("name") {
        let server = McpServer {
            name: name.as_str().unwrap_or("unknown").to_string(),
            command: json_value.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("npx")
                .to_string(),
            args: json_value.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(|| vec![]),
        };
        return Ok(server);
    }

    let keys: Vec<_> = json_value.as_object()
        .map(|obj| obj.keys().collect())
        .unwrap_or_default();
    
    if keys.len() == 1 {
        let server_name = &keys[0];
        let server_config = &json_value[server_name];
        
        let server = McpServer {
            name: server_name.to_string(),
            command: server_config.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("npx")
                .to_string(),
            args: server_config.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(|| vec![]),
        };
        return Ok(server);
    }

    anyhow::bail!("Unable to parse MCP server configuration from JSON");
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
        assert_eq!(result.command, "npx");
        assert_eq!(result.args, vec!["-y", "@modelcontextprotocol/server-filesystem"]);
    }

    #[test]
    fn test_normalize_mcp_json_nested_format() {
        let input = r#"{"playwright": {"command": "npx", "args": ["-y", "@executeautomation/playwright-mcp-server"]}}"#;
        
        let result = normalize_mcp_json(input, false).unwrap();
        
        assert_eq!(result.name, "playwright");
        assert_eq!(result.command, "npx");
        assert_eq!(result.args, vec!["-y", "@executeautomation/playwright-mcp-server"]);
    }

    #[test]
    fn test_normalize_mcp_json_with_dwiw() {
        let input = r#"// Comment here
{
  "github": {
    // Another comment
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"]
  }
}"#;
        
        let result = normalize_mcp_json(input, true).unwrap();
        
        assert_eq!(result.name, "github");
        assert_eq!(result.command, "npx");
        assert_eq!(result.args, vec!["-y", "@modelcontextprotocol/server-github"]);
    }

    #[test]
    fn test_normalize_mcp_json_defaults() {
        let input = r#"{"test": {}}"#;
        
        let result = normalize_mcp_json(input, false).unwrap();
        
        assert_eq!(result.name, "test");
        assert_eq!(result.command, "npx");
        assert_eq!(result.args, Vec::<String>::new());
    }

    #[test]
    fn test_normalize_mcp_json_invalid() {
        let input = r#"{"multiple": {}, "keys": {}}"#;
        
        let result = normalize_mcp_json(input, false);
        
        assert!(result.is_err());
    }
}