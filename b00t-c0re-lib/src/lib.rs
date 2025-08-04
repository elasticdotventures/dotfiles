//! # b00t-c0re-lib
//! 
//! Core shared library for b00t ecosystem providing:
//! - Template rendering with b00t context variables
//! - Common data structures and utilities
//! - Shared functionality between b00t-cli and b00t-mcp

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod template;
pub mod context;
pub mod utils;

// Re-export commonly used types
pub use template::TemplateRenderer;
pub use context::B00tContext;

/// Common configuration structure for b00t components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct B00tConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

/// Result type alias for b00t operations
pub type B00tResult<T> = Result<T, anyhow::Error>;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_available() {
        assert!(!VERSION.is_empty());
    }
}