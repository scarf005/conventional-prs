/**
 * Validate Conventional Commit headers with optional `semantic.yml` rules.
 *
 * This module exposes a synchronous parser backed by WebAssembly.
 *
 * @example Valid header with defaults
 * ```ts
 * import { validateCommitHeader } from "@scarf/conventional-prs"
 *
 * const result = validateCommitHeader("feat(api): add endpoint")
 *
 * if (result.ok) {
 *   console.log(result.header.type) // "feat"
 *   console.log(result.header.scope) // ["api"]
 * }
 * ```
 *
 * @example Header validated with custom `semantic.yml` text
 * ```ts
 * import { validateCommitHeader } from "@scarf/conventional-prs"
 *
 * const semanticYml = `
 * types: [feat, fix, chore]
 * scopes: [api, ui]
 * `
 *
 * const result = validateCommitHeader("chore(api): release", semanticYml)
 * console.log(result.ok) // true
 * ```
 *
 * @module
 */
import * as wasm from "./lib/rs_lib.wasm"
import {
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
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

/** Successful parsed header fields from a conventional commit header. */
export type ValidationHeader = {
  /** Commit type, like `feat`, `fix`, or `docs`. */
  type: string
  /** Parsed scopes, or `null` when no scope is present. */
  scope: string[] | null
  /** True when the header includes `!` before `:`. */
  breaking: boolean
  /** Commit description text after `: `. */
  description: string
}

/** Validation error entry emitted by the Rust parser. */
export type ValidationError = {
  /** Parser error kind name. */
  kind: string
  /** Byte offsets for the problematic input span. */
  span: {
    /** Inclusive start byte offset. */
    start: number
    /** Exclusive end byte offset. */
    end: number
  }
}

/**
 * Result union returned by {@link validateCommitHeader}.
 *
 * @example Success
 * ```ts
 * {
 *   ok: true,
 *   header: {
 *     type: "feat",
 *     scope: ["api"],
 *     breaking: false,
 *     description: "add endpoint"
 *   }
 * }
 * ```
 *
 * @example Parse failure
 * ```ts
 * {
 *   ok: false,
 *   errors: [
 *     {
 *       kind: "InvalidType { actual: \"fature\", expected: [\"feat\", ...] }",
 *       span: { start: 0, end: 6 }
 *     }
 *   ]
 * }
 * ```
 *
 * @example Config failure
 * ```ts
 * {
 *   ok: false,
 *   configError: "did not find expected node content at line 1 column 13"
 * }
 * ```
 */
export type ValidationResult =
  | {
    /** True when parsing and validation succeeded. */
    ok: true
    /** Parsed commit header. */
    header: ValidationHeader
  }
  | {
    /** False when validation fails. */
    ok: false
    /** All parser/validation errors collected by the parser. */
    errors: ValidationError[]
  }
  | {
    /** False when the optional semantic YAML config cannot be parsed. */
    ok: false
    /** Human-readable YAML parse error message. */
    configError: string
  }

/**
 * Validate a conventional commit header.
 *
 * @param input Conventional commit header text, such as `feat(api): add endpoint`.
 * @param semanticYamlRaw Optional raw `semantic.yml` text.
 * If provided, custom `types`/`scopes` from this YAML are used instead of defaults.
 * @returns Structured success, parse error, or config error result.
 */
export function validateCommitHeader(input: string, semanticYamlRaw?: string): ValidationResult {
  const rawResult = semanticYamlRaw === undefined
    ? validateHeaderRaw(input)
    : validateHeaderWithConfigRaw(input, semanticYamlRaw)

  return JSON.parse(rawResult) as ValidationResult
}
