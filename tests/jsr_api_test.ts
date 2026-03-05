import { assertEquals } from "jsr:@std/assert/equals"
import { assertSnapshot } from "jsr:@std/testing/snapshot"

import { config, parse, safeParse, summarize } from "../mod.ts"

Deno.test("config() returns strict standard-schema success object", () => {
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })
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
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })
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
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = safeParse(schema, "feat(api): add endpoint")

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

Deno.test("safeParse() returns valibot-esque issue array on failure", () => {
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = safeParse(schema, "fature(api): add endpoint")

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
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })
  const result = safeParse(schema, "fature(api): add endpoint")
  if (result.success) {
    throw new Error("expected failure")
  }

  const report = summarize(result.issues)
  await assertSnapshot(t, report)
})

Deno.test("parse() throws single-line error with verbose false", () => {
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })

  let message = ""
  try {
    parse(schema, "fature(api): add endpoint", { verbose: false })
  } catch (error) {
    message = error instanceof Error ? error.message : String(error)
  }

  assertEquals(message.startsWith("type: Invalid commit type"), true)
})

Deno.test("parse() throws full pretty report by default", () => {
  const schema = config({ types: ["feat", "fix"], scopes: ["api"] })

  let message = ""
  try {
    parse(schema, "fature(api): add endpoint")
  } catch (error) {
    message = error instanceof Error ? error.message : String(error)
  }

  assertEquals(message.includes("Error: Invalid commit type"), true)
})
