# @scarf/conventional-prs (TypeScript)

Validate Conventional Commit headers with Rust/WASM and Standard Schema-compatible results.

## Install

```bash
# Deno
deno add jsr:@scarf/conventional-prs

# Bun
bunx jsr add @scarf/conventional-prs

# Node/npm
npx jsr add @scarf/conventional-prs
```

## `commitHeaderSchema(config?)`

Creates a Standard Schema validator for commit headers.

```ts
import { commitHeaderSchema } from "@scarf/conventional-prs"

const schema = commitHeaderSchema({
  types: ["feat", "fix", "chore"],
  scopes: ["api", "ui"],
})
```

### Example: `~standard.validate` (idiomatic branching)

```ts
import { commitHeaderSchema } from "@scarf/conventional-prs"

const schema = commitHeaderSchema()
const result = schema["~standard"].validate("feat(api): add endpoint")

if (result instanceof Promise) {
  throw new Error("Expected synchronous validation")
}

if (result.issues) {
  for (const issue of result.issues) {
    console.error(issue.path, issue.message)
  }
} else {
  console.log(result.value.type)
}
```

Output:

```text
feat
```

### Example: async-safe `~standard.validate`

```ts
import { commitHeaderSchema } from "@scarf/conventional-prs"

const schema = commitHeaderSchema()
let result = schema["~standard"].validate("fature: add endpoint")
if (result instanceof Promise) {
  result = await result
}

if (result.issues) {
  console.log(result.issues.map((issue) => issue.message))
}
```

Output:

```text
["Invalid commit type \"fature\". Expected one of: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert."]
```

## `safeParseCommitHeader(input, config?)`

Returns a discriminated result with parsed data or typed issues.

### Example: success path

```ts
import { safeParseCommitHeader } from "@scarf/conventional-prs"

const result = safeParseCommitHeader("feat(api): add endpoint")
if (result.success) {
  console.log(result.data)
}
```

Output:

```text
{ type: "feat", scope: ["api"], breaking: false, description: "add endpoint" }
```

### Example: failure path with issue metadata

```ts
import { safeParseCommitHeader } from "@scarf/conventional-prs"

const result = safeParseCommitHeader("fature: add endpoint")
if (!result.success) {
  for (const issue of result.issues) {
    console.log(issue.code)
    console.log(issue.path)
    console.log(issue.message)
  }
}
```

Output:

```text
invalid_type
[{ key: "type" }]
Invalid commit type "fature". Expected one of: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert.
```

## `parseCommitHeader(input, config?)`

Parses and throws when the input is invalid.

```ts
import { parseCommitHeader } from "@scarf/conventional-prs"

const header = parseCommitHeader("feat(api): add endpoint")
console.log(header.description)
```

Output:

```text
add endpoint
```

## Pretty printing

### `prettyPrintCommitHeaderValidation(input, config?)`

Validates and returns Ariadne-formatted output for invalid headers, otherwise `null`.

```ts
import { prettyPrintCommitHeaderValidation } from "@scarf/conventional-prs"

const report = prettyPrintCommitHeaderValidation("fature: add endpoint")
if (report) {
  console.log(report)
}
```

Output (contains):

```text
Error: Invalid commit type
```

### `prettyPrintCommitIssues(input, issues, config?)`

Pretty-prints issues from `safeParseCommitHeader`. For string input, this uses Ariadne output.

```ts
import { prettyPrintCommitIssues, safeParseCommitHeader } from "@scarf/conventional-prs"

const result = safeParseCommitHeader("fature: add endpoint")
if (!result.success) {
  console.log(prettyPrintCommitIssues("fature: add endpoint", result.issues))
}
```

Output (contains):

```text
Error: Invalid commit type
```

### `prettyPrintCommitHeader(input, config?)`

Compatibility alias of `prettyPrintCommitHeaderValidation`.

## Config parsing

Validation APIs accept config objects only. They do not accept raw YAML text directly.

### `parseSemanticConfig(yamlText)`

Parses `semantic.yml` text and returns a typed config object.

```ts
import { parseSemanticConfig, safeParseCommitHeader } from "@scarf/conventional-prs"

const yamlText = await Deno.readTextFile(".github/semantic.yml")
const config = parseSemanticConfig(yamlText)
const result = safeParseCommitHeader("feat(api): add endpoint", config)
```

Output:

```text
result.success === true
```

### `safeParseSemanticConfig(yamlText)`

Returns parse result without throwing.

```ts
import { safeParseSemanticConfig } from "@scarf/conventional-prs"

const configResult = safeParseSemanticConfig("types: [feat\n")
if (!configResult.ok) {
  console.error(configResult.configError)
}
```

Output (starts with):

```text
Invalid semantic config:
```

## Browser usage

Use a pinned version URL:

```ts
import { commitHeaderSchema } from "https://esm.sh/jsr/@scarf/conventional-prs@0.3.1"
```
