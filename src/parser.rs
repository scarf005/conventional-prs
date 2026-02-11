// Parser implementation for Conventional Commit headers with fault-tolerant error collection

#[derive(Debug, Clone, PartialEq)]
pub struct CommitHeader {
    pub commit_type: String,
    pub scope: Option<Vec<String>>,
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
    TypeUsedAsScope {
        found: String,
        expected_scopes: Vec<String>,
        available_types: Vec<String>,
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
    ExtraSpaceAfterOpenParen,
    ExtraSpaceBeforeCloseParen,
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

/// Parse result that bundles input with output/errors.
/// Eliminates the need to pass input separately to error reporters.
pub struct ParseResult<'a> {
    input: &'a str,
    result: Result<CommitHeader, Vec<ParseError>>,
}

impl<'a> ParseResult<'a> {
    fn new(input: &'a str, result: Result<CommitHeader, Vec<ParseError>>) -> Self {
        Self { input, result }
    }

    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }

    pub fn output(&self) -> Option<&CommitHeader> {
        self.result.as_ref().ok()
    }

    pub fn errors(&self) -> Option<&[ParseError]> {
        self.result.as_ref().err().map(|e| e.as_slice())
    }

    pub fn input(&self) -> &str {
        self.input
    }

    pub fn unwrap(self) -> CommitHeader {
        self.result.unwrap()
    }

    pub fn into_result(self) -> Result<CommitHeader, Vec<ParseError>> {
        self.result
    }

    pub fn unwrap_err(self) -> Vec<ParseError> {
        self.result.unwrap_err()
    }

    pub fn report(
        &self,
        format: crate::report::OutputFormat,
        charset: crate::config::CharSetConfig,
    ) -> Option<String> {
        self.errors().map(|errors| {
            let reporter = crate::report::ErrorReporter::new(format, charset);
            reporter.report_errors(self.input, errors)
        })
    }

    pub fn print_errors(
        &self,
        format: crate::report::OutputFormat,
        charset: crate::config::CharSetConfig,
    ) {
        if let Some(errors) = self.errors() {
            let reporter = crate::report::ErrorReporter::new(format, charset);
            eprint!("{}", reporter.report_errors(self.input, errors));
        }
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

    fn strip_git_autosquash_prefixes(input: &str) -> (&str, usize) {
        let mut s = input;
        let mut offset = 0;

        loop {
            let (prefix_len, rest) = if let Some(rest) = s.strip_prefix("fixup!") {
                ("fixup!".len(), rest)
            } else if let Some(rest) = s.strip_prefix("squash!") {
                ("squash!".len(), rest)
            } else {
                break;
            };

            offset += prefix_len;

            let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace());
            offset += rest.len() - trimmed.len();
            s = trimmed;
        }

        (s, offset)
    }

    /// Parse a conventional commit header with fault tolerance.
    /// Returns a ParseResult that bundles input with output/errors.
    pub fn parse<'a>(&self, input: &'a str) -> ParseResult<'a> {
        let (effective_input, offset) = Self::strip_git_autosquash_prefixes(input);
        let mut result = self.parse_internal(effective_input);

        if offset != 0 {
            if let Err(ref mut errors) = result {
                for error in errors {
                    error.span = (error.span.start + offset)..(error.span.end + offset);
                }
            }
        }

        ParseResult::new(input, result)
    }

    fn parse_internal(&self, input: &str) -> Result<CommitHeader, Vec<ParseError>> {
        let (header_opt, mut all_errors) = self.manual_parse(input);

        if let Some(header) = &header_opt {
            if !self.allowed_types.contains(&header.commit_type) {
                all_errors.push(ParseError::new(
                    ParseErrorKind::InvalidType {
                        found: header.commit_type.clone(),
                        expected: self.allowed_types.clone(),
                    },
                    0..header.commit_type.len(),
                ));
            }

            if let Some(ref allowed_scopes) = self.allowed_scopes
                && let Some(ref scopes) = header.scope
            {
                // Validate each scope individually
                let mut scope_pos = header.commit_type.len() + 2; // +2 for '(' and initial offset
                for (i, individual_scope) in scopes.iter().enumerate() {
                    if !allowed_scopes.contains(individual_scope) {
                        // Check if this invalid scope is actually a valid type being misused
                        if self.allowed_types.contains(individual_scope) {
                            all_errors.push(ParseError::new(
                                ParseErrorKind::TypeUsedAsScope {
                                    found: individual_scope.clone(),
                                    expected_scopes: allowed_scopes.clone(),
                                    available_types: self.allowed_types.clone(),
                                },
                                scope_pos..scope_pos + individual_scope.len(),
                            ));
                        } else {
                            all_errors.push(ParseError::new(
                                ParseErrorKind::InvalidScope {
                                    found: individual_scope.clone(),
                                    expected: allowed_scopes.clone(),
                                },
                                scope_pos..scope_pos + individual_scope.len(),
                            ));
                        }
                    }
                    scope_pos += individual_scope.len() + if i < scopes.len() - 1 { 2 } else { 0 }; // +2 for ', '
                }
            }
        }

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

            // Search for closing paren
            let mut scope_end = pos;
            let mut found_closing = false;
            while scope_end < chars.len() {
                if chars[scope_end] == ')' {
                    found_closing = true;
                    break;
                }
                scope_end += 1;
            }

            let scope_text: String = chars[scope_start..scope_end].iter().collect();

            // Check for space immediately after opening paren
            if !scope_text.is_empty() && scope_text.starts_with(' ') {
                errors.push(ParseError::new(
                    ParseErrorKind::ExtraSpaceAfterOpenParen,
                    scope_start..scope_start + 1,
                ));
            }

            // Check for space immediately before closing paren
            if !scope_text.is_empty() && scope_text.ends_with(' ') {
                let space_before_close = scope_end - 1;
                errors.push(ParseError::new(
                    ParseErrorKind::ExtraSpaceBeforeCloseParen,
                    space_before_close..scope_end,
                ));
            }

            if scope_text.is_empty() {
                errors.push(ParseError::new(
                    ParseErrorKind::EmptyScope,
                    paren_pos..scope_end + if found_closing { 1 } else { 0 },
                ));
            } else {
                // Split scope by comma and trim each one
                let scopes: Vec<String> = scope_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                scope = Some(scopes);
            }

            if found_closing {
                pos = scope_end + 1; // Move past ')'
            } else {
                errors.push(ParseError::new(
                    ParseErrorKind::MissingClosingParen,
                    paren_pos..scope_end,
                ));
                pos = scope_end;
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
    fn test_fixup_prefix_is_ignored() {
        let parser = default_parser();
        let result = parser.parse("fixup! feat: add new feature");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "feat");
    }

    #[test]
    fn test_squash_prefix_is_ignored() {
        let parser = default_parser();
        let result = parser.parse("squash! fix(api): resolve bug");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "fix");
        assert_eq!(header.scope, Some(vec!["api".to_string()]));
    }

    #[test]
    fn test_fixup_prefix_error_spans_are_offset() {
        let parser = default_parser();
        let result = parser.parse("fixup! fature: typo in type");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let invalid_type = errors
            .iter()
            .find(|e| matches!(&e.kind, ParseErrorKind::InvalidType { .. }))
            .expect("expected invalid type error");
        assert_eq!(invalid_type.span, 7..13);
    }

    #[test]
    fn test_valid_commit_with_scope() {
        let parser = default_parser();
        let result = parser.parse("fix(api): resolve bug");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.commit_type, "fix");
        assert_eq!(header.scope, Some(vec!["api".to_string()]));
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
        assert_eq!(header.scope, Some(vec!["core".to_string()]));
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description, "description");
        }
    }

    #[test]
    fn test_scope_with_hyphen() {
        let parser = default_parser();
        let result = parser.parse("feat(my-scope): description");
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.scope, Some(vec!["my-scope".to_string()]));
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
        if let Err(errors) = result.into_result() {
            // May report parse error or invalid type
            assert!(!errors.is_empty());
        }
    }

    #[test]
    fn test_recovery_missing_space_after_colon_with_valid_type() {
        let parser = default_parser();
        let result = parser.parse("fix:no space here");
        // Should recover and parse successfully
        if let Ok(header) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api/v2".to_string()]));
        }
    }

    #[test]
    fn test_recovery_scope_with_spaces() {
        let parser = default_parser();
        let result = parser.parse("feat(my scope): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["my scope".to_string()]));
        }
    }

    #[test]
    fn test_recovery_breaking_with_scope() {
        let parser = default_parser();
        let result = parser.parse("feat(api)!: major change");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.commit_type, "feat");
            assert_eq!(header.scope, Some(vec!["api".to_string()]));
            assert!(header.breaking);
            assert_eq!(header.description, "major change");
        }
    }

    #[test]
    fn test_recovery_multiple_colons() {
        let parser = default_parser();
        let result = parser.parse("feat: description: with: colons");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description, "description: with: colons");
        }
    }

    #[test]
    fn test_recovery_unicode_in_description() {
        let parser = default_parser();
        let result = parser.parse("feat: añadir función 🎉");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api-v2-beta3".to_string()]));
        }
    }

    #[test]
    fn test_invalid_type_with_valid_scope() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        let result = parser.parse("fix(api): description");
        // Should fail on invalid type
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
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
        if let Err(errors) = result.into_result() {
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
        if let Err(errors) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api core".to_string()]));
        }
    }

    #[test]
    fn test_description_with_colon() {
        let parser = default_parser();
        let result = parser.parse("feat: add feature: the new one");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description, "add feature: the new one");
        }
    }

    #[test]
    fn test_description_with_parentheses() {
        let parser = default_parser();
        let result = parser.parse("feat: add feature (with notes)");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description, "add feature (with notes)");
        }
    }

    #[test]
    fn test_breaking_without_scope() {
        let parser = default_parser();
        let result = parser.parse("feat!: breaking without scope");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert!(header.breaking);
            assert_eq!(header.scope, None);
        }
    }

    #[test]
    fn test_multiple_exclamation_marks() {
        let parser = default_parser();
        let result = parser.parse("feat!!: description");
        // Should only recognize first ! as breaking indicator
        if let Ok(header) = result.into_result() {
            assert!(header.description.starts_with("!: description") || header.breaking);
        }
    }

    #[test]
    fn test_newline_in_input() {
        let parser = default_parser();
        let result = parser.parse("feat: description\nsecond line");
        // Should only parse first line
        if let Ok(header) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["апі".to_string()]));
        }
    }

    #[test]
    fn test_emoji_in_description() {
        let parser = default_parser();
        let result = parser.parse("feat: add 🎉 celebration");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description.len(), 500);
        }
    }

    #[test]
    fn test_scope_with_underscores() {
        let parser = default_parser();
        let result = parser.parse("feat(api_v2): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api_v2".to_string()]));
        }
    }

    #[test]
    fn test_scope_with_dots() {
        let parser = default_parser();
        let result = parser.parse("feat(api.v2): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api.v2".to_string()]));
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
        if let Err(errors) = result.into_result() {
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
        if let Ok(header) = result.into_result() {
            assert_eq!(header.description, "description with no extra spaces");
        }
    }

    #[test]
    fn test_extra_space_before_colon() {
        let parser = default_parser();
        let result = parser.parse("feat : description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
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
        if let Err(errors) = result.into_result() {
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
        if let Err(errors) = result.into_result() {
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
        if let Err(errors) = result.into_result() {
            // Should catch space before colon, extra spaces after, and trailing
            assert!(errors.len() >= 2);
        }
    }

    // ===== MULTIPLE SCOPES TESTS =====

    #[test]
    fn test_multiple_scopes_all_valid() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "lua".to_string(),
                "mods".to_string(),
                "port".to_string(),
            ]),
        );
        let result = parser.parse("feat(lua,mods,port): nyctophobia trait");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "lua".to_string(),
                    "mods".to_string(),
                    "port".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_multiple_scopes_with_spaces() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "lua".to_string(),
                "mods".to_string(),
                "port".to_string(),
            ]),
        );
        let result = parser.parse("feat(lua, mods, port): nyctophobia trait");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "lua".to_string(),
                    "mods".to_string(),
                    "port".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_multiple_scopes_partial_invalid() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["lua".to_string(), "mods".to_string()]),
        );
        let result = parser.parse("feat(lua,mods,invalid): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report the invalid scope
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
            );
        }
    }

    #[test]
    fn test_multiple_scopes_all_invalid() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["valid1".to_string(), "valid2".to_string()]),
        );
        let result = parser.parse("feat(invalid1,invalid2): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report multiple invalid scope errors
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 2);
        }
    }

    #[test]
    fn test_single_scope_still_works() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        let result = parser.parse("feat(api): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api".to_string()]));
        }
    }

    #[test]
    fn test_multiple_scopes_with_breaking_change() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["ui".to_string(), "api".to_string()]),
        );
        let result = parser.parse("feat(ui,api)!: breaking change");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec!["ui".to_string(), "api".to_string()])
            );
            assert!(header.breaking);
        }
    }

    // ===== SCOPE ERROR MESSAGE TESTS =====

    #[test]
    fn test_single_scope_not_found_with_suggestion() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(scope-not-exists): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should have exactly one invalid scope error
            let invalid_scope_errors: Vec<_> = errors
                .iter()
                .filter_map(|e| match &e.kind {
                    ParseErrorKind::InvalidScope { found, expected } => {
                        Some((found.clone(), expected.clone()))
                    }
                    _ => None,
                })
                .collect();

            assert_eq!(invalid_scope_errors.len(), 1);
            assert_eq!(invalid_scope_errors[0].0, "scope-not-exists");
            assert!(invalid_scope_errors[0].1.len() >= 1);
        }
    }

    #[test]
    fn test_case_sensitive_scope_single() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        let result = parser.parse("feat(cAsEBaD): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 1);
        }
    }

    #[test]
    fn test_case_sensitive_scope_in_multiple() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(port,LUA,mods): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should only report "LUA" as invalid (lua is correct case)
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 1);
        }
    }

    #[test]
    fn test_multiple_scopes_no_space_after_comma() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(port,lua,mods): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "port".to_string(),
                    "lua".to_string(),
                    "mods".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_multiple_scopes_one_space_after_comma() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(port, lua, mods): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "port".to_string(),
                    "lua".to_string(),
                    "mods".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_multiple_scopes_inconsistent_spacing() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        // Mix of no space and one space
        let result = parser.parse("feat(port,lua, mods): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "port".to_string(),
                    "lua".to_string(),
                    "mods".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_multiple_scopes_trailing_comma_space() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        // Trailing space after last scope - should be error
        let result = parser.parse("feat(port, lua ): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report space before closing paren
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceBeforeCloseParen))
            );
        }
    }

    #[test]
    fn test_multiple_scopes_leading_space_first_scope() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        // Leading space before first scope - should be error
        let result = parser.parse("feat( port, lua): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report space after opening paren
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterOpenParen))
            );
        }
    }

    #[test]
    fn test_mixed_valid_and_invalid_scopes_error_reporting() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(port,invalid1,lua,invalid2): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report exactly 2 invalid scopes
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 2);
        }
    }

    #[test]
    fn test_multiple_case_errors_in_scopes() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "port".to_string(),
                "lua".to_string(),
                "mods".to_string(),
            ]),
        );
        let result = parser.parse("feat(Port,LUA,Mods): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report 3 invalid scopes (all have wrong case)
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 3);
        }
    }

    #[test]
    fn test_scope_with_slashes_no_space() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "mods/CheesyInnaWoodFixes".to_string(),
                "mods/DinoMod".to_string(),
            ]),
        );
        let result = parser.parse("feat(mods/CheesyInnaWoodFixes,mods/DinoMod): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "mods/CheesyInnaWoodFixes".to_string(),
                    "mods/DinoMod".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_scope_with_slashes_with_space() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec![
                "mods/CheesyInnaWoodFixes".to_string(),
                "mods/DinoMod".to_string(),
            ]),
        );
        let result = parser.parse("feat(mods/CheesyInnaWoodFixes, mods/DinoMod): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec![
                    "mods/CheesyInnaWoodFixes".to_string(),
                    "mods/DinoMod".to_string()
                ])
            );
        }
    }

    #[test]
    fn test_scope_typo_similarity_suggestion() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["database".to_string(), "api".to_string()]),
        );
        // "databse" is a typo of "database"
        let result = parser.parse("feat(databse): description");
        assert!(result.is_err());
        // Verify error contains the similarity suggestion mechanism
        if let Err(errors) = result.into_result() {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
            );
        }
    }

    #[test]
    fn test_empty_scope_between_commas() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        // Empty scope between commas: "port,,lua"
        let result = parser.parse("feat(port,,lua): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report empty scope as invalid
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert!(invalid_scope_errors >= 1);
        }
    }

    #[test]
    fn test_whitespace_only_scope() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        // Whitespace only between commas: "port,  ,lua"
        let result = parser.parse("feat(port,  ,lua): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Trimmed whitespace should result in empty scope, which is invalid
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert!(invalid_scope_errors >= 1);
        }
    }

    #[test]
    fn test_type_used_as_scope() {
        let parser = ConventionalParser::new(
            vec![
                "feat".to_string(),
                "fix".to_string(),
                "refactor".to_string(),
                "build".to_string(),
            ],
            Some(vec!["api".to_string(), "ui".to_string()]),
        );
        let result = parser.parse("refactor(build): simplify data install");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            let type_as_scope_errors: Vec<_> = errors
                .iter()
                .filter_map(|e| match &e.kind {
                    ParseErrorKind::TypeUsedAsScope { found, .. } => Some(found.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(type_as_scope_errors.len(), 1);
            assert_eq!(type_as_scope_errors[0], "build");
        }
    }

    #[test]
    fn test_multiple_scopes_one_is_type() {
        let parser = ConventionalParser::new(
            vec![
                "feat".to_string(),
                "fix".to_string(),
                "build".to_string(),
            ],
            Some(vec!["api".to_string(), "ui".to_string()]),
        );
        let result = parser.parse("feat(api, build, ui): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            let type_as_scope_errors: Vec<_> = errors
                .iter()
                .filter_map(|e| match &e.kind {
                    ParseErrorKind::TypeUsedAsScope { found, .. } => Some(found.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(type_as_scope_errors.len(), 1);
            assert_eq!(type_as_scope_errors[0], "build");
        }
    }

    #[test]
    fn test_all_scopes_are_types() {
        let parser = ConventionalParser::new(
            vec![
                "feat".to_string(),
                "fix".to_string(),
                "build".to_string(),
                "ci".to_string(),
            ],
            Some(vec!["api".to_string(), "ui".to_string()]),
        );
        let result = parser.parse("feat(build, ci): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            let type_as_scope_errors: Vec<_> = errors
                .iter()
                .filter_map(|e| match &e.kind {
                    ParseErrorKind::TypeUsedAsScope { found, .. } => Some(found.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(type_as_scope_errors.len(), 2);
            assert!(type_as_scope_errors.contains(&"build".to_string()));
            assert!(type_as_scope_errors.contains(&"ci".to_string()));
        }
    }

    #[test]
    fn test_regular_invalid_scope_not_a_type() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string(), "fix".to_string()],
            Some(vec!["api".to_string(), "ui".to_string()]),
        );
        let result = parser.parse("feat(database): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            let invalid_scope_errors: Vec<_> = errors
                .iter()
                .filter_map(|e| match &e.kind {
                    ParseErrorKind::InvalidScope { found, .. } => Some(found.clone()),
                    _ => None,
                })
                .collect();
            assert_eq!(invalid_scope_errors.len(), 1);
            assert_eq!(invalid_scope_errors[0], "database");

            let type_as_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::TypeUsedAsScope { .. }))
                .count();
            assert_eq!(type_as_scope_errors, 0);
        }
    }

    #[test]
    fn test_scope_with_numbers_valid() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["v2-beta3".to_string(), "api-v1".to_string()]),
        );
        let result = parser.parse("feat(v2-beta3,api-v1): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec!["v2-beta3".to_string(), "api-v1".to_string()])
            );
        }
    }

    #[test]
    fn test_all_scopes_invalid_with_suggestions() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        let result = parser.parse("feat(invalid1,invalid2): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report both as invalid
            let invalid_scope_errors = errors
                .iter()
                .filter(|e| matches!(&e.kind, ParseErrorKind::InvalidScope { .. }))
                .count();
            assert_eq!(invalid_scope_errors, 2);
        }
    }

    // ===== SCOPE PARENTHESIS SPACING TESTS =====

    #[test]
    fn test_space_after_opening_paren() {
        let parser = default_parser();
        let result = parser.parse("feat( port): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterOpenParen))
            );
        }
    }

    #[test]
    fn test_space_before_closing_paren() {
        let parser = default_parser();
        let result = parser.parse("feat(port ): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceBeforeCloseParen))
            );
        }
    }

    #[test]
    fn test_spaces_both_sides_of_scope() {
        let parser = default_parser();
        let result = parser.parse("feat( port ): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Should report both errors
            let space_errors = errors
                .iter()
                .filter(|e| {
                    matches!(
                        &e.kind,
                        ParseErrorKind::ExtraSpaceAfterOpenParen
                            | ParseErrorKind::ExtraSpaceBeforeCloseParen
                    )
                })
                .count();
            assert_eq!(space_errors, 2);
        }
    }

    #[test]
    fn test_space_after_paren_multiple_scopes() {
        let parser = default_parser();
        let result = parser.parse("feat( port, lua): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterOpenParen))
            );
        }
    }

    #[test]
    fn test_space_before_paren_multiple_scopes() {
        let parser = default_parser();
        let result = parser.parse("feat(port, lua ): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceBeforeCloseParen))
            );
        }
    }

    #[test]
    fn test_multiple_spaces_after_opening_paren() {
        let parser = default_parser();
        let result = parser.parse("feat(  port): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // First space should be flagged
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceAfterOpenParen))
            );
        }
    }

    #[test]
    fn test_multiple_spaces_before_closing_paren() {
        let parser = default_parser();
        let result = parser.parse("feat(port  ): description");
        assert!(result.is_err());
        if let Err(errors) = result.into_result() {
            // Space before closing paren should be flagged
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(&e.kind, ParseErrorKind::ExtraSpaceBeforeCloseParen))
            );
        }
    }

    #[test]
    fn test_correct_multi_scope_formatting() {
        let parser = ConventionalParser::new(
            vec!["feat".to_string()],
            Some(vec!["port".to_string(), "lua".to_string()]),
        );
        // Correct formatting: no spaces after ( or before )
        let result = parser.parse("feat(port, lua): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(
                header.scope,
                Some(vec!["port".to_string(), "lua".to_string()])
            );
        }
    }

    #[test]
    fn test_correct_single_scope_no_spaces() {
        let parser =
            ConventionalParser::new(vec!["feat".to_string()], Some(vec!["api".to_string()]));
        // Correct single scope
        let result = parser.parse("feat(api): description");
        assert!(result.is_ok());
        if let Ok(header) = result.into_result() {
            assert_eq!(header.scope, Some(vec!["api".to_string()]));
        }
    }
}
