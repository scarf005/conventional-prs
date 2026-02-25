# @scarf/conventional-prs (TypeScript)

TypeScript bindings for validating Conventional Commit headers with Rust/WASM.

## Install

```bash
# Deno
deno add jsr:@scarf/conventional-prs

# Bun
bunx jsr add @scarf/conventional-prs

# Node/npm
npx jsr add @scarf/conventional-prs
```

## Standard Schema API

```ts
import { commitHeaderSchema } from "@scarf/conventional-prs"

const schema = commitHeaderSchema()
const result = schema["~standard"].validate("feat(api): add endpoint")

if (!("issues" in result) || result.issues === undefined) {
  console.log(result.value.type)
}
```

`~standard.validate` returns:

- Success: `{ value }`
- Failure: `{ issues }`

Each issue includes a readable `message` and a `path` for structured handling.

## Convenience APIs

```ts
import {
  parseCommitHeader,
  safeParseCommitHeader,
  prettyPrintCommitHeader,
} from "@scarf/conventional-prs"

const strict = parseCommitHeader("feat(api): add endpoint")

const safe = safeParseCommitHeader("fature: add endpoint")
if (!safe.success) {
  console.log(safe.issues[0].message)
  console.log(safe.issues[0].path)
}

const report = prettyPrintCommitHeader("fature: add endpoint")
console.log(report)
```

`prettyPrintCommitHeader` uses the original Ariadne report from Rust bindings.

## Config (object-only)

Validation APIs accept `ConventionalConfig` objects only. Raw YAML is not accepted as a validator argument.

```ts
import { safeParseCommitHeader } from "@scarf/conventional-prs"

const result = safeParseCommitHeader("chore(api): release", {
  types: ["feat", "fix", "chore"],
  scopes: ["api", "ui"],
})
```

## semantic.yml loader

Use the built-in loader when you want to read `.github/semantic.yml` from disk.

```ts
import { loadSemanticConfig, safeParseCommitHeader } from "@scarf/conventional-prs"

const config = await loadSemanticConfig()
const result = safeParseCommitHeader("feat(api): add endpoint", config)
```

You can also parse YAML text directly:

```ts
import { parseSemanticConfig } from "@scarf/conventional-prs"

const config = parseSemanticConfig("types: [feat, fix]\nscopes: [api, ui]\n")
```

## Browser usage

Use a pinned version URL:

```ts
import { commitHeaderSchema } from "https://esm.sh/jsr/@scarf/conventional-prs@0.3.0"
```
