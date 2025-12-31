# Conventional Commit Validator

A Rust-based tool to validate Pull Request titles and commit messages according to the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Features

- **Fault-Tolerant Parsing**: Uses Chumsky parser combinator to collect all errors at once, not just the first error
- **Rich Error Reporting**: Generates Rust-compiler-style error messages with ASCII art using Ariadne
- **Multiple Format Support**: Supports YAML, JSON, JSONC (JSON with comments), and TOML configuration files
- **Compatible**: Fully compatible with [Ezard/semantic-prs](https://github.com/Ezard/semantic-prs) configuration schema
- **GitHub Actions Ready**: Special output format for GitHub Actions with no ANSI colors

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Basic Usage

Validate a commit message from command line:

```bash
conventional-prs --input "feat: add new feature"
```

Validate from stdin:

```bash
echo "fix(api): resolve bug" | conventional-prs
```

### With Custom Configuration

```bash
conventional-prs --input "feat: add feature" --config .github/semantic.yml
```

### GitHub Actions Format

For use in GitHub Actions (outputs plain ASCII without colors):

```bash
conventional-prs --input "feat: add feature" --format github
```

## GitHub Actions Usage

### Standard Usage (Recommended)

Add this action to your workflow. GitHub will build the Docker container from the repository:

```yaml
name: Validate Conventional Commits

on:
  pull_request:
    types: [opened, edited, synchronize, reopened]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Validate PR Title
        uses: scarf005/conventional-prs@v0.1.0
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          validate_pr_title: 'true'
```

**Inputs:**
- `github_token`: GitHub token (required, defaults to `${{ github.token }}`)
- `config`: Path to custom config file (optional)
- `validate_pr_title`: Validate PR title (default: `'true'`)
- `validate_commits`: Validate all commits (default: `'false'`)
- `validate_both`: Validate both title and commits (default: `'false'`)

### Advanced: Pre-built Container Images

For faster execution, you can use pre-built images from GitHub Container Registry:

```yaml
- name: Validate PR Title
  uses: docker://ghcr.io/scarf005/conventional-prs:v0.1.0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    INPUT_VALIDATE_PR_TITLE: 'true'
```

**Note:** Using pre-built images skips the build step but requires manually managing environment variables with the `INPUT_*` prefix.

## Configuration

The tool loads configuration from the following locations (in order of precedence):

1. Path specified via `--config` flag
2. `.github/semantic.yml`
3. `.github/semantic.yaml`
4. `.github/semantic.json`
5. `.github/semantic.jsonc`
6. `.github/semantic.toml`
7. `$XDG_CONFIG_DIR/conventional-prs/config.toml`
8. `$HOME/.config/conventional-prs/config.toml`
9. Default values

### Configuration Options

| Field                  | Type            | Default                                                                                          | Description                                   |
| :--------------------- | :-------------- | :----------------------------------------------------------------------------------------------- | :-------------------------------------------- |
| `enabled`              | `bool`          | `true`                                                                                           | Enable/disable checks                         |
| `titleOnly`            | `bool`          | `false`                                                                                          | Validate PR title only                        |
| `commitsOnly`          | `bool`          | `false`                                                                                          | Validate commits only                         |
| `titleAndCommits`      | `bool`          | `false`                                                                                          | Validate both                                 |
| `anyCommit`            | `bool`          | `false`                                                                                          | If true, pass if at least one commit is valid |
| `types`                | `Vec<String>`   | `["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"]` | Allowed types                                 |
| `scopes`               | `Option<Vec>`   | `None` (Any)                                                                                     | Allowed scopes. If `None`, allow any.         |
| `allowMergeCommits`    | `bool`          | `false`                                                                                          | Skip validation for Merge commits             |
| `allowRevertCommits`   | `bool`          | `false`                                                                                          | Skip validation for Revert commits            |
| `targetUrl`            | `String`        | `https://github.com/Ezard/semantic-prs`                                                          | URL for details link                          |

### Example Configurations

#### YAML (.github/semantic.yml)

```yaml
enabled: true
titleOnly: true
types:
  - feat
  - fix
  - docs
scopes:
  - api
  - ui
  - core
targetUrl: "https://example.com/contributing"
```

#### JSON (.github/semantic.json)

```json
{
  "enabled": true,
  "types": ["feat", "fix", "docs"],
  "scopes": ["api", "ui"],
  "allowMergeCommits": true
}
```

#### JSONC (.github/semantic.jsonc)

```jsonc
{
  // Configuration with comments
  "enabled": true,
  "types": ["feat", "fix"], // allowed types
  "scopes": ["api"] /* allowed scopes */
}
```

#### TOML (.github/semantic.toml)

```toml
enabled = true
types = ["feat", "fix", "chore"]
scopes = ["core", "api"]
allowRevertCommits = true
```

## Commit Message Format

The tool validates commit messages according to the Conventional Commits specification:

```
<type>[optional scope][optional !]: <description>
```

### Examples

Valid commit messages:

- `feat: add new login feature`
- `fix(api): resolve authentication bug`
- `docs: update README`
- `feat(ui)!: breaking change to button component`
- `chore: update dependencies`

Invalid commit messages:

- `added new feature` (missing type and colon)
- `fature: typo in type` (invalid type)
- `feat(unknown): description` (invalid scope, if scopes are restricted)
- `feat missing colon` (missing separator)

## Error Output

The tool provides detailed error messages with visual indicators:

```
Error: Invalid commit type 'fature'
   ╭─[ input:1:1 ]
   │
 1 │ fature: typo in type
   │ ───┬──  
   │    ╰──── 'fature' is not a valid type
   │ 
   │ Help: Expected one of: feat, fix, docs, style, refactor, ... (11 total)
───╯
```

## Exit Codes

- `0`: Valid commit message
- `1`: Invalid commit message or configuration error

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run with verbose output
cargo test -- --nocapture
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## License

This project is compatible with the configuration schema of [Ezard/semantic-prs](https://github.com/Ezard/semantic-prs).

## Tech Stack

- **Parser**: Chumsky v0.10 (fault-tolerant parser combinator)
- **Error Reporting**: Ariadne v0.6 (beautiful error messages)
- **CLI**: Clap v4 (command-line argument parsing)
- **Serialization**: Serde (YAML, JSON, TOML support)
- **Language**: Rust 2024 Edition
