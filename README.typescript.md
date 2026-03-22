# @scarf/conventional-prs (TypeScript)

Read `semantic.yml`, parse it as schema, validate a PR title, and format parser errors.

Parsed output follows `conventional-commits-parser` shape.

## Install

```bash
# Deno
deno add jsr:@scarf/conventional-prs

# Bun
bunx jsr add @scarf/conventional-prs

# Node/npm
npx jsr add @scarf/conventional-prs
```

## API

```ts
import { assertEquals } from "jsr:@std/assert/equals"
import * as pr from "@scarf/conventional-prs"

const semanticYml = [
  'types: ["feat", "fix"]',
  'scopes: ["api"]',
].join("\n")

const schema = pr.parseConfig(semanticYml)
const ok = pr.safeParse(schema, "feat(api): add schema validation")
assertEquals(ok.success, true)

const invalid = pr.safeParse(schema, "fature(web): add schema validation")
assertEquals(invalid.success, false)

if (!invalid.success) {
  console.error(pr.summarize(invalid.issues))
}
```

### `config(config?)`

Creates a schema object. `schema["~standard"].validate(...)` strictly returns `{ value } | { issues }`.

### `parse(schema, input, options?)`

Parses a PR title and returns a conventional-commits-parser style object, or throws.

- `scope` is `string | null`
- multi-scope headers are serialized as comma-separated scope text
- `options.verbose` defaults to `true`
- `verbose: false` throws a single-line error message
- `verbose: true` throws a full multi-line report

### `safeParse(schema, input)`

Safe variant of `parse`.

- Success: `{ success: true, output }`
- Failure: `{ success: false, issues }`
- `output` uses conventional-commits-parser style fields

### `parseConfig(text)`

Parses `semantic.yml` text and returns a usable schema.

### `summarize(issues)`

Converts `safeParse(...).issues` into a full report string.
