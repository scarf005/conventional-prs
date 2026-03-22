import test from "node:test"
import assert from "node:assert/strict"

import {
  buildFailureComment,
  inferConfigFormat,
  parseBooleanInput,
  readPullRequest,
} from "./index.mjs"

test("inferConfigFormat resolves supported extensions", () => {
  assert.equal(inferConfigFormat(".github/semantic.yml"), "yml")
  assert.equal(inferConfigFormat(".github/semantic.yaml"), "yaml")
  assert.equal(inferConfigFormat(".github/semantic.json"), "json")
  assert.equal(inferConfigFormat(".github/semantic.jsonc"), "jsonc")
  assert.equal(inferConfigFormat(".github/semantic.toml"), "toml")
})

test("parseBooleanInput handles common GitHub input values", () => {
  assert.equal(parseBooleanInput("true"), true)
  assert.equal(parseBooleanInput("YES"), true)
  assert.equal(parseBooleanInput("0", true), false)
  assert.equal(parseBooleanInput(undefined, true), true)
})

test("readPullRequest extracts title and number", () => {
  assert.deepEqual(
    readPullRequest({
      pull_request: {
        number: 7,
        title: "feat: add validation",
      },
    }),
    {
      number: 7,
      title: "feat: add validation",
    },
  )
})

test("buildFailureComment includes marker and report", () => {
  const comment = buildFailureComment("Error: invalid title")

  assert.match(comment, /conventional-prs-validation/)
  assert.match(comment, /Error: invalid title/)
})
