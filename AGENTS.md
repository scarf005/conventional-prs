# AGENTS.md

- **MUST** commit after implementing a subtask
- **MUST** write tests (Unit tests for parser recovery are mandatory)
- **MUST NOT** reinvent wheel: Use existing crates from crates.io instead of implementing common algorithms (e.g., use `strsim` for string similarity, not manual Levenshtein)
- **MUST USE** named format parameters in macros like `format!`, `println!`, etc.
  - ✅ GOOD: `format!("Hello {name}")` when `name` is in scope
  - ❌ BAD: `format!("Hello {name}", name = name)` (redundant)
  - ✅ GOOD: `format!("Hello {user}", user = get_user())` when you need to call a function or use an expression
  - The named parameter is ONLY needed when the expression is not just a simple variable with the same name

## Project Overview

Build a **Conventional Commit Validator** using Rust. This tool will run as a
GitHub Action to validate Pull Request titles and commit messages.

**Key Goals:**

1. **Fault-Tolerant Parsing:** Use a parser combinator that doesn't stop at the
   first error. It should collect all errors (e.g., missing parenthesis, invalid
   type, missing space) and report them simultaneously.
2. **Rich Error Reporting:** Generate Rust-compiler-style error messages (with
   arrows, labels, and spans) using ASCII art.
3. **Compatibility:** Fully compatible with the configuration schema of
   [Ezard/semantic-prs](https://github.com/Ezard/semantic-prs).
4. **Format Support:** Support configuration files in YAML, JSON, and JSONC
   (JSON with comments).

---

## Tech Stack & Dependencies

- **Language:** Rust (Latest Stable, 2024 Edition or newer)
- **Error Reporting:** `ariadne = "0.6"`
- **CLI Args:** `clap` (with derive features)
- **Serialization:** `serde`, `serde_json`, `serde_yaml`, `toml`
- **JSONC Support:** `json_comments` (to strip comments before parsing JSON)
- **Async/Runtime:** `tokio` (only if necessary for GitHub API, otherwise
  synchronous is fine)

---

## 1. Configuration System (`config.rs`)

The tool must load configuration with the following precedence:

1. Path specified via CLI flag `--config <PATH>`.
2. `.github/semantic.yml`
3. `.github/semantic.yaml`
4. `.github/semantic.json`
5. `.github/semantic.jsonc`
6. `.github/semantic.toml`
7. `XDG_CONFIG_DIR/conventional-prs/config.toml`
8. `$HOME/.config/conventional-prs/config.toml`
9. Default values (if no file is found).

### Configuration Struct

Map the `Ezard/semantic-prs` schema to a Rust struct. Use
`#[serde(rename_all = "camelCase")]`.

| Field                  | Type                  | Default                                                                                          | Description                                   |
| :--------------------- | :-------------------- | :----------------------------------------------------------------------------------------------- | :-------------------------------------------- |
| `enabled`              | `bool`                | `true`                                                                                           | Enable/disable checks                         |
| `title_only`           | `bool`                | `false`                                                                                          | Validate PR title only                        |
| `commits_only`         | `bool`                | `false`                                                                                          | Validate commits only                         |
| `title_and_commits`    | `bool`                | `false`                                                                                          | Validate both                                 |
| `any_commit`           | `bool`                | `false`                                                                                          | If true, pass if at least one commit is valid |
| `types`                | `Vec<String>`         | `["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"]` | Allowed types                                 |
| `scopes`               | `Option<Vec<String>>` | `None` (Any)                                                                                     | Allowed scopes. If `None`, allow any.         |
| `allow_merge_commits`  | `bool`                | `false`                                                                                          | Skip validation for Merge commits             |
| `allow_revert_commits` | `bool`                | `false`                                                                                          | Skip validation for Revert commits            |
| `target_url`           | `String`              | `https://github.com/Ezard/semantic-prs`                                                          | URL for details link                          |

**Implementation Detail:**

- Create a `Config` struct.
- Implement a loader function that tries to read files in the order listed
  above.
- For JSONC/JSON, use a comment stripper (like `json_comments::StripComments`)
  before passing to `serde_json`.

---

## 2. Parsing Logic (`parser.rs`)

Implement a fault-tolerant parser that collects all errors simultaneously instead of stopping at the first error.

### Grammar Rules

Standard Conventional Commit Header: `type(scope): description` or `type: description`.
(Note: This parser **only validates the header (first line)** of the commit message).

1. **Type:**
   - Must be one of the `types` allowed in Config.
   - _Recovery:_ If an unknown type is found (e.g., "fature"), report an error
     with a custom label but continue parsing.
2. **Scope (Optional):**
   - Surrounded by `(` and `)`.
   - If `scopes` is defined in Config, the value must be in the list.
   - _Recovery:_ Handle missing closing parenthesis `)` gracefully and continue parsing.
3. **Breaking Change (`!`)**:
   - Optional `!` before the `:`.
4. **Separator:**
   - Must be `:` (colon followed by space).
   - _Recovery:_ Handle missing space (e.g., `feat:msg`) or missing colon.
5. **Description:**
   - Rest of the line. Must not be empty.

### Error Handling Strategy

- Collect all errors encountered during parsing instead of stopping at the first one.
- Track error location and type (missing token, invalid type, invalid scope, etc.).
- **Tests:** Create tests for each recovery scenario (e.g., "test_missing_scope_parenthesis", "test_invalid_type_but_valid_rest").

---

## 3. Reporting (`report.rs`)

Use **Ariadne (v0.6)** to visualize errors.

### Output Requirements

1. **Standard Output (Human/Local):**
   - Enable colors (`Config::default().with_color(true)`).
   - Print to `stderr`.
2. **GitHub Action Output (`--format github`):**
   - **CRITICAL:** Disable colors (`Config::default().with_color(false)`).
   - Generate a clean ASCII report suitable for a Markdown code block.
   - Print to `stdout` so it can be captured by the GitHub Action workflow.

---

## 4. Main Application Flow (`main.rs`)

1. **Parse Arguments:**
   - `--config <PATH>`
   - `--input <STRING>` (or stdin)
   - `--format <default|github>` (Controls output style)
2. **Load Config:** Read from file or defaults.
3. **Get Input:** Read the commit message/PR title.
4. **Validate:**
   - Parse and validate the commit message against the grammar rules.
   - Check business logic (e.g., allowed scopes).
5. **Output:**
   - If valid: Exit code `0`.
   - If invalid:
     - If `--format github`: Print color-less ASCII report to `stdout` (for capturing).
     - Else: Print colored report to `stderr` (for logs).
     - Exit code `1`.

---

## Example Usage

**Input:**

```bash
./validator --input "fature(api) fixed login" --format github
```

**Expected Output (Stdout):**

```text
Error: Invalid commit type
   ╭─[input:1:1]
   │
 1 │ fature(api) fixed login
   │ ──────
   │    ╰── expected one of "feat", "fix", ...

Error: Missing separator
   ╭─[input:1:12]
   │
 1 │ fature(api) fixed login
   │            │
   │            ╰── expected ': ' here
```
