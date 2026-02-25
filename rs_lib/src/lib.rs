use conventional_prs::{Config, ConventionalParser, OutputFormat};
use serde_json::json;
use wasm_bindgen::prelude::*;

fn config_json(config: &Config) -> serde_json::Value {
    json!({
        "enabled": config.enabled,
        "titleOnly": config.title_only,
        "commitsOnly": config.commits_only,
        "titleAndCommits": config.title_and_commits,
        "anyCommit": config.any_commit,
        "types": config.types,
        "scopes": config.scopes,
        "allowMergeCommits": config.allow_merge_commits,
        "allowRevertCommits": config.allow_revert_commits,
        "targetUrl": config.target_url,
    })
}

fn validate_with_config(input: &str, config: &Config) -> String {
    let parser = ConventionalParser::new(config.types.clone(), config.scopes.clone());
    let result = parser.parse(input);

    if result.is_ok() {
        let header = result
            .output()
            .expect("parse output must exist when parse result is ok");
        json!({
            "ok": true,
            "header": {
                "type": header.commit_type,
                "scope": header.scope,
                "breaking": header.breaking,
                "description": header.description
            }
        })
        .to_string()
    } else {
        let errors = result
            .errors()
            .expect("parse errors must exist when parse result is err")
            .iter()
            .map(|error| {
                json!({
                    "kind": format!("{:?}", error.kind),
                    "span": {
                        "start": error.span.start,
                        "end": error.span.end
                    }
                })
            })
            .collect::<Vec<_>>();

        json!({
            "ok": false,
            "errors": errors
        })
        .to_string()
    }
}

fn pretty_print_with_config(input: &str, config: &Config) -> String {
    let parser = ConventionalParser::new(config.types.clone(), config.scopes.clone());
    let result = parser.parse(input);

    if result.is_ok() {
        String::new()
    } else {
        result
            .report(OutputFormat::Ascii, config.charset)
            .unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn validate_header(input: &str) -> String {
    let config = Config::default();
    validate_with_config(input, &config)
}

#[wasm_bindgen]
pub fn validate_header_with_config(input: &str, semantic_yaml_raw: &str) -> String {
    match serde_yaml::from_str::<Config>(semantic_yaml_raw) {
        Ok(config) => validate_with_config(input, &config),
        Err(error) => json!({
            "ok": false,
            "configError": format!("{error}")
        })
        .to_string(),
    }
}

#[wasm_bindgen]
pub fn pretty_print_header(input: &str) -> String {
    let config = Config::default();
    pretty_print_with_config(input, &config)
}

#[wasm_bindgen]
pub fn pretty_print_header_with_config(input: &str, semantic_yaml_raw: &str) -> String {
    match serde_yaml::from_str::<Config>(semantic_yaml_raw) {
        Ok(config) => pretty_print_with_config(input, &config),
        Err(error) => format!("Config parse error: {error}"),
    }
}

#[wasm_bindgen]
pub fn parse_semantic_yaml_config(semantic_yaml_raw: &str) -> String {
    match serde_yaml::from_str::<Config>(semantic_yaml_raw) {
        Ok(config) => json!({
            "ok": true,
            "config": config_json(&config)
        })
        .to_string(),
        Err(error) => json!({
            "ok": false,
            "configError": format!("{error}")
        })
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_valid_header() {
        let output = validate_header("feat(api): add endpoint");
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], true);
        assert_eq!(json["header"]["type"], "feat");
        assert_eq!(json["header"]["description"], "add endpoint");
    }

    #[test]
    fn validates_invalid_header() {
        let output = validate_header("fature: typo in type");
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], false);
        assert!(json["errors"]
            .as_array()
            .expect("errors should be array")
            .iter()
            .any(|entry| entry["kind"]
                .as_str()
                .unwrap_or_default()
                .contains("InvalidType")));
    }

    #[test]
    fn validates_header_with_custom_semantic_yaml() {
        let semantic_yaml = "types: [foo]\nscopes: [core]\n";
        let output = validate_header_with_config("foo(core): add custom type", semantic_yaml);
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], true);
        assert_eq!(json["header"]["type"], "foo");
        assert_eq!(json["header"]["scope"][0], "core");
    }

    #[test]
    fn returns_config_error_for_invalid_semantic_yaml() {
        let output = validate_header_with_config("feat: add endpoint", "types: [feat");
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], false);
        assert!(json["configError"].is_string());
    }

    #[test]
    fn pretty_report_is_empty_for_valid_header() {
        let output = pretty_print_header("feat(api): add endpoint");

        assert_eq!(output, "");
    }

    #[test]
    fn pretty_report_contains_invalid_type_message() {
        let output = pretty_print_header("fature: typo in type");

        assert!(output.contains("Invalid commit type"));
    }

    #[test]
    fn pretty_report_contains_config_error_for_invalid_yaml() {
        let output = pretty_print_header_with_config("feat: add endpoint", "types: [feat");

        assert!(output.contains("Config parse error"));
    }

    #[test]
    fn parses_semantic_yaml_config_to_json() {
        let output = parse_semantic_yaml_config("types: [foo]\nscopes: [core]\n");
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], true);
        assert_eq!(json["config"]["types"][0], "foo");
        assert_eq!(json["config"]["scopes"][0], "core");
    }

    #[test]
    fn parse_semantic_yaml_config_returns_error_for_invalid_yaml() {
        let output = parse_semantic_yaml_config("types: [feat");
        let json: serde_json::Value = serde_json::from_str(&output).expect("valid json output");

        assert_eq!(json["ok"], false);
        assert!(json["configError"].is_string());
    }
}
