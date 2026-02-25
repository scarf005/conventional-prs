import * as wasm from "./lib/rs_lib.wasm"
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

export interface StandardSchemaV1Issue {
  readonly message: string
  readonly path?: ReadonlyArray<PropertyKey | { readonly key: PropertyKey }> | undefined
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

const normalizeConfig = (config: ConventionalConfig | undefined): ConventionalConfig | undefined => {
  if (config === undefined) {
    return undefined
  }

  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new TypeError("config must be an object when provided")
  }

  if (config.types !== undefined && !Array.isArray(config.types)) {
    throw new TypeError("config.types must be an array of strings")
  }
  if (config.types !== undefined && config.types.some((entry) => typeof entry !== "string")) {
    throw new TypeError("config.types must be an array of strings")
  }

  if (config.scopes !== undefined && config.scopes !== null && !Array.isArray(config.scopes)) {
    throw new TypeError("config.scopes must be an array of strings, null, or undefined")
  }
  if (
    config.scopes !== undefined &&
    config.scopes !== null &&
    config.scopes.some((entry) => typeof entry !== "string")
  ) {
    throw new TypeError("config.scopes must be an array of strings, null, or undefined")
  }

  return config
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

const toIssue = (entry: RawValidationError): CommitHeaderIssue => {
  return {
    code: issueCodeFromKind(entry.kind),
    message: entry.kind,
    span: {
      start: entry.span.start,
      end: entry.span.end,
    },
  }
}

const safeParseInternal = (value: unknown, config: ConventionalConfig | undefined): CommitHeaderParseResult => {
  if (typeof value !== "string") {
    return {
      success: false,
      issues: [{
        code: "invalid_input_type",
        message: "Input must be a string",
        path: ["input"],
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
        message: result.configError,
        path: ["config"],
      }],
    }
  }

  return {
    success: false,
    issues: result.errors.map(toIssue),
  }
}

const createSchema = (config: ConventionalConfig | undefined): CommitHeaderSchema => {
  const schema: CommitHeaderSchema = {
    "~standard": {
      version: 1,
      vendor: "conventional-prs",
      validate: (
        value: unknown,
        _options?: StandardSchemaV1Options | undefined,
      ): StandardSchemaV1Result<CommitHeader> => {
        const result = safeParseInternal(value, config)
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
      const result = safeParseInternal(value, config)
      if (result.success) {
        return result.data
      }

      throw new Error(formatIssues(value, result.issues, config))
    },
    safeParse: (value: unknown): CommitHeaderParseResult => {
      return safeParseInternal(value, config)
    },
  }
  return schema
}

export function commitHeaderSchema(config?: ConventionalConfig): CommitHeaderSchema {
  return createSchema(config)
}

export function safeParseCommitHeader(
  input: unknown,
  config?: ConventionalConfig,
): CommitHeaderParseResult {
  return commitHeaderSchema(config).safeParse(input)
}

export function parseCommitHeader(input: unknown, config?: ConventionalConfig): CommitHeader {
  return commitHeaderSchema(config).parse(input)
}

export function formatIssues(
  input: unknown,
  issues: ReadonlyArray<CommitHeaderIssue>,
  config?: ConventionalConfig,
): string {
  if (typeof input !== "string") {
    return issues.map((issue) => issue.message).join("\n")
  }

  const hasConfigIssue = issues.some((issue) => issue.code === "config_invalid")
  if (hasConfigIssue) {
    return issues.map((issue) => issue.message).join("\n")
  }

  const normalized = normalizeConfig(config)
  const report = normalized === undefined
    ? prettyPrintHeaderRaw(input)
    : prettyPrintHeaderWithConfigRaw(input, configToYaml(normalized))

  if (report.length > 0) {
    return report
  }
  return issues.map((issue) => issue.message).join("\n")
}
