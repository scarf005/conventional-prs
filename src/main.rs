use clap::Parser;
use conventional_prs::{Config, ConventionalParser, ErrorReporter, OutputFormat};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "conventional-prs",
    about = "A Conventional Commit Validator for PR titles and commit messages",
    version
)]
struct Cli {
    /// Path to configuration file
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Input string to validate (if not provided, reads from stdin)
    #[arg(long, value_name = "STRING")]
    input: Option<String>,

    /// Output format (default or github)
    #[arg(long, value_enum, default_value = "default")]
    format: Format,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Format {
    Default,
    #[value(name = "github")]
    GitHub,
}

impl From<Format> for OutputFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Default => OutputFormat::Color,
            Format::GitHub => OutputFormat::Ascii,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // Load configuration
    let config = match Config::load(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            process::exit(1);
        }
    };

    // Check if validation is enabled
    if !config.enabled {
        eprintln!("Validation is disabled in configuration");
        process::exit(0);
    }

    // Get input from CLI arg or stdin
    let input = match cli.input {
        Some(text) => text,
        None => {
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("Error reading from stdin: {}", e);
                process::exit(1);
            }
            buffer.trim().to_string()
        }
    };

    // Validate the input
    let parser = ConventionalParser::new(config.types.clone(), config.scopes.clone());
    let output_format = OutputFormat::from(cli.format);
    let reporter = ErrorReporter::new(output_format);

    match parser.parse(&input) {
        Ok(_header) => {
            // Valid commit message
            if output_format == OutputFormat::Ascii {
                println!("✓ Valid conventional commit");
            } else {
                eprintln!("✓ Valid conventional commit");
            }
            process::exit(0);
        }
        Err(errors) => {
            // Invalid commit message - print errors
            reporter.print_errors(&input, &errors);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;
        let _ = Cli::command();
    }
}
