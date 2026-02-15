#!/usr/bin/env bash

set -euo pipefail

commit_msg_file=${1:-}

if [[ -z "${commit_msg_file}" || ! -f "${commit_msg_file}" ]]; then
  echo "ERROR: commit-msg hook requires a commit message file path." >&2
  exit 1
fi

commit_title=$(head -n 1 "${commit_msg_file}" | tr -d '\r')

if [[ -z "${commit_title}" ]]; then
  echo "ERROR: commit title must not be empty." >&2
  exit 1
fi

if command -v conventional-prs >/dev/null 2>&1; then
  validator=(conventional-prs)
elif [[ -x "./target/debug/conventional-prs" ]]; then
  validator=(./target/debug/conventional-prs)
elif [[ -x "./target/release/conventional-prs" ]]; then
  validator=(./target/release/conventional-prs)
else
  validator=(cargo run --quiet --)
fi

if ! "${validator[@]}" --input "${commit_title}" >/dev/null 2>&1; then
  echo "ERROR: commit title does not follow Conventional Commits." >&2
  echo >&2
  "${validator[@]}" --input "${commit_title}"
  exit 1
fi
