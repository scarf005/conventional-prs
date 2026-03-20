import { assertEquals } from "jsr:@std/assert/equals"
import { assertSnapshot } from "jsr:@std/testing/snapshot"

import * as pr from "../mod.ts"

Deno.test("config() returns strict standard-schema success object", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = schema["~standard"].validate("feat(api): add endpoint")

  if (result instanceof Promise) {
    throw new Error("expected sync result")
  }

  assertEquals(result, {
    value: {
      type: "feat",
      scope: "api",
      subject: "add endpoint",
      merge: null,
      header: "feat(api): add endpoint",
      body: null,
      footer: null,
      notes: [],
      references: [],
      mentions: [],
      revert: null,
    },
  })
})

Deno.test("config() returns strict standard-schema failure issues", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = schema["~standard"].validate("fature(api): add endpoint")

  if (result instanceof Promise || !("issues" in result)) {
    throw new Error("expected sync failure result")
  }

  if (!result.issues || result.issues.length === 0) {
    throw new Error("expected issues")
  }

  const first = result.issues[0]
  assertEquals(Object.keys(first).sort(), ["message", "path"])
})

Deno.test("safeParse() returns conventional-commits-parser-like success object", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = pr.safeParse(schema, "feat(api): add endpoint")

  assertEquals(result, {
    success: true,
    output: {
      type: "feat",
      scope: "api",
      subject: "add endpoint",
      merge: null,
      header: "feat(api): add endpoint",
      body: null,
      footer: null,
      notes: [],
      references: [],
      mentions: [],
      revert: null,
    },
  })
})

Deno.test("safeParse() preserves comma-separated multi-scope text", () => {
  const schema = pr.config({ types: ["feat"], scopes: ["api", "ui"] })
  const result = pr.safeParse(schema, "feat(api, ui): add endpoint")

  assertEquals(result.success, true)
  if (!result.success) {
    throw new Error("expected success")
  }

  assertEquals(result.output.scope, "api, ui")
})

Deno.test("safeParse() returns valibot-esque issue array on failure", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = pr.safeParse(schema, "fature(api): add endpoint")

  if (result.success) {
    throw new Error("expected failure")
  }

  const first = result.issues[0]
  assertEquals(first.kind, "validation")
  assertEquals(first.type, "invalid_type")
  assertEquals(typeof first.input, "string")
  assertEquals(Array.isArray(first.path), true)
})

Deno.test("summarize() matches snapshot", async (t) => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = pr.safeParse(schema, "fature(api): add endpoint")
  if (result.success) {
    throw new Error("expected failure")
  }

  const report = pr.summarize(result.issues)
  await assertSnapshot(t, report)
})

Deno.test("parse() throws single-line error with verbose false", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })

  let message = ""
  try {
    pr.parse(schema, "fature(api): add endpoint", { verbose: false })
  } catch (error) {
    message = error instanceof Error ? error.message : String(error)
  }

  assertEquals(message.startsWith("type: Invalid commit type"), true)
})

Deno.test("parse() throws full pretty report by default", () => {
  const schema = pr.config({ types: ["feat", "fix"], scopes: ["api"] })

  let message = ""
  try {
    pr.parse(schema, "fature(api): add endpoint")
  } catch (error) {
    message = error instanceof Error ? error.message : String(error)
  }

  assertEquals(message.includes("Error: Invalid commit type"), true)
})

Deno.test("parseConfig() parses semantic.yml text and returns a usable schema", () => {
  const schema = pr.parseConfig(`types: ["feat", "fix"]\nscopes: ["api"]\n`)
  const result = pr.safeParse(schema, "feat(api): add endpoint")

  assertEquals(result.success, true)
  if (!result.success) {
    throw new Error("expected success")
  }
  assertEquals(result.output.type, "feat")
  assertEquals(result.output.scope, "api")
})

Deno.test("parseConfig() applies default semantic-prs settings for empty YAML", () => {
  const schema = pr.parseConfig("")
  const result = pr.safeParse(schema, "feat: add endpoint")

  assertEquals(result.success, true)
})

Deno.test("parseConfig() throws on malformed YAML", () => {
  let message = ""
  try {
    pr.parseConfig("types: [feat")
  } catch (error) {
    message = error instanceof Error ? error.message : String(error)
  }

  assertEquals(message.length > 0, true)
})
