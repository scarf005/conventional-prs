# conventional-prs

Validates PR titles against [Conventional Commits](https://www.conventionalcommits.org/). Posts error comments on PRs.

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

## JSR (WASM)

This repository also publishes a WebAssembly package to JSR as `@scarf/conventional-prs`.

Build the WASM bindings with `wasmbuild`:

```bash
deno task wasmbuild
```

Install with the official JSR package flow:

```bash
# Deno
deno add jsr:@scarf/conventional-prs

# Bun
bunx jsr add @scarf/conventional-prs

# Node/npm
npx jsr add @scarf/conventional-prs
```

### Standard Schema API

```ts
import { commitHeaderSchema } from "@scarf/conventional-prs"

const schema = commitHeaderSchema()
const result = schema["~standard"].validate("feat(api): add endpoint")

if (!("issues" in result) || result.issues === undefined) {
  console.log(result.value)
}
```

Success result shape:

```ts
{
  value: {
    type: "feat",
    scope: ["api"],
    breaking: false,
    description: "add endpoint"
  }
}
```

Validation error result shape:

```ts
{
  issues: [
    {
      message: "InvalidType { found: \"fature\", expected: [\"feat\", ...] }"
    }
  ]
}
```

### Object-only config

The schema factory accepts a typed config object. Raw YAML text is not accepted by the TypeScript API.

```ts
import { safeParseCommitHeader } from "@scarf/conventional-prs"

const result = safeParseCommitHeader("chore(api): release", {
  types: ["feat", "fix", "chore"],
  scopes: ["api", "ui"],
})
```

Convenience parse API:

```ts
import { parseCommitHeader } from "@scarf/conventional-prs"

const header = parseCommitHeader("feat(api): add endpoint")
console.log(header.type)
```

### Browser usage

Use a pinned version URL to avoid CDN alias lag:

```ts
import { commitHeaderSchema } from "https://esm.sh/jsr/@scarf/conventional-prs@0.2.0"
```

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
