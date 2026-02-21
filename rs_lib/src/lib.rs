use conventional_prs::{Config, ConventionalParser};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate_header(input: &str) -> String {
    let config = Config::default();
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
}
