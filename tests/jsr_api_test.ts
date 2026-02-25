import {
  commitHeaderSchema,
  formatIssues,
  parseCommitHeader,
  parseSemanticConfig,
  prettyPrintCommitHeaderValidation,
  prettyPrintCommitIssues,
  prettyPrintCommitHeader,
  safeParseCommitHeader,
  safeParseSemanticConfig,
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

Deno.test("safeParseCommitHeader returns user-friendly issues with path", () => {
  const result = safeParseCommitHeader("fature: add endpoint")

  if (result.success) {
    throw new Error("expected failure result")
  }

  if (result.issues.length === 0) {
    throw new Error("expected at least one issue")
  }

  const first = result.issues[0]
  if (first.message.includes("InvalidType {")) {
    throw new Error(`expected friendly issue message, got: ${first.message}`)
  }

  if (!first.path || first.path.length === 0) {
    throw new Error("expected issue path")
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

  const first = invalid.issues[0]
  if (!first.path || first.path.length === 0) {
    throw new Error("expected standard-schema issue path")
  }
})

Deno.test("prettyPrintCommitHeader keeps ariadne report", () => {
  const report = prettyPrintCommitHeader("fature: add endpoint")
  if (typeof report !== "string" || report.length === 0) {
    throw new Error("expected formatted pretty report")
  }

  if (!report.includes("Invalid commit type")) {
    throw new Error(`expected ariadne message, got: ${report}`)
  }
})

Deno.test("prettyPrintCommitHeader returns null for valid header", () => {
  const report = prettyPrintCommitHeader("feat(api): add endpoint")
  if (report !== null) {
    throw new Error(`expected null report for valid header, got: ${report}`)
  }
})

Deno.test("prettyPrintCommitHeaderValidation returns null for valid header", () => {
  const report = prettyPrintCommitHeaderValidation("feat(api): add endpoint")
  if (report !== null) {
    throw new Error(`expected null report for valid header, got: ${report}`)
  }
})

Deno.test("prettyPrintCommitIssues pretty-prints safeParse issues", () => {
  const result = safeParseCommitHeader("fature: add endpoint")
  if (result.success) {
    throw new Error("expected failure result")
  }

  const report = prettyPrintCommitIssues("fature: add endpoint", result.issues)
  if (!report.includes("Invalid commit type")) {
    throw new Error(`expected pretty commit issue report, got: ${report}`)
  }
})

Deno.test("formatIssues renders issue path when pretty report is unavailable", () => {
  const result = safeParseCommitHeader(123)
  if (result.success) {
    throw new Error("expected failure result")
  }

  const formatted = formatIssues(123, result.issues)
  if (!formatted.includes("input:")) {
    throw new Error(`expected path-aware format output, got: ${formatted}`)
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

Deno.test("parseSemanticConfig parses semantic yaml into object config", () => {
  const config = parseSemanticConfig("types: [foo]\nscopes: [core]\n")

  if (!Array.isArray(config.types) || config.types[0] !== "foo") {
    throw new Error("expected parsed types from yaml")
  }

  if (!Array.isArray(config.scopes) || config.scopes[0] !== "core") {
    throw new Error("expected parsed scopes from yaml")
  }
})

Deno.test("safeParseSemanticConfig returns configError for invalid yaml", () => {
  const result = safeParseSemanticConfig("types: [feat")

  if (result.ok) {
    throw new Error("expected semantic config parse failure")
  }

  if (!result.configError.startsWith("Invalid semantic config:")) {
    throw new Error(`expected prefixed config error, got: ${result.configError}`)
  }
})
