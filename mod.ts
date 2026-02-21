/**
 * WASM bindings for validating Conventional Commit headers.
 *
 * @example
 * ```ts
 * import { validateCommitHeader } from "@scarf/conventional-prs"
 *
 * const result = validateCommitHeader("feat(api): add endpoint")
 * console.log(result)
 * ```
 *
 * @module
 */
import * as wasm from "./lib/rs_lib.wasm"
import {
  __wbg_set_wasm,
  __wbindgen_init_externref_table,
  validate_header as validateHeaderRaw,
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

/** Result union returned by {@link validateCommitHeader}. */
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

/**
 * Validate a conventional commit header.
 *
 * @param input Conventional commit header text, such as `feat(api): add endpoint`.
 * @returns Structured success or error result from the WASM validator.
 */
export function validateCommitHeader(input: string): ValidationResult {
  return JSON.parse(validateHeaderRaw(input)) as ValidationResult
}
