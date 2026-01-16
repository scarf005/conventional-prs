use crate::config::CharSetConfig;
use crate::parser::{ParseError, ParseErrorKind};
use ariadne::{CharSet, ColorGenerator, Label, Report, ReportKind, Source};
use strsim::jaro_winkler;

/// Find the most similar string from a list using Jaro-Winkler similarity
fn find_similar(target: &str, candidates: &[String]) -> Option<String> {
    candidates
        .iter()
        .map(|candidate| (candidate, jaro_winkler(target, candidate)))
        .filter(|(_, similarity)| *similarity > 0.8) // High similarity threshold
        .max_by(|(_, sim1), (_, sim2)| sim1.partial_cmp(sim2).unwrap())
        .map(|(candidate, _)| candidate.clone())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Color, // Colored for terminal
    Ascii, // Plain ASCII for GitHub Actions
}

pub struct ErrorReporter {
    format: OutputFormat,
    charset: CharSetConfig,
}

impl ErrorReporter {
    pub fn new(format: OutputFormat, charset: CharSetConfig) -> Self {
        Self { format, charset }
    }

    /// Generate error report and return it as a String
    pub fn report_errors(&self, input: &str, errors: &[ParseError]) -> String {
        let mut output = Vec::new();

        // Replace problematic spaces with visible character based on error spans
        let display_input = self.visualize_spacing_errors(input, errors);

        let source = Source::from(&display_input);

        // Group related errors together
        let error_groups = self.group_errors(errors);

        for group in error_groups {
            let report = if group.len() == 1 {
                self.build_report(&group[0])
            } else {
                self.build_combined_report(&group)
            };

            // Write to buffer
            report
                .write(("input", source.clone()), &mut output)
                .unwrap_or_else(|e| eprintln!("Failed to write report: {}", e));
        }

        let rendered =
            String::from_utf8(output).unwrap_or_else(|_| "Error generating report".to_string());

        // Post-process to use custom characters for ranges vs points
        self.customize_underlines(rendered, errors)
    }

    /// Customize underline characters to distinguish ranges from single points
    fn customize_underlines(&self, output: String, _errors: &[ParseError]) -> String {
        if self.charset == CharSetConfig::Ascii {
            return output;
        }

        let lines: Vec<&str> = output.lines().collect();
        let mut result_lines = Vec::new();

        for line in &lines {
            let stripped = Self::strip_ansi(line);

            if stripped.contains('┬') || stripped.contains('─') {
                let customized = self.customize_with_ansi_preserved(line, &stripped);
                result_lines.push(customized);
            } else {
                result_lines.push(line.to_string());
            }
        }

        result_lines.join("\n") + "\n"
    }

    /// Customize underline characters while preserving ANSI escape codes
    fn customize_with_ansi_preserved(&self, original: &str, stripped: &str) -> String {
        // First, customize the stripped version to know what characters to output
        let customized_stripped = self.customize_underline_chars(stripped);

        // Build a mapping of stripped position to customized character
        let customized_chars: Vec<char> = customized_stripped.chars().collect();

        // Now rebuild the string with ANSI codes preserved
        let mut result = String::new();
        let mut stripped_idx = 0;
        let mut chars = original.chars();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Copy escape sequence as-is
                result.push(ch);
                if chars.next() == Some('[') {
                    result.push('[');
                    // Copy until we find the command letter
                    for ch in chars.by_ref() {
                        result.push(ch);
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                // This is a visible character - use the customized version
                if stripped_idx < customized_chars.len() {
                    result.push(customized_chars[stripped_idx]);
                    stripped_idx += 1;
                } else {
                    result.push(ch);
                }
            }
        }

        result
    }

    /// Simple ANSI escape code stripper
    fn strip_ansi(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip escape sequence
                if chars.next() == Some('[') {
                    // Skip until we find a letter (the command)
                    for ch in chars.by_ref() {
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Customize underline characters: ranges get ╰─╯ boundaries, points get ╿
    fn customize_underline_chars(&self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut result = chars.clone();

        // Process each ┬ to determine if it's part of a range or a standalone point
        for i in 0..chars.len() {
            if chars[i] == '┬' {
                // Check if this connector has dashes immediately around it
                let has_dash_before = i > 0 && chars[i - 1] == '─';
                let has_dash_after = i + 1 < chars.len() && chars[i + 1] == '─';

                // Standalone point: missing dash on at least one side
                if !has_dash_before || !has_dash_after {
                    result[i] = '╿';
                }
            }
        }

        // Now find range boundaries (first and last dash in each continuous dash-connector sequence)
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '─' {
                // Start of a potential range
                let start = i;
                let mut end = i;

                // Extend through dashes and connectors
                while end + 1 < chars.len() && (chars[end + 1] == '─' || chars[end + 1] == '┬')
                {
                    end += 1;
                }

                // Only process as range if we moved past the start AND have at least one connector
                if end > start {
                    let has_connector = (start..=end).any(|idx| chars[idx] == '┬');
                    if has_connector {
                        // Find the last dash in this sequence
                        if let Some(last_dash) = (start..=end).rev().find(|&idx| chars[idx] == '─')
                        {
                            result[start] = '╰';
                            if last_dash != start {
                                result[last_dash] = '╯';
                            }
                        }
                    }
                }

                i = end + 1;
            } else {
                i += 1;
            }
        }

        result.iter().collect()
    }

    /// Replace spaces in error spans with visible character
    fn visualize_spacing_errors(&self, input: &str, errors: &[ParseError]) -> String {
        let mut chars: Vec<char> = input.chars().collect();
        let space_char = if self.charset == CharSetConfig::Ascii {
            '_'
        } else {
            '␣'
        };

        for error in errors {
            if matches!(
                error.kind,
                ParseErrorKind::ExtraSpaceBeforeColon
                    | ParseErrorKind::ExtraSpaceAfterColon
                    | ParseErrorKind::TrailingSpaces
            ) {
                for i in error.span.clone() {
                    if i < chars.len() && chars[i] == ' ' {
                        chars[i] = space_char;
                    }
                }
            }
        }

        chars.iter().collect()
    }

    fn build_report(
        &self,
        error: &ParseError,
    ) -> Report<'static, (&'static str, std::ops::Range<usize>)> {
        let mut colors = ColorGenerator::new();
        let error_color = if self.format == OutputFormat::Ascii {
            None
        } else {
            Some(colors.next())
        };

        let (message, label_text, help_text) = match &error.kind {
            ParseErrorKind::InvalidType { found, expected } => {
                let msg = format!("Invalid commit type '{found}'");
                let label = format!("'{found}' is not a valid type");

                // Find similar type for suggestion
                let suggestion = find_similar(found, expected);
                let valid_types = expected.join(", ");

                let help = if let Some(suggestion) = suggestion {
                    format!("Did you mean '{suggestion}'?\nValid types: {valid_types}")
                } else {
                    format!("Valid types: {valid_types}")
                };
                (msg, label, Some(help))
            }
            ParseErrorKind::InvalidScope { found, expected } => {
                let msg = format!("Invalid scope '{found}'");
                let label = format!("'{found}' is not a valid scope");

                // Find similar scope for suggestion
                let suggestion = find_similar(found, expected);
                let valid_scopes = expected.join(", ");

                let help = if let Some(suggestion) = suggestion {
                    format!("Did you mean '{suggestion}'?\nValid scopes: {valid_scopes}")
                } else {
                    format!("Valid scopes: {valid_scopes}")
                };
                (msg, label, Some(help))
            }
            ParseErrorKind::MissingClosingParen => (
                "Missing closing parenthesis".to_string(),
                "expected ')' here".to_string(),
                Some("Add a closing ')' after the scope".to_string()),
            ),
            ParseErrorKind::MissingSeparator => (
                "Missing separator".to_string(),
                "expected ': ' here".to_string(),
                Some("Add a colon followed by a space ': '".to_string()),
            ),
            ParseErrorKind::MissingDescription => (
                "Missing description".to_string(),
                "description is required".to_string(),
                Some("Add a description after the colon".to_string()),
            ),
            ParseErrorKind::EmptyType => (
                "Empty type".to_string(),
                "type cannot be empty".to_string(),
                Some("Add a commit type (e.g., 'feat', 'fix')".to_string()),
            ),
            ParseErrorKind::EmptyScope => (
                "Empty scope".to_string(),
                "scope cannot be empty".to_string(),
                Some("Either remove the parentheses or add a scope inside them".to_string()),
            ),
            ParseErrorKind::UnexpectedChar(c) => (
                format!("Unexpected character '{}'", c),
                "unexpected character".to_string(),
                None,
            ),
            ParseErrorKind::GenericParseError(msg) => (
                msg.clone(),
                "parse error".to_string(),
                Some(
                    "Ensure your commit message follows the format: type(scope): description"
                        .to_string(),
                ),
            ),
            ParseErrorKind::ExtraSpaceBeforeColon => (
                "Extra space found between type and colon".to_string(),
                "extra space found here".to_string(),
                Some("Remove spaces between the type/scope and the colon".to_string()),
            ),
            ParseErrorKind::ExtraSpaceAfterColon => (
                "Extra spaces after colon".to_string(),
                "too many spaces".to_string(),
                Some("Use exactly one space after the colon".to_string()),
            ),
            ParseErrorKind::MissingColon => (
                "Missing colon separator".to_string(),
                "expected ':' here".to_string(),
                Some("Add a colon ':' after the type/scope, followed by a space".to_string()),
            ),
            ParseErrorKind::MissingSpace => (
                "Missing space after colon".to_string(),
                "expected space here".to_string(),
                Some("Add a space after the colon, before the description".to_string()),
            ),
            ParseErrorKind::TrailingSpaces => (
                "Trailing spaces at end of commit message".to_string(),
                "trailing whitespace".to_string(),
                Some("Remove trailing spaces from the end of the commit message".to_string()),
            ),
            ParseErrorKind::ExtraSpaceAfterOpenParen => (
                "Extra space after opening parenthesis".to_string(),
                "unexpected space here".to_string(),
                Some("Remove the space immediately after '('".to_string()),
            ),
            ParseErrorKind::ExtraSpaceBeforeCloseParen => (
                "Extra space before closing parenthesis".to_string(),
                "unexpected space here".to_string(),
                Some("Remove the space immediately before ')'".to_string()),
            ),
        };

        let mut label = Label::new(("input", error.span.clone())).with_message(label_text);

        if let Some(color) = error_color {
            label = label.with_color(color);
        }

        let mut report_builder = Report::build(ReportKind::Error, ("input", error.span.clone()))
            .with_message(&message)
            .with_label(label);

        if let Some(help) = help_text {
            report_builder = report_builder.with_help(help);
        }

        if self.format == OutputFormat::Ascii {
            report_builder = report_builder.with_config(
                ariadne::Config::default()
                    .with_color(false)
                    .with_char_set(CharSet::from(self.charset)),
            );
        } else {
            report_builder = report_builder
                .with_config(ariadne::Config::default().with_char_set(CharSet::from(self.charset)));
        }

        report_builder.finish()
    }

    /// Group related errors that should be displayed together
    fn group_errors(&self, errors: &[ParseError]) -> Vec<Vec<ParseError>> {
        // Group all errors into a single report for less verbosity
        if errors.is_empty() {
            vec![]
        } else {
            vec![errors.to_vec()]
        }
    }

    /// Build a combined report for multiple related errors
    fn build_combined_report(
        &self,
        errors: &[ParseError],
    ) -> Report<'static, (&'static str, std::ops::Range<usize>)> {
        let mut colors = ColorGenerator::new();

        // Use the first error's span as the main report span
        let main_span = errors[0].span.clone();

        // Determine the overall message based on error types
        let message = if errors.len() == 1 {
            let (msg, _, _) = self.get_error_details(&errors[0].kind);
            msg
        } else {
            "Invalid commit message format".to_string()
        };

        let mut report_builder =
            Report::build(ReportKind::Error, ("input", main_span)).with_message(message);

        // Add a label for each error
        for (idx, error) in errors.iter().enumerate() {
            let error_color = if self.format == OutputFormat::Ascii {
                None
            } else {
                Some(colors.next())
            };

            let (_msg, label_text, help_text) = self.get_error_details(&error.kind);
            let label_with_num = format!("{label_text} (#{num})", num = idx + 1);

            let mut label = Label::new(("input", error.span.clone())).with_message(label_with_num);

            if let Some(color) = error_color {
                label = label.with_color(color);
            }

            report_builder = report_builder.with_label(label);

            if let Some(help) = help_text {
                report_builder = report_builder.with_help(help);
            }
        }

        if self.format == OutputFormat::Ascii {
            report_builder = report_builder.with_config(
                ariadne::Config::default()
                    .with_color(false)
                    .with_char_set(CharSet::from(self.charset)),
            );
        } else {
            report_builder = report_builder
                .with_config(ariadne::Config::default().with_char_set(CharSet::from(self.charset)));
        }

        report_builder.finish()
    }

    /// Extract error details (message, label, help) for a given error kind
    fn get_error_details(&self, kind: &ParseErrorKind) -> (String, String, Option<String>) {
        match kind {
            ParseErrorKind::InvalidType { found, expected } => {
                let msg = format!("Invalid commit type '{found}'");
                let label = format!("'{found}' is not a valid type");

                // Find similar type for suggestion
                let suggestion = find_similar(found, expected);
                let valid_types = expected.join(", ");

                let help = if let Some(suggestion) = suggestion {
                    format!("Did you mean '{suggestion}'?\nValid types: {valid_types}")
                } else {
                    format!("Valid types: {valid_types}")
                };
                (msg, label, Some(help))
            }
            ParseErrorKind::InvalidScope { found, expected } => {
                let msg = format!("Invalid scope '{found}'");
                let label = format!("'{found}' is not a valid scope");

                // Find similar scope for suggestion
                let suggestion = find_similar(found, expected);
                let valid_scopes = expected.join(", ");

                let help = if let Some(suggestion) = suggestion {
                    format!("Did you mean '{suggestion}'?\nValid scopes: {valid_scopes}")
                } else {
                    format!("Valid scopes: {valid_scopes}")
                };
                (msg, label, Some(help))
            }
            ParseErrorKind::MissingClosingParen => (
                "Missing closing parenthesis".to_string(),
                "expected ')' here".to_string(),
                Some("Add a closing ')' after the scope".to_string()),
            ),
            ParseErrorKind::MissingSeparator => (
                "Missing separator".to_string(),
                "expected ': ' here".to_string(),
                Some("Add a colon followed by a space ': '".to_string()),
            ),
            ParseErrorKind::MissingDescription => (
                "Missing description".to_string(),
                "description is required".to_string(),
                Some("Add a description after the colon".to_string()),
            ),
            ParseErrorKind::EmptyType => (
                "Empty type".to_string(),
                "type cannot be empty".to_string(),
                Some("Add a commit type (e.g., 'feat', 'fix')".to_string()),
            ),
            ParseErrorKind::EmptyScope => (
                "Empty scope".to_string(),
                "scope cannot be empty".to_string(),
                Some("Either remove the parentheses or add a scope inside them".to_string()),
            ),
            ParseErrorKind::UnexpectedChar(c) => (
                format!("Unexpected character '{c}'"),
                "unexpected character".to_string(),
                None,
            ),
            ParseErrorKind::GenericParseError(msg) => (
                msg.clone(),
                "parse error".to_string(),
                Some(
                    "Ensure your commit message follows the format: type(scope): description"
                        .to_string(),
                ),
            ),
            ParseErrorKind::ExtraSpaceBeforeColon => (
                "Extra space found between type and colon".to_string(),
                "extra space found here".to_string(),
                Some("Remove spaces between the type/scope and the colon".to_string()),
            ),
            ParseErrorKind::ExtraSpaceAfterColon => (
                "Extra spaces after colon".to_string(),
                "too many spaces".to_string(),
                Some("Use exactly one space after the colon".to_string()),
            ),
            ParseErrorKind::MissingColon => (
                "Missing colon separator".to_string(),
                "expected ':' here".to_string(),
                Some("Add a colon ':' after the type/scope, followed by a space".to_string()),
            ),
            ParseErrorKind::MissingSpace => (
                "Missing space after colon".to_string(),
                "expected space here".to_string(),
                Some("Add a space after the colon, before the description".to_string()),
            ),
            ParseErrorKind::TrailingSpaces => (
                "Trailing spaces at end of commit message".to_string(),
                "trailing whitespace".to_string(),
                Some("Remove trailing spaces from the end of the commit message".to_string()),
            ),
            ParseErrorKind::ExtraSpaceAfterOpenParen => (
                "Extra space after opening parenthesis".to_string(),
                "unexpected space here".to_string(),
                Some("Remove the space immediately after '('".to_string()),
            ),
            ParseErrorKind::ExtraSpaceBeforeCloseParen => (
                "Extra space before closing parenthesis".to_string(),
                "unexpected space here".to_string(),
                Some("Remove the space immediately before ')'".to_string()),
            ),
        }
    }

    /// Print errors to stderr (for terminal) or stdout (for GitHub)
    pub fn print_errors(&self, input: &str, errors: &[ParseError]) {
        let report = self.report_errors(input, errors);

        if self.format == OutputFormat::Ascii {
            // Print to stdout for GitHub Actions to capture
            print!("{}", report);
        } else {
            // Print to stderr for terminal
            eprint!("{}", report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseErrorKind;

    #[test]
    fn test_report_invalid_type() {
        let reporter = ErrorReporter::new(OutputFormat::Color, CharSetConfig::Unicode);
        let error = ParseError::new(
            ParseErrorKind::InvalidType {
                found: "fature".to_string(),
                expected: vec!["feat".to_string(), "fix".to_string()],
            },
            0..6,
        );

        let input = "fature: description";
        let report = reporter.report_errors(input, &[error]);

        assert!(report.contains("Invalid commit type"));
        assert!(report.contains("fature"));
    }

    #[test]
    fn test_report_invalid_scope() {
        let reporter = ErrorReporter::new(OutputFormat::Color, CharSetConfig::Unicode);
        let error = ParseError::new(
            ParseErrorKind::InvalidScope {
                found: "wrong".to_string(),
                expected: vec!["api".to_string(), "ui".to_string()],
            },
            5..10,
        );

        let input = "feat(wrong): description";
        let report = reporter.report_errors(input, &[error]);

        assert!(report.contains("Invalid scope"));
        assert!(report.contains("wrong"));
    }

    #[test]
    fn test_github_format_no_colors() {
        let reporter = ErrorReporter::new(OutputFormat::Ascii, CharSetConfig::Ascii);
        let error = ParseError::new(ParseErrorKind::MissingSeparator, 4..4);

        let input = "feat description";
        let report = reporter.report_errors(input, &[error]);

        assert!(!report.contains("\x1b["));
        assert!(report.contains("Missing separator"));
    }

    #[test]
    fn test_ascii_charset_uses_ascii_chars() {
        let reporter = ErrorReporter::new(OutputFormat::Ascii, CharSetConfig::Ascii);
        let error = ParseError::new(ParseErrorKind::MissingSeparator, 4..4);
        let input = "feat description";
        let report = reporter.report_errors(input, &[error]);

        assert!(!report.contains("─"));
        assert!(!report.contains("│"));
        assert!(!report.contains("╭"));
        assert!(report.contains("|") || report.contains("-"));
    }

    #[test]
    fn test_unicode_charset_uses_unicode_chars() {
        let reporter = ErrorReporter::new(OutputFormat::Ascii, CharSetConfig::Unicode);
        let error = ParseError::new(ParseErrorKind::MissingSeparator, 4..4);
        let input = "feat description";
        let report = reporter.report_errors(input, &[error]);

        assert!(report.contains("─") || report.contains("│"));
    }

    #[test]
    fn test_ascii_charset_uses_underscore_for_spaces() {
        let reporter = ErrorReporter::new(OutputFormat::Ascii, CharSetConfig::Ascii);
        let error = ParseError::new(ParseErrorKind::TrailingSpaces, 4..6);
        let input = "feat  ";
        let report = reporter.report_errors(input, &[error]);

        assert!(report.contains("feat__"));
    }

    #[test]
    fn test_unicode_charset_uses_visible_space_char() {
        let reporter = ErrorReporter::new(OutputFormat::Ascii, CharSetConfig::Unicode);
        let error = ParseError::new(ParseErrorKind::TrailingSpaces, 4..6);
        let input = "feat  ";
        let report = reporter.report_errors(input, &[error]);

        assert!(report.contains("feat␣␣"));
    }
}
