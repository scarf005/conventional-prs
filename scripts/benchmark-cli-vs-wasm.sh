#!/usr/bin/env bash
set -euo pipefail

input=${BENCH_INPUT:-feat(api): add recovery}
config=${BENCH_CONFIG_PATH:-.github/semantic.yml}
out_dir=${BENCH_OUTPUT_DIR:-target/benchmark}
mkdir -p "${out_dir}"

hyperfine --warmup "${BENCH_WARMUP:-3}" --runs "${BENCH_RUNS:-10}" \
  --export-markdown "${out_dir}/cli-vs-wasm-cold.md" \
  --export-json "${out_dir}/cli-vs-wasm-cold.json" \
  --command-name native-cli "./target/release/conventional-prs --config \"${config}\" --format github --input \"${input}\" >/dev/null" \
  --command-name deno-wasm "BENCH_CONFIG_PATH=\"${config}\" BENCH_INPUT=\"${input}\" deno run -A ./scripts/benchmark_wasm.ts >/dev/null"
