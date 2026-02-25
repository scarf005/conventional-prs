import {
  commitHeaderSchema,
  formatIssues,
  parseCommitHeader,
  safeParseCommitHeader,
} from "../mod.ts"

Deno.test("safeParseCommitHeader returns parsed header on success", () => {
  const result = safeParseCommitHeader("feat(api): add endpoint")

  if (!result.success) {
    throw new Error("expected success result")
  }

  if (result.data.type !== "feat") {
    throw new Error(`expected feat type, got ${result.data.type}`)
  }

  if (result.data.description !== "add endpoint") {
    throw new Error(`unexpected description: ${result.data.description}`)
  }
})

Deno.test("safeParseCommitHeader applies object config", () => {
  const result = safeParseCommitHeader("foo(core): works", {
    types: ["foo"],
    scopes: ["core"],
  })

  if (!result.success) {
    throw new Error("expected success result with object config")
  }

  if (result.data.type !== "foo") {
    throw new Error(`expected foo type, got ${result.data.type}`)
  }
})

Deno.test("safeParseCommitHeader returns issues on invalid header", () => {
  const result = safeParseCommitHeader("fature: add endpoint")

  if (result.success) {
    throw new Error("expected failure result")
  }

  if (result.issues.length === 0) {
    throw new Error("expected at least one issue")
  }
})

Deno.test("parseCommitHeader throws on invalid header", () => {
  let threw = false
  try {
    parseCommitHeader("fature: add endpoint")
  } catch {
    threw = true
  }

  if (!threw) {
    throw new Error("expected parseCommitHeader to throw")
  }
})

Deno.test("commitHeaderSchema exposes standard validate", () => {
  const schema = commitHeaderSchema()
  const valid = schema["~standard"].validate("feat(api): add endpoint")
  if (valid instanceof Promise) {
    throw new Error("expected sync result")
  }
  if ("issues" in valid && valid.issues) {
    throw new Error("expected no issues for valid header")
  }

  const invalid = schema["~standard"].validate("fature: add endpoint")
  if (invalid instanceof Promise) {
    throw new Error("expected sync result")
  }
  if (!("issues" in invalid) || !invalid.issues || invalid.issues.length === 0) {
    throw new Error("expected issues for invalid header")
  }
})

Deno.test("formatIssues returns non-empty text for invalid header", () => {
  const result = safeParseCommitHeader("fature: add endpoint")
  if (result.success) {
    throw new Error("expected failure result")
  }

  const formatted = formatIssues("fature: add endpoint", result.issues)
  if (formatted.length === 0) {
    throw new Error("expected formatted issue text")
  }
})

Deno.test("object-only config rejects raw yaml string", () => {
  let threw = false
  try {
    safeParseCommitHeader(
      "feat: add endpoint",
      "types: [feat]" as unknown as { types?: readonly string[] },
    )
  } catch {
    threw = true
  }

  if (!threw) {
    throw new Error("expected raw yaml config string to be rejected")
  }
})
