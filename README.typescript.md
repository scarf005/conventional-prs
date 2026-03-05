# @scarf/conventional-prs (TypeScript)

Simplified PR-title API with strict Standard Schema validation output.

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
import * as pr from "@scarf/conventional-prs"

const schema = pr.config({
  types: ["feat", "fix"],
  scopes: ["api"],
})

const strict = schema["~standard"].validate("feature: foo")
```

### `config(config?)`

Creates a schema object. `schema["~standard"].validate(...)` strictly returns `{ value } | { issues }`.

### `parse(schema, input, options?)`

Parses a PR title and returns a conventional-commits-parser style object, or throws.

- `options.verbose` defaults to `true`
- `verbose: false` throws a single-line error message
- `verbose: true` throws an Ariadne-formatted multi-line error report

### `safeParse(schema, input)`

Safe variant of `parse`.

- Success: `{ success: true, output }`
- Failure: `{ success: false, issues }`

Failure `issues` are valibot-esque and include:

- `kind`, `type`, `input`
- `message`, `path`
- `expected`, `received`
- `span` (when parser span exists)

### `summarize(issues)`

Converts `safeParse(...).issues` into a full Ariadne-style report string.
