//! Configuration file handling for persistent preferences

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub scan: ScanConfig,
    pub display: DisplayConfig,
    pub behavior: BehaviorConfig,
}

/// Scan-related configuration
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ScanConfig {
    /// Default paths to scan (empty = home directory)
    pub default_paths: Vec<PathBuf>,
    /// Patterns to exclude from scanning
    pub exclude_patterns: Vec<String>,
    /// Include global ~/.claude folder
    pub include_global: bool,
}

/// Display-related configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Show project type column
    pub show_project_type: bool,
    /// Show filter bar by default
    pub show_filter_bar: bool,
    /// Default sort order: size_desc, size_asc, name_asc, name_desc, date_desc, date_asc
    pub default_sort: String,
}

/// Behavior-related configuration
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Use permanent delete instead of trash
    pub permanent_delete: bool,
    /// Show confirmation dialog before delete
    pub confirm_delete: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_project_type: true,
            show_filter_bar: false,
            default_sort: "size_desc".to_string(),
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            permanent_delete: false,
            confirm_delete: true,
        }
    }
}

impl Config {
    /// Load configuration from file, using defaults if not found
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))
    }

    /// Get the configuration file path
    pub fn config_path() -> PathBuf {
        ProjectDirs::from("", "", "claudekill")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join(".config/claudekill/config.toml")
            })
    }

    /// Create default config file if it doesn't exist
    pub fn create_default_if_missing() -> Result<bool> {
        let path = Self::config_path();

        if path.exists() {
            return Ok(false);
        }

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = Self::default_config_content(&path);
        fs::write(&path, content)?;
        Ok(true)
    }

    /// Generate default config file content with comments
    fn default_config_content(path: &Path) -> String {
        format!(
            r#"# ClaudeKill Configuration
# Location: {}
# Documentation: https://github.com/olbboy/claudekill#configuration

[scan]
# Default paths to scan (empty = home directory)
# default_paths = ["~/Projects", "~/Work"]

# Patterns to exclude from scanning
# exclude_patterns = ["node_modules", ".git"]

# Include global ~/.claude folder in scan
include_global = false

[display]
# Show project type column
show_project_type = true

# Show filter bar by default
show_filter_bar = false

# Default sort: "size_desc", "size_asc", "name_asc", "name_desc", "date_desc", "date_asc"
default_sort = "size_desc"

[behavior]
# Use permanent delete instead of moving to trash
permanent_delete = false

# Show confirmation dialog before deleting
confirm_delete = true
"#,
            path.display()
        )
    }

    /// Parse sort order string to SortOrder enum
    pub fn parse_sort_order(&self) -> crate::filter::SortOrder {
        match self.display.default_sort.as_str() {
            "size_asc" => crate::filter::SortOrder::SizeAsc,
            "name_asc" => crate::filter::SortOrder::NameAsc,
            "name_desc" => crate::filter::SortOrder::NameDesc,
            "date_desc" => crate::filter::SortOrder::DateDesc,
            "date_asc" => crate::filter::SortOrder::DateAsc,
            _ => crate::filter::SortOrder::SizeDesc, // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.scan.include_global);
        assert!(!config.behavior.permanent_delete);
        assert!(config.behavior.confirm_delete);
        assert!(config.display.show_project_type);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
            [scan]
            include_global = true
            exclude_patterns = ["node_modules", "target"]

            [behavior]
            permanent_delete = true
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.scan.include_global);
        assert!(config.behavior.permanent_delete);
        assert_eq!(config.scan.exclude_patterns.len(), 2);
    }

    #[test]
    fn test_parse_sort_order() {
        let mut config = Config::default();

        config.display.default_sort = "size_asc".to_string();
        assert_eq!(config.parse_sort_order(), crate::filter::SortOrder::SizeAsc);

        config.display.default_sort = "name_asc".to_string();
        assert_eq!(config.parse_sort_order(), crate::filter::SortOrder::NameAsc);

        config.display.default_sort = "invalid".to_string();
        assert_eq!(
            config.parse_sort_order(),
            crate::filter::SortOrder::SizeDesc
        );
    }

    #[test]
    fn test_config_path_not_empty() {
        let path = Config::config_path();
        assert!(!path.as_os_str().is_empty());
    }
}
