import * as wasm from "./lib/rs_lib.wasm"
import {
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
  parse_semantic_yaml_config as parseSemanticYamlConfigRaw,
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

export interface CommitHeader {
  type: string
  scope: readonly string[] | null
  breaking: boolean
  description: string
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

type RawSemanticConfigResult =
  | {
    ok: true
    config: ConventionalConfig
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

export interface CommitHeaderIssue extends StandardSchemaV1Issue {
  readonly code: string
  readonly span?: {
    readonly start: number
    readonly end: number
  } | undefined
  readonly expected?: unknown
  readonly received?: unknown
}

export interface CommitHeaderParseSuccess {
  readonly success: true
  readonly data: CommitHeader
}

export interface CommitHeaderParseFailure {
  readonly success: false
  readonly issues: ReadonlyArray<CommitHeaderIssue>
}

export type CommitHeaderParseResult = CommitHeaderParseSuccess | CommitHeaderParseFailure

export interface CommitHeaderSchema extends StandardSchemaV1<string, CommitHeader> {
  parse: (value: unknown) => CommitHeader
  safeParse: (value: unknown) => CommitHeaderParseResult
}

export interface SemanticConfigParseSuccess {
  readonly ok: true
  readonly config: ConventionalConfig
}

export interface SemanticConfigParseFailure {
  readonly ok: false
  readonly configError: string
}

export type SemanticConfigParseResult = SemanticConfigParseSuccess | SemanticConfigParseFailure

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

const issueLine = (issue: CommitHeaderIssue): string => {
  const path = pathToString(issue.path)
  if (path.length === 0) {
    return issue.message
  }
  return `${path}: ${issue.message}`
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

const parseRawSemanticConfigResult = (raw: string): RawSemanticConfigResult => {
  return JSON.parse(raw) as RawSemanticConfigResult
}

const toIssue = (entry: RawValidationError): CommitHeaderIssue => {
  const code = issueCodeFromKind(entry.kind)
  const details = issueDetailsFromKind(entry.kind, code)
  return {
    code,
    message: details.message,
    path: pathForCode(code),
    span: {
      start: entry.span.start,
      end: entry.span.end,
    },
    expected: details.expected,
    received: details.received,
  }
}

const safeParseInternal = (value: unknown, config: ConventionalConfig | undefined): CommitHeaderParseResult => {
  if (typeof value !== "string") {
    return {
      success: false,
      issues: [{
        code: "invalid_input_type",
        message: "Commit header input must be a string.",
        path: pathForCode("invalid_input_type"),
        expected: "string",
        received: typeof value,
      }],
    }
  }

  const normalized = normalizeConfig(config)
  const raw = normalized === undefined
    ? validateHeaderRaw(value)
    : validateHeaderWithConfigRaw(value, configToYaml(normalized))
  const result = parseRawValidationResult(raw)

  if (result.ok) {
    return {
      success: true,
      data: {
        type: result.header.type,
        scope: result.header.scope,
        breaking: result.header.breaking,
        description: result.header.description,
      },
    }
  }

  if ("configError" in result) {
    return {
      success: false,
      issues: [{
        code: "config_invalid",
        message: `Invalid semantic config: ${result.configError}`,
        path: pathForCode("config_invalid"),
      }],
    }
  }

  return {
    success: false,
    issues: result.errors.map(toIssue),
  }
}

const prettyPrintInternal = (input: string, config: ConventionalConfig | undefined): string => {
  const normalized = normalizeConfig(config)
  if (normalized === undefined) {
    return prettyPrintHeaderRaw(input)
  }
  return prettyPrintHeaderWithConfigRaw(input, configToYaml(normalized))
}

const createSchema = (config: ConventionalConfig | undefined): CommitHeaderSchema => {
  const normalizedConfig = normalizeConfig(config)

  const schema: CommitHeaderSchema = {
    "~standard": {
      version: 1,
      vendor: "conventional-prs",
      validate: (
        value: unknown,
        options?: StandardSchemaV1Options | undefined,
      ): StandardSchemaV1Result<CommitHeader> => {
        const effectiveConfig = resolveRuntimeConfig(normalizedConfig, options)
        const result = safeParseInternal(value, effectiveConfig)
        if (result.success) {
          return {
            value: result.data,
          }
        }
        return {
          issues: result.issues,
        }
      },
    },
    parse: (value: unknown): CommitHeader => {
      const result = safeParseInternal(value, normalizedConfig)
      if (result.success) {
        return result.data
      }

      throw new Error(formatIssues(value, result.issues, normalizedConfig))
    },
    safeParse: (value: unknown): CommitHeaderParseResult => {
      return safeParseInternal(value, normalizedConfig)
    },
  }
  return schema
}

export function commitHeaderSchema(config?: ConventionalConfig): CommitHeaderSchema {
  return createSchema(config)
}

/**
 * Parses a commit header and returns a success/failure result.
 *
 * # Examples
 *
 * ```ts
 * const ok = safeParseCommitHeader("feat(api): add endpoint")
 * if (ok.success) {
 *   console.log(ok.data.type)
 * }
 * // Output: feat
 * ```
 */
export function safeParseCommitHeader(
  input: unknown,
  config?: ConventionalConfig,
): CommitHeaderParseResult {
  return commitHeaderSchema(config).safeParse(input)
}

/**
 * Parses a commit header and throws when invalid.
 *
 * # Examples
 *
 * ```ts
 * const header = parseCommitHeader("fix(ui): resolve button state")
 * console.log(header.description)
 * // Output: resolve button state
 * ```
 */
export function parseCommitHeader(input: unknown, config?: ConventionalConfig): CommitHeader {
  return commitHeaderSchema(config).parse(input)
}

/**
 * Validates and returns an Ariadne-formatted report for invalid headers.
 * Returns `null` when valid.
 *
 * # Examples
 *
 * ```ts
 * const report = prettyPrintCommitHeaderValidation("fature: add endpoint")
 * console.log(typeof report === "string" && report.includes("Invalid commit type"))
 * // Output: true
 * ```
 */
export function prettyPrintCommitHeaderValidation(input: unknown, config?: ConventionalConfig): string | null {
  const result = safeParseInternal(input, config)
  if (result.success) {
    return null
  }

  return prettyPrintCommitIssues(input, result.issues, config)
}

export function prettyPrintCommitHeader(input: unknown, config?: ConventionalConfig): string | null {
  return prettyPrintCommitHeaderValidation(input, config)
}

/**
 * Pretty-prints issues from `safeParseCommitHeader`.
 *
 * # Examples
 *
 * ```ts
 * const parsed = safeParseCommitHeader("fature: add endpoint")
 * if (!parsed.success) {
 *   const report = prettyPrintCommitIssues("fature: add endpoint", parsed.issues)
 *   console.log(report.includes("Invalid commit type"))
 *   // Output: true
 * }
 * ```
 */
export function prettyPrintCommitIssues(
  input: unknown,
  issues: ReadonlyArray<CommitHeaderIssue>,
  config?: ConventionalConfig,
): string {
  if (typeof input !== "string") {
    return issues.map(issueLine).join("\n")
  }

  const report = prettyPrintInternal(input, config)
  if (report.length > 0) {
    return report
  }

  return issues.map(issueLine).join("\n")
}

export function formatIssues(
  input: unknown,
  issues: ReadonlyArray<CommitHeaderIssue>,
  config?: ConventionalConfig,
): string {
  return prettyPrintCommitIssues(input, issues, config)
}

/**
 * Parses semantic.yml text into a typed config object.
 *
 * # Examples
 *
 * ```ts
 * const parsed = safeParseSemanticConfig("types: [feat, fix]\nscopes: [api]\n")
 * console.log(parsed.ok)
 * // Output: true
 * ```
 */
export function safeParseSemanticConfig(yamlText: string): SemanticConfigParseResult {
  const raw = parseSemanticYamlConfigRaw(yamlText)
  const result = parseRawSemanticConfigResult(raw)

  if (result.ok) {
    return {
      ok: true,
      config: normalizeUnknownConfig(result.config),
    }
  }

  return {
    ok: false,
    configError: `Invalid semantic config: ${result.configError}`,
  }
}

/**
 * Parses semantic.yml text and throws on invalid input.
 *
 * # Examples
 *
 * ```ts
 * const config = parseSemanticConfig("types: [feat, fix]\nscopes: [api]\n")
 * console.log(config.types?.includes("feat") === true)
 * // Output: true
 * ```
 */
export function parseSemanticConfig(yamlText: string): ConventionalConfig {
  const result = safeParseSemanticConfig(yamlText)
  if (result.ok) {
    return result.config
  }
  throw new Error(result.configError)
}
