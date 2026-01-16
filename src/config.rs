use ariadne::CharSet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to parse TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub title_only: bool,

    #[serde(default)]
    pub commits_only: bool,

    #[serde(default)]
    pub title_and_commits: bool,

    #[serde(default)]
    pub any_commit: bool,

    #[serde(default = "default_types")]
    pub types: Vec<String>,

    #[serde(default)]
    pub scopes: Option<Vec<String>>,

    #[serde(default)]
    pub allow_merge_commits: bool,

    #[serde(default)]
    pub allow_revert_commits: bool,

    #[serde(default = "default_target_url")]
    pub target_url: String,

    #[serde(default = "default_charset", skip_serializing)]
    pub charset: CharSetConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CharSetConfig {
    Ascii,
    Unicode,
}

impl From<CharSetConfig> for CharSet {
    fn from(config: CharSetConfig) -> Self {
        match config {
            CharSetConfig::Ascii => CharSet::Ascii,
            CharSetConfig::Unicode => CharSet::Unicode,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_types() -> Vec<String> {
    vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "perf".to_string(),
        "test".to_string(),
        "build".to_string(),
        "ci".to_string(),
        "chore".to_string(),
        "revert".to_string(),
    ]
}

fn default_target_url() -> String {
    "https://github.com/Ezard/semantic-prs".to_string()
}

fn default_charset() -> CharSetConfig {
    CharSetConfig::Ascii
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            title_only: false,
            commits_only: false,
            title_and_commits: false,
            any_commit: false,
            types: default_types(),
            scopes: None,
            allow_merge_commits: false,
            allow_revert_commits: false,
            target_url: default_target_url(),
            charset: default_charset(),
        }
    }
}

impl Config {
    /// Load configuration with the following precedence:
    /// 1. Path specified via config_path parameter
    /// 2. .github/semantic.yml
    /// 3. .github/semantic.yaml
    /// 4. .github/semantic.json
    /// 5. .github/semantic.jsonc
    /// 6. .github/semantic.toml
    /// 7. XDG_CONFIG_DIR/conventional-prs/config.toml
    /// 8. $HOME/.config/conventional-prs/config.toml
    /// 9. Default values
    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        // If explicit path is provided, use it
        if let Some(path) = config_path {
            return Self::load_from_path(path);
        }

        // Try standard locations in order
        let candidate_paths = vec![
            PathBuf::from(".github/semantic.yml"),
            PathBuf::from(".github/semantic.yaml"),
            PathBuf::from(".github/semantic.json"),
            PathBuf::from(".github/semantic.jsonc"),
            PathBuf::from(".github/semantic.toml"),
        ];

        for path in candidate_paths {
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }

        // Try XDG_CONFIG_DIR
        if let Ok(xdg_dir) = std::env::var("XDG_CONFIG_DIR") {
            let path = PathBuf::from(xdg_dir).join("conventional-prs/config.toml");
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }

        // Try $HOME/.config/conventional-prs/config.toml
        if let Ok(home_dir) = std::env::var("HOME") {
            let path = PathBuf::from(home_dir).join(".config/conventional-prs/config.toml");
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }

        // Return default configuration if no file found
        Ok(Self::default())
    }

    fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        // Determine format by extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "yml" | "yaml" => {
                let config: Config = serde_yaml::from_str(&content)?;
                Ok(config)
            }
            "json" => {
                let config: Config = serde_json::from_str(&content)?;
                Ok(config)
            }
            "jsonc" => {
                // Strip comments before parsing
                let stripped = json_comments::StripComments::new(content.as_bytes());
                let config: Config = serde_json::from_reader(stripped)?;
                Ok(config)
            }
            "toml" => {
                let config: Config = toml::from_str(&content)?;
                Ok(config)
            }
            _ => {
                // Try to auto-detect format
                // Try JSON first (most strict)
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    return Ok(config);
                }
                // Try YAML
                if let Ok(config) = serde_yaml::from_str::<Config>(&content) {
                    return Ok(config);
                }
                // Try TOML
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return Ok(config);
                }
                // If all fail, return default
                Ok(Self::default())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.title_only, false);
        assert_eq!(config.types.len(), 11);
        assert!(config.types.contains(&"feat".to_string()));
        assert!(config.scopes.is_none());
    }

    #[test]
    fn test_yaml_parsing() {
        let yaml = r#"
enabled: true
titleOnly: false
types:
  - feat
  - fix
scopes:
  - api
  - ui
targetUrl: "https://example.com"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.title_only, false);
        assert_eq!(config.types, vec!["feat", "fix"]);
        assert_eq!(
            config.scopes,
            Some(vec!["api".to_string(), "ui".to_string()])
        );
        assert_eq!(config.target_url, "https://example.com");
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{
            "enabled": true,
            "titleOnly": true,
            "types": ["feat", "fix"],
            "scopes": ["core"]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.title_only, true);
        assert_eq!(config.types, vec!["feat", "fix"]);
        assert_eq!(config.scopes, Some(vec!["core".to_string()]));
    }

    #[test]
    fn test_jsonc_parsing_with_comments() {
        let jsonc = r#"{
            // This is a comment
            "enabled": true,
            "types": ["feat", "fix"] // inline comment
        }"#;
        let stripped = json_comments::StripComments::new(jsonc.as_bytes());
        let config: Config = serde_json::from_reader(stripped).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.types, vec!["feat", "fix"]);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
enabled = true
titleOnly = false
types = ["feat", "fix", "docs"]
scopes = ["api", "cli"]
targetUrl = "https://example.com"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.title_only, false);
        assert_eq!(config.types, vec!["feat", "fix", "docs"]);
    }
}
