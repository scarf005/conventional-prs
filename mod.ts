import * as wasm from "./lib/rs_lib.wasm"
import { parse as parseYaml } from "jsr:@std/yaml@^1.0.12"
import {
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
  pretty_print_header as prettyPrintHeaderRaw,
  pretty_print_header_with_config as prettyPrintHeaderWithConfigRaw,
  validate_header as validateHeaderRaw,
  validate_header_with_config as validateHeaderWithConfigRaw,
} from "./lib/rs_lib.internal.js"

type WasmExports = {
  __wbindgen_start?: () => void
}

type BunLike = {
  file: (path: string) => {
    arrayBuffer: () => Promise<ArrayBuffer>
  }
}

const instantiateWasmExports = async (bytes: Uint8Array | ArrayBuffer): Promise<WasmExports> => {
  const source = bytes instanceof Uint8Array ? Uint8Array.from(bytes) : bytes
  const { instance } = await WebAssembly.instantiate(source, {
    "./rs_lib.internal.js": {
      __wbindgen_init_externref_table,
    },
  })

  return instance.exports as unknown as WasmExports
}

const loadWasmExports = async (): Promise<WasmExports> => {
  const wasmModule = wasm as Record<string, unknown>

  if (typeof wasmModule.__wbindgen_malloc === "function") {
    return wasmModule as unknown as WasmExports
  }

  if (typeof wasmModule.default === "string") {
    const bun = (globalThis as { Bun?: BunLike }).Bun
    if (!bun) {
      throw new Error("WASM path import detected, but Bun runtime is unavailable")
    }

    const bytes = await bun.file(wasmModule.default).arrayBuffer()
    return instantiateWasmExports(bytes)
  }

  if (wasmModule.default instanceof Uint8Array) {
    return instantiateWasmExports(wasmModule.default)
  }

  if (wasmModule.default instanceof ArrayBuffer) {
    return instantiateWasmExports(new Uint8Array(wasmModule.default))
  }

  throw new Error("Unsupported WASM module shape")
}

const wasmExports = await loadWasmExports()
__wbg_set_wasm(wasmExports)
if (typeof wasmExports.__wbindgen_start === "function") {
  wasmExports.__wbindgen_start()
} else {
  __wbindgen_init_externref_table()
}

export interface ConventionalConfig {
  enabled?: boolean
  titleOnly?: boolean
  commitsOnly?: boolean
  titleAndCommits?: boolean
  anyCommit?: boolean
  types?: readonly string[]
  scopes?: readonly string[] | null
  allowMergeCommits?: boolean
  allowRevertCommits?: boolean
  targetUrl?: string
}

export interface ConventionalCommit {
  type: string
  scope: string | null
  subject: string
  merge: null
  header: string
  body: string | null
  footer: string | null
  notes: readonly {
    title: string
    text: string
  }[]
  references: readonly {
    action: string | null
    owner: string | null
    repository: string | null
    issue: string
    raw: string
    prefix: string
  }[]
  mentions: readonly string[]
  revert: null
}

type RawValidationError = {
  kind: string
  span: {
    start: number
    end: number
  }
}

type RawValidationResult =
  | {
    ok: true
    header: {
      type: string
      scope: string[] | null
      breaking: boolean
      description: string
    }
  }
  | {
    ok: false
    errors: RawValidationError[]
  }
  | {
    ok: false
    configError: string
  }

export interface StandardSchemaV1PathSegment {
  readonly key: PropertyKey
}

export interface StandardSchemaV1Issue {
  readonly message: string
  readonly path?: ReadonlyArray<PropertyKey | StandardSchemaV1PathSegment> | undefined
}

export interface StandardSchemaV1SuccessResult<Output> {
  readonly value: Output
  readonly issues?: undefined
}

export interface StandardSchemaV1FailureResult {
  readonly issues: ReadonlyArray<StandardSchemaV1Issue>
}

export type StandardSchemaV1Result<Output> =
  | StandardSchemaV1SuccessResult<Output>
  | StandardSchemaV1FailureResult

export interface StandardSchemaV1Options {
  readonly libraryOptions?: Record<string, unknown> | undefined
}

export interface StandardSchemaV1<Input = unknown, Output = Input> {
  readonly "~standard": {
    readonly version: 1
    readonly vendor: string
    readonly types?: {
      readonly input: Input
      readonly output: Output
    } | undefined
    readonly validate: (
      value: unknown,
      options?: StandardSchemaV1Options | undefined,
    ) => StandardSchemaV1Result<Output> | Promise<StandardSchemaV1Result<Output>>
  }
}

export interface ParseIssue extends StandardSchemaV1Issue {
  readonly kind: "validation"
  readonly type: string
  readonly input: unknown
  readonly expected?: unknown
  readonly received?: unknown
  readonly span?: {
    readonly start: number
    readonly end: number
  }
  readonly config?: ConventionalConfig
}

export interface SafeParseSuccess {
  readonly success: true
  readonly output: ConventionalCommit
  readonly issues?: undefined
}

export interface SafeParseFailure {
  readonly success: false
  readonly output?: undefined
  readonly issues: ReadonlyArray<ParseIssue>
}

export type SafeParseResult = SafeParseSuccess | SafeParseFailure

export interface ParseOptions {
  readonly verbose?: boolean
}

export interface ConfiguredSchema extends StandardSchemaV1<string, ConventionalCommit> {}

const segment = (key: PropertyKey): StandardSchemaV1PathSegment => {
  return { key }
}

const issueCodeFromKind = (kind: string): string => {
  if (kind.startsWith("InvalidType")) return "invalid_type"
  if (kind.startsWith("InvalidScope")) return "invalid_scope"
  if (kind.startsWith("TypeUsedAsScope")) return "type_used_as_scope"
  if (kind.startsWith("MissingClosingParen")) return "missing_closing_paren"
  if (kind.startsWith("MissingSeparator")) return "missing_separator"
  if (kind.startsWith("MissingDescription")) return "missing_description"
  if (kind.startsWith("EmptyType")) return "empty_type"
  if (kind.startsWith("EmptyScope")) return "empty_scope"
  if (kind.startsWith("MissingColon")) return "missing_colon"
  if (kind.startsWith("MissingSpace")) return "missing_space"
  if (kind.startsWith("ExtraSpaceBeforeColon")) return "extra_space_before_colon"
  if (kind.startsWith("ExtraSpaceAfterColon")) return "extra_space_after_colon"
  if (kind.startsWith("TrailingSpaces")) return "trailing_spaces"
  if (kind.startsWith("UnexpectedChar")) return "unexpected_char"
  return "parse_error"
}

const parseQuotedValues = (fragment: string): readonly string[] => {
  const matches = fragment.match(/"([^"]*)"/g)
  if (matches === null) {
    return []
  }
  return matches.map((match) => match.slice(1, -1))
}

const parseFoundValue = (kind: string): string | undefined => {
  const match = /found: "([^"]+)"/.exec(kind)
  if (match === null) {
    return undefined
  }
  return match[1]
}

const parseExpectedValues = (kind: string, label: string): readonly string[] => {
  const pattern = new RegExp(`${label}: \\[(.*)\\]`)
  const match = pattern.exec(kind)
  if (match === null) {
    return []
  }
  return parseQuotedValues(match[1])
}

type IssueDetails = {
  message: string
  expected?: unknown
  received?: unknown
}

const issueDetailsFromKind = (kind: string, code: string): IssueDetails => {
  if (code === "invalid_type") {
    const found = parseFoundValue(kind)
    const expected = parseExpectedValues(kind, "expected")
    if (found !== undefined && expected.length > 0) {
      return {
        message: `Invalid commit type \"${found}\". Expected one of: ${expected.join(", ")}.`,
        expected,
        received: found,
      }
    }
    return { message: "Invalid commit type." }
  }

  if (code === "invalid_scope") {
    const found = parseFoundValue(kind)
    const expected = parseExpectedValues(kind, "expected")
    if (found !== undefined && expected.length > 0) {
      return {
        message: `Invalid scope \"${found}\". Expected one of: ${expected.join(", ")}.`,
        expected,
        received: found,
      }
    }
    return { message: "Invalid scope." }
  }

  if (code === "type_used_as_scope") {
    const found = parseFoundValue(kind)
    const expectedScopes = parseExpectedValues(kind, "expected_scopes")
    if (found !== undefined && expectedScopes.length > 0) {
      return {
        message: `Scope \"${found}\" is a commit type. Expected scopes: ${expectedScopes.join(", ")}.`,
        expected: expectedScopes,
        received: found,
      }
    }
    return { message: "Scope value is a commit type." }
  }

  if (code === "missing_closing_paren") return { message: "Missing closing ')' in scope." }
  if (code === "missing_separator") return { message: "Missing ': ' separator between header and description." }
  if (code === "missing_description") return { message: "Missing commit description after ': '." }
  if (code === "empty_type") return { message: "Missing commit type before scope or separator." }
  if (code === "empty_scope") return { message: "Scope cannot be empty." }
  if (code === "missing_colon") return { message: "Missing ':' separator." }
  if (code === "missing_space") return { message: "Missing required space after ':'." }
  if (code === "extra_space_before_colon") return { message: "Extra space before ':' is not allowed." }
  if (code === "extra_space_after_colon") return { message: "Extra spaces after ':' are not allowed." }
  if (code === "trailing_spaces") return { message: "Trailing spaces are not allowed in the header." }
  if (code === "unexpected_char") return { message: "Unexpected character in commit header." }
  return { message: kind }
}

const pathForCode = (code: string): ReadonlyArray<PropertyKey | StandardSchemaV1PathSegment> => {
  if (code === "invalid_input_type") return [segment("input")]
  if (code === "config_invalid") return [segment("config")]
  if (code === "invalid_type" || code === "empty_type") return [segment("type")]
  if (code === "invalid_scope" || code === "type_used_as_scope" || code === "empty_scope") {
    return [segment("scope")]
  }
  if (code === "missing_description" || code === "trailing_spaces") return [segment("description")]
  if (
    code === "missing_separator" ||
    code === "missing_colon" ||
    code === "missing_space" ||
    code === "extra_space_before_colon" ||
    code === "extra_space_after_colon"
  ) {
    return [segment("separator")]
  }
  return [segment("header")]
}

const pathToString = (path: ReadonlyArray<PropertyKey | StandardSchemaV1PathSegment> | undefined): string => {
  if (path === undefined || path.length === 0) {
    return ""
  }

  const parts = path.map((entry) => {
    if (typeof entry === "object" && entry !== null) {
      return String(entry.key)
    }
    return String(entry)
  })

  return parts.join(".")
}

const normalizeUnknownConfig = (config: unknown): ConventionalConfig => {
  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new TypeError("config must be an object when provided")
  }

  const record = config as Record<string, unknown>

  if (record["types"] !== undefined && !Array.isArray(record["types"])) {
    throw new TypeError("config.types must be an array of strings")
  }
  if (Array.isArray(record["types"]) && record["types"].some((entry) => typeof entry !== "string")) {
    throw new TypeError("config.types must be an array of strings")
  }

  if (record["scopes"] !== undefined && record["scopes"] !== null && !Array.isArray(record["scopes"])) {
    throw new TypeError("config.scopes must be an array of strings, null, or undefined")
  }
  if (
    Array.isArray(record["scopes"]) &&
    record["scopes"].some((entry) => typeof entry !== "string")
  ) {
    throw new TypeError("config.scopes must be an array of strings, null, or undefined")
  }

  return config as ConventionalConfig
}

const normalizeConfig = (config: ConventionalConfig | undefined): ConventionalConfig | undefined => {
  if (config === undefined) {
    return undefined
  }
  return normalizeUnknownConfig(config)
}

const resolveRuntimeConfig = (
  baseConfig: ConventionalConfig | undefined,
  options: StandardSchemaV1Options | undefined,
): ConventionalConfig | undefined => {
  if (options === undefined || options.libraryOptions === undefined) {
    return baseConfig
  }

  const runtimeConfig = options.libraryOptions["config"]
  if (runtimeConfig === undefined) {
    return baseConfig
  }

  return normalizeUnknownConfig(runtimeConfig)
}

const yamlScalar = (value: string | boolean | null): string => {
  if (typeof value === "boolean") {
    return value ? "true" : "false"
  }
  if (value === null) {
    return "null"
  }
  return JSON.stringify(value)
}

const yamlArray = (values: readonly string[]): string => {
  return `[${values.map((value) => JSON.stringify(value)).join(", ")}]`
}

const configToYaml = (config: ConventionalConfig): string => {
  const lines: string[] = []

  if (config.enabled !== undefined) lines.push(`enabled: ${yamlScalar(config.enabled)}`)
  if (config.titleOnly !== undefined) lines.push(`titleOnly: ${yamlScalar(config.titleOnly)}`)
  if (config.commitsOnly !== undefined) lines.push(`commitsOnly: ${yamlScalar(config.commitsOnly)}`)
  if (config.titleAndCommits !== undefined) lines.push(`titleAndCommits: ${yamlScalar(config.titleAndCommits)}`)
  if (config.anyCommit !== undefined) lines.push(`anyCommit: ${yamlScalar(config.anyCommit)}`)
  if (config.types !== undefined) lines.push(`types: ${yamlArray(config.types)}`)
  if (config.scopes !== undefined) {
    if (config.scopes === null) {
      lines.push("scopes: null")
    } else {
      lines.push(`scopes: ${yamlArray(config.scopes)}`)
    }
  }
  if (config.allowMergeCommits !== undefined) {
    lines.push(`allowMergeCommits: ${yamlScalar(config.allowMergeCommits)}`)
  }
  if (config.allowRevertCommits !== undefined) {
    lines.push(`allowRevertCommits: ${yamlScalar(config.allowRevertCommits)}`)
  }
  if (config.targetUrl !== undefined) lines.push(`targetUrl: ${yamlScalar(config.targetUrl)}`)

  return `${lines.join("\n")}\n`
}

const parseRawValidationResult = (raw: string): RawValidationResult => {
  return JSON.parse(raw) as RawValidationResult
}

const toStandardIssue = (entry: RawValidationError): StandardSchemaV1Issue => {
  const code = issueCodeFromKind(entry.kind)
  const details = issueDetailsFromKind(entry.kind, code)
  return {
    message: details.message,
    path: pathForCode(code),
  }
}

const toParseIssue = (
  entry: RawValidationError,
  input: unknown,
  config: ConventionalConfig | undefined,
): ParseIssue => {
  const type = issueCodeFromKind(entry.kind)
  const details = issueDetailsFromKind(entry.kind, type)
  return {
    kind: "validation",
    type,
    input,
    expected: details.expected,
    received: details.received,
    message: details.message,
    path: pathForCode(type),
    span: {
      start: entry.span.start,
      end: entry.span.end,
    },
    config,
  }
}

const issueLine = (issue: StandardSchemaV1Issue): string => {
  const path = pathToString(issue.path)
  if (path.length === 0) {
    return issue.message
  }
  return `${path}: ${issue.message}`
}

const parseIssueLine = (issue: ParseIssue): string => {
  return issueLine(issue)
}

const validateRaw = (input: string, config: ConventionalConfig | undefined): RawValidationResult => {
  const normalized = normalizeConfig(config)
  const raw = normalized === undefined
    ? validateHeaderRaw(input)
    : validateHeaderWithConfigRaw(input, configToYaml(normalized))
  return parseRawValidationResult(raw)
}

const prettyPrintInternal = (input: string, config: ConventionalConfig | undefined): string => {
  const normalized = normalizeConfig(config)
  if (normalized === undefined) {
    return prettyPrintHeaderRaw(input)
  }
  return prettyPrintHeaderWithConfigRaw(input, configToYaml(normalized))
}

const toConventionalCommit = (
  header: {
    type: string
    scope: string[] | null
    breaking: boolean
    description: string
  },
  input: string,
): ConventionalCommit => {
  return {
    type: header.type,
    scope: header.scope === null ? null : header.scope.join(", "),
    subject: header.description,
    merge: null,
    header: input.split(/\r?\n/u, 1)[0],
    body: null,
    footer: null,
    notes: header.breaking ? [{ title: "BREAKING CHANGE", text: "" }] : [],
    references: [],
    mentions: [],
    revert: null,
  }
}

const schemaConfigStore = new WeakMap<object, ConventionalConfig | undefined>()

const getSchemaConfig = (schema: ConfiguredSchema): ConventionalConfig | undefined => {
  if (!schemaConfigStore.has(schema as object)) {
    throw new TypeError("schema must be created by config()")
  }
  return schemaConfigStore.get(schema as object)
}

const createSchema = (baseConfig: ConventionalConfig | undefined): ConfiguredSchema => {
  const schema: ConfiguredSchema = {
    "~standard": {
      version: 1,
      vendor: "conventional-prs",
      validate: (
        value: unknown,
        options?: StandardSchemaV1Options | undefined,
      ): StandardSchemaV1Result<ConventionalCommit> => {
        const config = resolveRuntimeConfig(baseConfig, options)
        if (typeof value !== "string") {
          return {
            issues: [{
              message: "Commit header input must be a string.",
              path: pathForCode("invalid_input_type"),
            }],
          }
        }

        const result = validateRaw(value, config)
        if (result.ok) {
          return {
            value: toConventionalCommit(result.header, value),
          }
        }

        if ("configError" in result) {
          return {
            issues: [{
              message: `Invalid semantic config: ${result.configError}`,
              path: pathForCode("config_invalid"),
            }],
          }
        }

        return {
          issues: result.errors.map(toStandardIssue),
        }
      },
    },
  }
  schemaConfigStore.set(schema as object, baseConfig)
  return schema
}

export function config(options?: ConventionalConfig): ConfiguredSchema {
  return createSchema(normalizeConfig(options))
}

export function parseConfig(input: string): ConfiguredSchema {
  const parsed = parseYaml(input)
  if (parsed === null || parsed === undefined) {
    return config()
  }
  return config(normalizeUnknownConfig(parsed))
}

export function safeParse(schema: ConfiguredSchema, input: unknown): SafeParseResult {
  const cfg = getSchemaConfig(schema)
  if (typeof input !== "string") {
    return {
      success: false,
      issues: [{
        kind: "validation",
        type: "invalid_input_type",
        input,
        expected: "string",
        received: typeof input,
        message: "Commit header input must be a string.",
        path: pathForCode("invalid_input_type"),
        config: cfg,
      }],
    }
  }

  const result = validateRaw(input, cfg)
  if (result.ok) {
    return {
      success: true,
      output: toConventionalCommit(result.header, input),
    }
  }

  if ("configError" in result) {
    return {
      success: false,
      issues: [{
        kind: "validation",
        type: "config_invalid",
        input,
        message: `Invalid semantic config: ${result.configError}`,
        path: pathForCode("config_invalid"),
        config: cfg,
      }],
    }
  }

  return {
    success: false,
    issues: result.errors.map((entry) => toParseIssue(entry, input, cfg)),
  }
}

export function summarize(issues: ReadonlyArray<ParseIssue>): string {
  if (issues.length === 0) {
    return ""
  }

  const input = issues[0].input
  if (typeof input === "string") {
    const report = prettyPrintInternal(input, issues[0].config)
    if (report.length > 0) {
      return report
    }
  }

  return issues.map(parseIssueLine).join("\n")
}

export function parse(
  schema: ConfiguredSchema,
  input: unknown,
  options?: ParseOptions,
): ConventionalCommit {
  const result = safeParse(schema, input)
  if (result.success) {
    return result.output
  }

  if (options?.verbose === false) {
    throw new Error(parseIssueLine(result.issues[0]))
  }

  throw new Error(summarize(result.issues))
}
