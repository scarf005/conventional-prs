// @generated file from wasmbuild -- do not edit
// deno-lint-ignore-file
// deno-fmt-ignore-file

export function parse_semantic_yaml_config(semantic_yaml_raw: string): string

export function pretty_print_header(input: string): string

export function pretty_print_header_with_config(
  input: string,
  semantic_yaml_raw: string,
): string

export function pretty_print_header_with_config_auto(
  input: string,
  config_raw: string,
  format_hint?: string | null,
): string

export function validate_header(input: string): string

export function validate_header_with_config(
  input: string,
  semantic_yaml_raw: string,
): string

export function validate_header_with_config_auto(
  input: string,
  config_raw: string,
  format_hint?: string | null,
): string
