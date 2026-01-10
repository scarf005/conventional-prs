use conventional_prs::{Config, ConventionalParser, ErrorReporter, OutputFormat};
use std::path::PathBuf;

#[test]
fn test_end_to_end_valid_commit() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);

    let result = parser.parse("feat: add new feature");
    assert!(result.is_ok());
}

#[test]
fn test_end_to_end_invalid_commit() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);

    let result = parser.parse("fature: typo");
    assert!(result.is_err());
}

#[test]
fn test_end_to_end_error_reporting() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);
    let reporter = ErrorReporter::new(OutputFormat::Ascii);

    let input = "fature: typo";
    let result = parser.parse(input);
    if let Some(errors) = result.errors() {
        let report = reporter.report_errors(input, errors);
        assert!(report.contains("Invalid commit type"));
        assert!(report.contains("fature"));
        // GitHub format should not have ANSI colors
        assert!(!report.contains("\x1b["));
    } else {
        panic!("Expected parse to fail");
    }
}

#[test]
fn test_end_to_end_with_scope_validation() {
    let config = Config {
        types: vec!["feat".to_string(), "fix".to_string()],
        scopes: Some(vec!["api".to_string(), "ui".to_string()]),
        ..Default::default()
    };
    let parser = ConventionalParser::new(config.types, config.scopes);

    // Valid type and scope
    assert!(parser.parse("feat(api): description").is_ok());

    // Valid type but invalid scope
    let result = parser.parse("feat(core): description");
    assert!(result.is_err());
}

#[test]
fn test_end_to_end_breaking_changes() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);

    // Breaking change without scope
    let result = parser.parse("feat!: breaking change");
    assert!(result.is_ok());
    if let Some(header) = result.output() {
        assert!(header.breaking);
    }

    // Breaking change with scope
    let result = parser.parse("feat(api)!: breaking change");
    assert!(result.is_ok());
    if let Some(header) = result.output() {
        assert!(header.breaking);
        assert_eq!(header.scope, Some(vec!["api".to_string()]));
    }
}

#[test]
fn test_end_to_end_edge_cases() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);

    // Multiple colons in description
    assert!(parser.parse("feat: add feature: with: colons").is_ok());

    // Unicode characters
    assert!(parser.parse("feat: añadir función 🎉").is_ok());

    // Long description
    let long_desc = "a".repeat(200);
    assert!(parser.parse(&format!("feat: {}", long_desc)).is_ok());
}

#[test]
fn test_config_with_custom_types() {
    let config = Config {
        types: vec!["custom".to_string(), "mytype".to_string()],
        ..Default::default()
    };
    let parser = ConventionalParser::new(config.types, config.scopes);

    // Custom type should work
    assert!(parser.parse("custom: description").is_ok());

    // Standard type should fail
    assert!(parser.parse("feat: description").is_err());
}

#[test]
fn test_disabled_config() {
    let config = Config {
        enabled: false,
        ..Default::default()
    };

    // When disabled, we would skip validation in main.rs
    // Here we just verify the config field works
    assert!(!config.enabled);
}

#[test]
fn test_reporter_formats() {
    let config = Config::default();
    let parser = ConventionalParser::new(config.types, config.scopes);

    let input = "wrongtype: desc";
    let result = parser.parse(input);
    if let Some(errors) = result.errors() {
        // Test default format (with colors)
        let reporter_default = ErrorReporter::new(OutputFormat::Color);
        let report_default = reporter_default.report_errors(input, errors);
        // Default format may have colors (ANSI codes)

        // Test GitHub format (no colors)
        let reporter_github = ErrorReporter::new(OutputFormat::Ascii);
        let report_github = reporter_github.report_errors(input, errors);
        assert!(!report_github.contains("\x1b["));

        // Both should contain error message
        assert!(report_default.contains("Invalid commit type"));
        assert!(report_github.contains("Invalid commit type"));
    } else {
        panic!("Expected parse to fail");
    }
}

#[test]
fn test_config_loading_yaml() {
    let path = PathBuf::from("tests/fixtures/test-config.yml");
    let config = Config::load(Some(&path)).expect("Failed to load YAML config");

    assert!(config.enabled);
    assert!(config.title_only);
    assert_eq!(config.types, vec!["feat", "fix", "docs"]);
    assert_eq!(
        config.scopes,
        Some(vec![
            "api".to_string(),
            "ui".to_string(),
            "core".to_string()
        ])
    );
    assert_eq!(config.target_url, "https://example.com/conventional");
}

#[test]
fn test_config_loading_json() {
    let path = PathBuf::from("tests/fixtures/test-config.json");
    let config = Config::load(Some(&path)).expect("Failed to load JSON config");

    assert!(config.enabled);
    assert!(config.commits_only);
    assert_eq!(config.types, vec!["feat", "fix"]);
    assert_eq!(
        config.scopes,
        Some(vec!["api".to_string(), "ui".to_string()])
    );
    assert!(config.allow_merge_commits);
}

#[test]
fn test_config_loading_jsonc() {
    let path = PathBuf::from("tests/fixtures/test-config.jsonc");
    let config = Config::load(Some(&path)).expect("Failed to load JSONC config");

    assert!(config.enabled);
    assert_eq!(config.types, vec!["feat", "fix", "docs"]);
    assert_eq!(config.scopes, Some(vec!["api".to_string()]));
}

#[test]
fn test_config_loading_toml() {
    let path = PathBuf::from("tests/fixtures/test-config.toml");
    let config = Config::load(Some(&path)).expect("Failed to load TOML config");

    assert!(config.enabled);
    assert!(config.title_and_commits);
    assert_eq!(config.types, vec!["feat", "fix", "chore"]);
    assert_eq!(
        config.scopes,
        Some(vec!["core".to_string(), "api".to_string()])
    );
    assert!(config.allow_revert_commits);
}

#[test]
fn test_config_loading_defaults_when_no_file() {
    let config = Config::load(None).expect("Failed to load default config");

    // Should use defaults
    assert!(config.enabled);
    assert!(!config.title_only);
    assert_eq!(config.types.len(), 11); // Default types
    assert!(config.scopes.is_none());
}

#[test]
fn test_config_with_all_formats() {
    // Test that all format files can be loaded successfully
    let formats = vec![
        "tests/fixtures/test-config.yml",
        "tests/fixtures/test-config.json",
        "tests/fixtures/test-config.jsonc",
        "tests/fixtures/test-config.toml",
    ];

    for format_path in formats {
        let path = PathBuf::from(format_path);
        let result = Config::load(Some(&path));
        assert!(result.is_ok(), "Failed to load {}", format_path);
    }
}
