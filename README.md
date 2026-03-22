# conventional-prs

Validates PR titles against [Conventional Commits](https://www.conventionalcommits.org/). Posts error comments on PRs.

## CLI Usage

```bash
cargo install --git https://github.com/scarf005/conventional-prs
conventional-prs --input "feat: add feature"

conventional-prs --input ' fet : foo '
Error: Invalid commit message format
   ,-[ input:1:1 ]
   |
 1 |  fet : foo_
   | ||        |  
   | `------------ type cannot be empty (#1)
   | ||        |  
   | `------------ expected space here (#3)
   |  |        |  
   |  `----------- expected ':' here (#2)
   |           |  
   |           `-- trailing whitespace (#4)
   | 
   | Help 1: Add a commit type (e.g., 'feat', 'fix')
   | 
   | Help 2: Add a colon ':' after the type/scope, followed by a space
   | 
   | Help 3: Add a space after the colon, before the description
   | 
   | Help 4: Remove trailing spaces from the end of the commit message
---'
```

## GitHub Action

Add to `.github/workflows/pr-validation.yml`:

```yaml
name: PR Validation
on:
  # Use pull_request_target so the action can post PR comments on fork PRs.
  # Security note: do not check out or execute untrusted PR code in the same job when using pull_request_target.
  pull_request_target:
    types: [opened, edited, reopened]

jobs:
  validate:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      issues: write
    steps:
      - uses: scarf005/conventional-prs@main
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

Uses a JavaScript action backed by the bundled WASM validator.

## Benchmarks

```bash
cargo install hyperfine
deno task bench
```

Writes `target/benchmark/cli-vs-wasm-cold.{md,json}`.

Custom:

```bash
deno task build
BENCH_INPUT='feat(api): add recovery' BENCH_CONFIG_PATH='.github/semantic.yml' BENCH_RUNS=30 BENCH_WARMUP=5 ./scripts/benchmark-cli-vs-wasm.sh
```

## TypeScript bindings (JSR)

TypeScript and JSR usage is documented in `README.typescript.md`.

When publishing to JSR, the TypeScript README is shipped as the package README.

## Local Git Hooks (prek)

Use `prek` for commit-title validation via `commit-msg` hooks:

```bash
# Install prek (pick one)
brew install prek
# or: cargo install --locked prek

# Install repo hooks from prek.toml
prek install -f --hook-type commit-msg
```

Hook behavior:

- Reads only the commit title (first line).
- Validates title with `conventional-prs` and `.github/semantic.yml`.
- Rejects invalid titles before commit is created.

## Rust Library

```toml
[dependencies]
conventional-prs = { git = "https://github.com/scarf005/conventional-prs" }
```

```rust
use conventional_prs::{ConventionalParser, OutputFormat};

let parser = ConventionalParser::new(
    vec!["feat".into(), "fix".into()],
    Some(vec!["api".into(), "ui".into()])
);

let result = parser.parse("feat(api): add endpoint");
if result.is_ok() {
    println!("Valid commit!");
} else {
    result.print_errors(OutputFormat::Color);
}
```

## Configuration

Optional. Create `.github/semantic.yml`:

```yaml
types: [feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]
scopes: [api, cli, cfg, ci, deps, docs, prs]
```

The scope is optional; if present, it must be one of the terse values above.

Compatible with [semantic-prs](https://github.com/Ezard/semantic-prs).

## License

AGPL-3.0-only
