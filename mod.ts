import { validate_header as validateHeaderRaw } from "./lib/rs_lib.js"

export type ValidationHeader = {
  type: string
  scope: string[] | null
  breaking: boolean
  description: string
}

export type ValidationError = {
  kind: string
  span: {
    start: number
    end: number
  }
}

export type ValidationResult =
  | {
    ok: true
    header: ValidationHeader
  }
  | {
    ok: false
    errors: ValidationError[]
  }

export function validateCommitHeader(input: string): ValidationResult {
  return JSON.parse(validateHeaderRaw(input)) as ValidationResult
}
