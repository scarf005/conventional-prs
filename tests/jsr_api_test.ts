import { validateCommitHeader } from "../mod.ts"

Deno.test("validateCommitHeader returns parsed header on success", () => {
  const result = validateCommitHeader("feat(api): add endpoint")

  if (!result.ok) {
    throw new Error("expected success result")
  }

  if (result.header.type !== "feat") {
    throw new Error(`expected feat type, got ${result.header.type}`)
  }

  if (result.header.description !== "add endpoint") {
    throw new Error(`unexpected description: ${result.header.description}`)
  }
})

Deno.test("validateCommitHeader applies optional semantic yaml", () => {
  const semanticYamlRaw = "types: [foo]\nscopes: [core]\n"
  const result = validateCommitHeader("foo(core): works", semanticYamlRaw)

  if (!result.ok) {
    throw new Error("expected success result with custom semantic yaml")
  }

  if (result.header.type !== "foo") {
    throw new Error(`expected foo type, got ${result.header.type}`)
  }
})

Deno.test("validateCommitHeader returns configError for invalid semantic yaml", () => {
  const result = validateCommitHeader("feat: add endpoint", "types: [feat")

  if (result.ok) {
    throw new Error("expected config parse failure")
  }

  if (!("configError" in result) || typeof result.configError !== "string") {
    throw new Error("expected configError string")
  }
})
