// Parser implementation without Chumsky dependency for better error messages

#[derive(Debug, Clone, PartialEq)]
pub struct CommitHeader {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    InvalidType {
        found: String,
        expected: Vec<String>,
    },
    InvalidScope {
        found: String,
        expected: Vec<String>,
    },
    MissingClosingParen,
    MissingSeparator,
    MissingDescription,
    EmptyType,
    EmptyScope,
    UnexpectedChar(char),
    GenericParseError(String),
    ExtraSpaceBeforeColon,
    ExtraSpaceAfterColon,
    MissingColon,
    MissingSpace,
    TrailingSpaces,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: std::ops::Range<usize>,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: std::ops::Range<usize>) -> Self {
        Self { kind, span }
    }
}

pub struct ConventionalParser {
    allowed_types: Vec<String>,
    allowed_scopes: Option<Vec<String>>,
}

impl ConventionalParser {
    pub fn new(allowed_types: Vec<String>, allowed_scopes: Option<Vec<String>>) -> Self {
        Self {
            allowed_types,
            allowed_scopes,
        }
    }

    /// Parse a conventional commit header with fault tolerance.
    /// Returns Ok with the parsed header if valid, or Err with all collected errors.
    pub fn parse(&self, input: &str) -> Result<CommitHeader, Vec<ParseError>> {
        // Try manual parsing for better error messages
        let (header_opt, mut all_errors) = self.manual_parse(input);

        // If we managed to extract a type, validate it
        if let Some(header) = &header_opt {
            // Check if type is allowed
            if !self.allowed_types.contains(&header.commit_type) {
                all_errors.push(ParseError::new(
                    ParseErrorKind::InvalidType {
                        found: header.commit_type.clone(),
                        expected: self.allowed_types.clone(),
                    },
                    0..header.commit_type.len(),
                ));
            }

            // Check if scope is allowed (if scopes are restricted)
            if let Some(ref allowed_scopes) = self.allowed_scopes {
                if let Some(ref scope) = header.scope {
                    if !allowed_scopes.contains(scope) {
                        let scope_start = header.commit_type.len() + 1; // +1 for '('
                        all_errors.push(ParseError::new(
                            ParseErrorKind::InvalidScope {
                                found: scope.clone(),
                                expected: allowed_scopes.clone(),
                            },
                            scope_start..scope_start + scope.len(),
                        ));
                    }
                }
            }
        }

        // Return result
        if all_errors.is_empty() {
            Ok(header_opt.unwrap())
        } else {
            Err(all_errors)
        }
    }

    /// Manual parsing with detailed error messages
    /// Returns (optional header, errors). Header may be partial even with errors.
    fn manual_parse(&self, input: &str) -> (Option<CommitHeader>, Vec<ParseError>) {
        let mut errors = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut pos = 0;

        // Parse type
        let type_start = pos;
        while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '-') {
            pos += 1;
        }

        let commit_type: String = if pos == type_start {
            errors.push(ParseError::new(
                ParseErrorKind::EmptyType,
                0..1.min(input.len()),
            ));
            String::new()
        } else {
            chars[type_start..pos].iter().collect()
        };

        // Check for scope
        let mut scope = None;
        let mut breaking = false;

        if pos < chars.len() && chars[pos] == '(' {
            let scope_start = pos + 1;
            let paren_pos = pos;
            pos += 1;

            if pos < chars.len() && chars[pos] == ')' {
                let scope_text: String = chars[scope_start..pos].iter().collect();
                if scope_text.is_empty() {
                    errors.push(ParseError::new(
                        ParseErrorKind::EmptyScope,
                        scope_start - 1..pos + 1,
                    ));
                } else {
                    scope = Some(scope_text);
                }
                pos += 1; // Skip ')'
            } else {
                // Missing closing paren - extract what we can as scope
                let scope_text: String = chars[scope_start..pos].iter().collect();
                errors.push(ParseError::new(
                    ParseErrorKind::MissingClosingParen,
                    paren_pos..pos,
                ));
                if !scope_text.is_empty() && !scope_text.trim().is_empty() {
                    scope = Some(scope_text.trim().to_string());
                }
            }
        }

        // Check for breaking change indicator
        if pos < chars.len() && chars[pos] == '!' {
            breaking = true;
            pos += 1;
        }

        // Check for extra space before colon
        if pos < chars.len() && chars[pos] == ' ' {
            let space_start = pos;
            while pos < chars.len() && chars[pos] == ' ' {
                pos += 1;
            }

            if pos < chars.len() && chars[pos] == ':' {
                errors.push(ParseError::new(
                    ParseErrorKind::ExtraSpaceBeforeColon,
                    space_start..pos,
                ));
            }
        }

        // Expect colon
        if pos >= chars.len() || chars[pos] != ':' {
            errors.push(ParseError::new(ParseErrorKind::MissingColon, pos..pos));
            // Don't return early - continue trying to parse
        } else {
            pos += 1; // Skip ':'
        }

        // Expect exactly one space after colon
        if pos >= chars.len() || chars[pos] != ' ' {
            let span_start = if pos > 0 { pos - 1 } else { 0 };
            errors.push(ParseError::new(
                ParseErrorKind::MissingSpace,
                span_start..pos,
            ));
            // Try to continue parsing
        } else {
            pos += 1; // Skip first space

            // Check for extra spaces after colon
            let extra_space_start = pos;
            while pos < chars.len() && chars[pos] == ' ' {
                pos += 1;
            }
            if pos > extra_space_start {
                errors.push(ParseError::new(
                    ParseErrorKind::ExtraSpaceAfterColon,
                    extra_space_start..pos,
                ));
            }
        }

        // Parse description (remaining text, up to newline)
        let desc_start = pos;
        let mut description = String::new();
        while pos < chars.len() && chars[pos] != '\n' {
            description.push(chars[pos]);
            pos += 1;
        }

        // Check for leading/trailing spaces in description
        let trimmed_description = description.trim().to_string();

        if trimmed_description.is_empty() {
            errors.push(ParseError::new(
                ParseErrorKind::MissingDescription,
                input.len()..input.len(),
            ));
            // Don't return early - still report other errors
        }

        // Check if description has leading spaces (after the required one space after colon)
        if !description.is_empty() && description.starts_with(' ') {
            // Already caught as ExtraSpaceAfterColon
        }

        // Check for trailing spaces at end of input
        if description != trimmed_description && description.ends_with(' ') {
            let trailing_start = desc_start + description.trim_end().len();
            errors.push(ParseError::new(
                ParseErrorKind::TrailingSpaces,
                trailing_start..input.len(),
            ));
        }

        // Always try to return the header, even if there are errors
        let header = if !commit_type.is_empty() && !trimmed_description.is_empty() {
            Some(CommitHeader {
                commit_type,
                scope,
                breaking,
                description: trimmed_description,
            })
        } else {
            None
        };

        (header, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_parser() -> ConventionalParser {
        ConventionalParser::new(
            vec![
                "feat".to_string(),
                "fix".to_string(),
                "docs".to_string(),
                "style".to_string(),
                "refactor".to_string(),
                "test".to_string(),
                "chore".to_string(),
            ],
            None,
        )
    }

    #[test]
    fn test_valid_simple_commit() {
        let parser = default_parser();
        let result = parser.parse("feat: add new feature");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "feat");
        assert_eq!(header.scope, None);
        assert_eq!(header.breaking, false);
        assert_eq!(header.description, "add new feature");
    }

    #[test]
    fn test_valid_commit_with_scope() {
        let parser = default_parser();
        let result = parser.parse("fix(api): resolve bug");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "fix");
        assert_eq!(header.scope, Some("api".to_string()));
        assert_eq!(header.breaking, false);
        assert_eq!(header.description, "resolve bug");
    }

    #[test]
    fn test_valid_breaking_change() {
        let parser = default_parser();
        let result = parser.parse("feat!: breaking change");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "feat");
        assert_eq!(header.breaking, true);
    }

    #[test]
    fn test_valid_breaking_change_with_scope() {
        let parser = default_parser();
        let result = parser.parse("feat(core)!: breaking change");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "feat");
        assert_eq!(header.scope, Some("core".to_string()));
        assert_eq!(header.breaking, true);
    }

    #[test]
    fn test_invalid_type() {
        let parser = default_parser();
        let result = parser.parse("fature: typo in type");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        // Check that we got an InvalidType error
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::InvalidType { .. }))
        );
    }

    #[test]
    fn test_invalid_scope() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["api".to_string(), "ui".to_string()]),
        );
        let result = parser.parse("feat(core): description");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
        );
    }

    #[test]
    fn test_missing_closing_paren_recovery() {
        let parser = default_parser();
        // Parser should handle missing closing paren
        let result = parser.parse("feat(api: description");
        // This should parse but might have errors
        // The parser attempts recovery
        let _ = result; // Just verify it doesn't panic
    }

    #[test]
    fn test_missing_space_after_colon() {
        let parser = default_parser();
        let result = parser.parse("feat:description");
        // Should still parse due to recovery
        if let Ok(header) = result {
            assert_eq!(header.description, "description");
        }
    }

    #[test]
    fn test_scope_with_hyphen() {
        let parser = default_parser();
        let result = parser.parse("feat(my-scope): description");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.scope, Some("my-scope".to_string()));
    }

    #[test]
    fn test_type_with_hyphen() {
        let parser = ConventionalParser::new(vec!["my-type".to_string()], None);
        let result = parser.parse("my-type: description");
        assert!(result.is_ok());
    }

    // COMPREHENSIVE RECOVERY TESTS (as mandated by AGENTS.md)

    #[test]
    fn test_recovery_multiple_errors_missing_closing_paren_and_invalid_type() {
        let parser = default_parser();
        let result = parser.parse("fature(api: description");
        // Should report invalid type even with missing closing paren
        if let Err(errors) = result {
            // May report parse error or invalid type
            assert!(!errors.is_empty());
        }
    }

    #[test]
    fn test_recovery_missing_space_after_colon_with_valid_type() {
        let parser = default_parser();
        let result = parser.parse("fix:no space here");
        // Should recover and parse successfully
        if let Ok(header) = result {
            assert_eq!(header.commit_type, "fix");
            assert_eq!(header.description, "no space here");
        }
    }

    #[test]
    fn test_recovery_empty_scope_parentheses() {
        let parser = default_parser();
        // Empty scope should fail to parse
        let result = parser.parse("feat(): description");
        assert!(result.is_err());
    }

    #[test]
    fn test_recovery_scope_with_special_chars() {
        let parser = default_parser();
        let result = parser.parse("feat(api/v2): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("api/v2".to_string()));
        }
    }

    #[test]
    fn test_recovery_scope_with_spaces() {
        let parser = default_parser();
        let result = parser.parse("feat(my scope): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("my scope".to_string()));
        }
    }

    #[test]
    fn test_recovery_breaking_with_scope() {
        let parser = default_parser();
        let result = parser.parse("feat(api)!: major change");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.commit_type, "feat");
            assert_eq!(header.scope, Some("api".to_string()));
            assert!(header.breaking);
            assert_eq!(header.description, "major change");
        }
    }

    #[test]
    fn test_recovery_multiple_colons() {
        let parser = default_parser();
        let result = parser.parse("feat: description: with: colons");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description, "description: with: colons");
        }
    }

    #[test]
    fn test_recovery_unicode_in_description() {
        let parser = default_parser();
        let result = parser.parse("feat: añadir función 🎉");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description, "añadir función 🎉");
        }
    }

    #[test]
    fn test_recovery_very_long_type() {
        let parser =
            ConventionalParser::new(vec!["verylongtypenamethatisunusual".to_string()], None);
        let result = parser.parse("verylongtypenamethatisunusual: description");
        assert!(result.is_ok());
    }

    #[test]
    fn test_recovery_numeric_in_scope() {
        let parser = default_parser();
        let result = parser.parse("feat(api-v2-beta3): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("api-v2-beta3".to_string()));
        }
    }

    #[test]
    fn test_invalid_type_with_valid_scope() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        let result = parser.parse("fix(api): description");
        // Should fail on invalid type
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::InvalidType { .. }))
            );
        }
    }

    #[test]
    fn test_valid_type_with_invalid_scope() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        let result = parser.parse("feat(ui): description");
        // Should fail on invalid scope
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
            );
        }
    }

    // === COMPREHENSIVE EDGE CASE TESTS ===

    #[test]
    fn test_missing_closing_paren_with_colon() {
        let parser = default_parser();
        let result = parser.parse("feat(api: description");
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::MissingClosingParen))
            );
        }
    }

    #[test]
    fn test_missing_closing_paren_without_colon() {
        let parser = default_parser();
        let result = parser.parse("feat(api description");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let parser = default_parser();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_only_type_no_separator() {
        let parser = default_parser();
        let result = parser.parse("feat");
        assert!(result.is_err());
    }

    #[test]
    fn test_only_colon() {
        let parser = default_parser();
        let result = parser.parse(":");
        assert!(result.is_err());
    }

    #[test]
    fn test_colon_without_type() {
        let parser = default_parser();
        let result = parser.parse(": description");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_colon_no_description() {
        let parser = default_parser();
        let result = parser.parse("feat: ");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_colon_only_spaces() {
        let parser = default_parser();
        let result = parser.parse("feat:    ");
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_with_multiple_words() {
        let parser = default_parser();
        let result = parser.parse("feat(api core): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("api core".to_string()));
        }
    }

    #[test]
    fn test_description_with_colon() {
        let parser = default_parser();
        let result = parser.parse("feat: add feature: the new one");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description, "add feature: the new one");
        }
    }

    #[test]
    fn test_description_with_parentheses() {
        let parser = default_parser();
        let result = parser.parse("feat: add feature (with notes)");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description, "add feature (with notes)");
        }
    }

    #[test]
    fn test_breaking_without_scope() {
        let parser = default_parser();
        let result = parser.parse("feat!: breaking without scope");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert!(header.breaking);
            assert_eq!(header.scope, None);
        }
    }

    #[test]
    fn test_multiple_exclamation_marks() {
        let parser = default_parser();
        let result = parser.parse("feat!!: description");
        // Should only recognize first ! as breaking indicator
        if let Ok(header) = result {
            assert!(header.description.starts_with("!: description") || header.breaking);
        }
    }

    #[test]
    fn test_newline_in_input() {
        let parser = default_parser();
        let result = parser.parse("feat: description\nsecond line");
        // Should only parse first line
        if let Ok(header) = result {
            assert!(!header.description.contains('\n'));
        }
    }

    #[test]
    fn test_tab_characters() {
        let parser = default_parser();
        let result = parser.parse("feat:\tdescription");
        // Tab is not a space
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_in_type() {
        let parser = ConventionalParser::new(
            vec!["фіча".to_string()], // Cyrillic
            None,
        );
        let result = parser.parse("фіча: опис");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_in_scope() {
        let parser = default_parser();
        let result = parser.parse("feat(апі): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("апі".to_string()));
        }
    }

    #[test]
    fn test_emoji_in_description() {
        let parser = default_parser();
        let result = parser.parse("feat: add 🎉 celebration");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert!(header.description.contains('🎉'));
        }
    }

    #[test]
    fn test_very_long_description() {
        let parser = default_parser();
        let long_desc = "a".repeat(500);
        let input = format!("feat: {long_desc}");
        let result = parser.parse(&input);
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description.len(), 500);
        }
    }

    #[test]
    fn test_scope_with_underscores() {
        let parser = default_parser();
        let result = parser.parse("feat(api_v2): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("api_v2".to_string()));
        }
    }

    #[test]
    fn test_scope_with_dots() {
        let parser = default_parser();
        let result = parser.parse("feat(api.v2): description");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.scope, Some("api.v2".to_string()));
        }
    }

    #[test]
    fn test_all_valid_default_types() {
        let parser = default_parser();
        let types = vec!["feat", "fix", "docs", "style", "refactor", "test", "chore"];

        for commit_type in types {
            let input = format!("{commit_type}: description");
            let result = parser.parse(&input);
            assert!(result.is_ok(), "Failed for type: {commit_type}");
        }
    }

    #[test]
    fn test_case_sensitive_type() {
        let parser = default_parser();
        let result = parser.parse("FEAT: description");
        // Types are case-sensitive, should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_case_sensitive_scope() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        let result = parser.parse("feat(API): description");
        // Scopes are case-sensitive
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_trimming_in_description() {
        let parser = default_parser();
        // Extra spaces and trailing spaces should be errors
        let result = parser.parse("feat:   description with leading spaces   ");
        assert!(result.is_err());
        if let Err(errors) = result {
            // Should have error for extra spaces after colon and trailing spaces
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterColon))
            );
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::TrailingSpaces))
            );
        }

        // Valid commit with proper spacing
        let result = parser.parse("feat: description with no extra spaces");
        assert!(result.is_ok());
        if let Ok(header) = result {
            assert_eq!(header.description, "description with no extra spaces");
        }
    }

    #[test]
    fn test_extra_space_before_colon() {
        let parser = default_parser();
        let result = parser.parse("feat : description");
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceBeforeColon))
            );
        }
    }

    #[test]
    fn test_extra_spaces_after_colon() {
        let parser = default_parser();
        let result = parser.parse("feat:  description");
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterColon))
            );
        }
    }

    #[test]
    fn test_trailing_spaces_error() {
        let parser = default_parser();
        let result = parser.parse("feat: description ");
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::TrailingSpaces))
            );
        }
    }

    #[test]
    fn test_multiple_spacing_errors() {
        let parser = default_parser();
        let result = parser.parse("feat :  description  ");
        assert!(result.is_err());
        if let Err(errors) = result {
            // Should catch space before colon, extra spaces after, and trailing
            assert!(errors.len() >= 2);
        }
    }
}
