use clap::Parser;
use conventional_prs::{CharSetConfig, Config, ConventionalParser, OutputFormat};
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

    /// Character set for error rendering (ascii or unicode)
    #[arg(long, value_enum)]
    charset: Option<CharSet>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Format {
    Default,
    #[value(name = "github")]
    GitHub,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum CharSet {
    Ascii,
    Unicode,
}

impl From<Format> for OutputFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Default => OutputFormat::Color,
            Format::GitHub => OutputFormat::Ascii,
        }
    }
}

impl From<CharSet> for CharSetConfig {
    fn from(c: CharSet) -> Self {
        match c {
            CharSet::Ascii => CharSetConfig::Ascii,
            CharSet::Unicode => CharSetConfig::Unicode,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let mut config = match Config::load(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {e}");
            process::exit(1);
        }
    };

    if let Some(charset) = cli.charset {
        config.charset = CharSetConfig::from(charset);
    }

    if !config.enabled {
        eprintln!("Validation is disabled in configuration");
        process::exit(0);
    }

    let input = match cli.input {
        Some(text) => text,
        None => {
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("Error reading from stdin: {e}");
                process::exit(1);
            }
            buffer.trim().to_string()
        }
    };

    let parser = ConventionalParser::new(config.types.clone(), config.scopes.clone());
    let output_format = OutputFormat::from(cli.format);

    let result = parser.parse(&input);

    if result.is_ok() {
        if output_format == OutputFormat::Ascii {
            println!("✓ Valid conventional commit");
        } else {
            eprintln!("✓ Valid conventional commit");
        }
        process::exit(0);
    } else {
        result.print_errors(output_format, config.charset);
        process::exit(1);
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
