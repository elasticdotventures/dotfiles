use crate::traits::*;
use crate::{BootDatum, check_command_available, get_config};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// MCP-specific multi-method structures
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct McpCliMethod {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct McpHttpMethod {
    pub url: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default = "default_true")]
    pub requires_internet: bool,
    #[serde(default)]
    pub requires_auth: bool,
}

fn default_true() -> bool {
    true
}

pub struct McpDatum {
    pub datum: BootDatum,
}

impl McpDatum {
    pub fn from_config(name: &str, path: &str) -> Result<Self> {
        let (config, _filename) = get_config(name, path).map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(McpDatum { datum: config.b00t })
    }

    // Helper to parse CLI methods from raw data
    fn parse_cli_methods(&self) -> Vec<McpCliMethod> {
        if let Some(cli_data) = &self.datum.mcp_cli {
            cli_data.iter()
                .filter_map(|raw| serde_json::from_value(serde_json::Value::Object(
                    raw.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                )).ok())
                .collect()
        } else {
            Vec::new()
        }
    }

    // Helper to parse HTTP method from raw data  
    fn parse_http_method(&self) -> Option<McpHttpMethod> {
        if let Some(http_data) = &self.datum.mcp_http {
            serde_json::from_value(serde_json::Value::Object(
                http_data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            )).ok()
        } else {
            None
        }
    }

    /// Select the best available method based on priority and availability
    pub fn select_best_method(&self) -> Option<McpSelectedMethod> {
        let mut candidates = Vec::new();

        // Add HTTP method if available
        if let Some(http) = self.parse_http_method() {
            if self.check_http_requirements(&http) {
                candidates.push(McpSelectedMethod::Http(http));
            }
        }

        // Add CLI methods if available
        let cli_methods = self.parse_cli_methods();
        for cli in cli_methods {
            if self.check_cli_requirements(&cli) {
                candidates.push(McpSelectedMethod::Cli(cli));
            }
        }

        // Fall back to legacy command/args if no multi-methods available
        if candidates.is_empty() {
            if let Some(command) = &self.datum.command {
                if check_command_available(command) {
                    let legacy_cli = McpCliMethod {
                        command: command.clone(),
                        args: self.datum.args.clone().unwrap_or_default(),
                        priority: 255, // Lowest priority
                        requires: self.datum.require.clone().unwrap_or_default(),
                        env: self.datum.env.clone().unwrap_or_default(),
                    };
                    return Some(McpSelectedMethod::Cli(legacy_cli));
                }
            }
            return None;
        }

        // Sort by priority (lower number = higher priority)
        candidates.sort_by_key(|method| method.priority());
        candidates.into_iter().next()
    }

    fn check_http_requirements(&self, http: &McpHttpMethod) -> bool {
        // Check if internet is required and available
        if http.requires_internet {
            // TODO: Add actual internet connectivity check
            // For now, assume internet is available
        }
        
        // Check other requirements
        self.evaluate_method_constraints(&http.requires)
    }

    fn check_cli_requirements(&self, cli: &McpCliMethod) -> bool {
        // Check if command is available
        if !check_command_available(&cli.command) {
            return false;
        }
        
        // Check method-specific constraints
        self.evaluate_method_constraints(&cli.requires)
    }

    fn evaluate_method_constraints(&self, requires: &[String]) -> bool {
        if requires.is_empty() {
            return true;
        }

        requires.iter().all(|constraint| {
            match constraint.as_str() {
                "node" => check_command_available("node") || check_command_available("npx"),
                "python" => check_command_available("python") || check_command_available("python3") || check_command_available("uvx"),
                "docker" => check_command_available("docker"),
                "internet" => true, // TODO: Add actual internet check
                constraint if constraint.starts_with("CMD:") => {
                    check_command_available(&constraint[4..])
                }
                _ => true, // Unknown constraints default to true
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum McpSelectedMethod {
    Http(McpHttpMethod),
    Cli(McpCliMethod),
}

impl McpSelectedMethod {
    pub fn priority(&self) -> u8 {
        match self {
            McpSelectedMethod::Http(http) => http.priority,
            McpSelectedMethod::Cli(cli) => cli.priority,
        }
    }

    pub fn command(&self) -> Option<&str> {
        match self {
            McpSelectedMethod::Http(_) => None,
            McpSelectedMethod::Cli(cli) => Some(&cli.command),
        }
    }

    pub fn args(&self) -> Option<&[String]> {
        match self {
            McpSelectedMethod::Http(_) => None,
            McpSelectedMethod::Cli(cli) => Some(&cli.args),
        }
    }

    pub fn is_http(&self) -> bool {
        matches!(self, McpSelectedMethod::Http(_))
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            McpSelectedMethod::Http(http) => Some(&http.url),
            McpSelectedMethod::Cli(_) => None,
        }
    }
}

impl TryFrom<(&str, &str)> for McpDatum {
    type Error = anyhow::Error;

    fn try_from((name, path): (&str, &str)) -> Result<Self, Self::Error> {
        Self::from_config(name, path)
    }
}

impl DatumChecker for McpDatum {
    fn is_installed(&self) -> bool {
        // Check if any method is available
        self.select_best_method().is_some()
    }

    fn current_version(&self) -> Option<String> {
        // MCP servers don't typically have semantic versions
        // Return the selected method info if available
        if let Some(method) = self.select_best_method() {
            match method {
                McpSelectedMethod::Http(http) => Some(format!("HTTP: {}", http.url)),
                McpSelectedMethod::Cli(cli) => Some(format!("{} available", cli.command)),
            }
        } else {
            None
        }
    }

    fn desired_version(&self) -> Option<String> {
        self.datum.desires.clone()
    }

    fn version_status(&self) -> VersionStatus {
        if DatumChecker::is_installed(self) {
            VersionStatus::Unknown // MCP servers are just available/not available
        } else {
            VersionStatus::Missing
        }
    }
}

impl StatusProvider for McpDatum {
    fn name(&self) -> &str {
        &self.datum.name
    }

    fn subsystem(&self) -> &str {
        "mcp"
    }

    fn hint(&self) -> &str {
        &self.datum.hint
    }

    fn is_disabled(&self) -> bool {
        false // MCP servers are never disabled by default
    }
}

impl FilterLogic for McpDatum {
    fn is_available(&self) -> bool {
        !DatumChecker::is_installed(self) && self.prerequisites_satisfied()
    }

    fn prerequisites_satisfied(&self) -> bool {
        // Check if require constraints are satisfied
        if let Some(require) = &self.datum.require {
            self.evaluate_constraints(require)
        } else {
            // Default: check if all ENV vars are satisfied
            self.evaluate_constraints(&[])
        }
    }

    fn evaluate_constraints(&self, require: &[String]) -> bool {
        self.evaluate_constraints_default(require)
    }
}

impl ConstraintEvaluator for McpDatum {
    fn datum(&self) -> &BootDatum {
        &self.datum
    }
}

impl DatumProvider for McpDatum {
    fn datum(&self) -> &BootDatum {
        &self.datum
    }
}
