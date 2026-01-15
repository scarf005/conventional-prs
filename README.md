# conventional-prs

Validates PR titles against [Conventional Commits](https://www.conventionalcommits.org/). Posts error comments on PRs.

## GitHub Action

Add to `.github/workflows/pr-validation.yml`:

```yaml
name: PR Validation
on:
  # Use pull_request_target so the action can post PR comments on fork PRs.
  pull_request_target:
    types: [opened, edited, synchronize, reopened]

jobs:
  validate:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
      issues: write
    steps:
      - uses: docker://ghcr.io/scarf005/conventional-prs:main
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

Uses pre-built Docker images for fast validation (no build step required).

## CLI Usage

```bash
cargo install --git https://github.com/scarf005/conventional-prs
conventional-prs --input "feat: add feature"
```

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
types: [feat, fix, docs]
scopes: [api, ui]
```

Compatible with [semantic-prs](https://github.com/Ezard/semantic-prs).

## License

AGPL-3.0-only
