#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

if [ "${GITHUB_EVENT_NAME:-}" != "pull_request" ] && [ "${GITHUB_EVENT_NAME:-}" != "pull_request_target" ]; then
  echo "This action only runs on pull_request or pull_request_target events"
  exit 0
fi

PR_NUMBER=$(jq -r .pull_request.number "${GITHUB_EVENT_PATH:?}")
PR_TITLE=$(jq -r .pull_request.title "${GITHUB_EVENT_PATH:?}")

if [[ ! "$PR_NUMBER" =~ ^[0-9]+$ ]]; then
  echo "Invalid PR number: $PR_NUMBER" >&2
  exit 1
fi

echo "Validating PR #$PR_NUMBER: $PR_TITLE"

CONFIG_FILE=""
for ext in yml yaml json jsonc toml; do
  RESPONSE=$(curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN:?}" \
    -H "Accept: application/vnd.github.v3.raw" \
    "https://api.github.com/repos/${GITHUB_REPOSITORY:?}/contents/.github/semantic.$ext" 2>/dev/null || true)

  if [ -n "$RESPONSE" ] && [ "$RESPONSE" != "404: Not Found" ]; then
    CONFIG_FILE="/tmp/semantic.$ext"
    printf '%s' "$RESPONSE" > "$CONFIG_FILE"
    echo "Found config: .github/semantic.$ext"
    break
  fi
done

ARGS=(--input "$PR_TITLE" --format github)
if [ -n "$CONFIG_FILE" ]; then
  ARGS+=(--config "$CONFIG_FILE")
fi

set +e
OUTPUT=$(conventional-prs "${ARGS[@]}" 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 0 ]; then
  echo "$OUTPUT"
  {
    echo '```'
    echo "$OUTPUT"
    echo '```'
  } >> "$GITHUB_STEP_SUMMARY"
  
  COMMENT_MARKER="<!-- conventional-prs-validation -->"
  COMMENT_BODY="${COMMENT_MARKER}
## ❌ PR Title Validation Failed

\`\`\`
${OUTPUT}
\`\`\`

---
<details>
<summary>ℹ️ How to fix</summary>

Your PR title must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

Check your repository's \`.github/semantic.yml\` for allowed types and scopes.
</details>"

  EXISTING_COMMENT=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
    "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$PR_NUMBER/comments" | \
    jq -r "[.[] | select(.body | contains(\"$COMMENT_MARKER\"))] | first | .id")
  
  if [ -n "$EXISTING_COMMENT" ] && [ "$EXISTING_COMMENT" != "null" ]; then
    curl -s -X PATCH -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "Content-Type: application/json" \
      "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/comments/$EXISTING_COMMENT" \
      -d "{\"body\": $(echo "$COMMENT_BODY" | jq -Rs .)}" > /dev/null
  else
    curl -s -X POST -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "Content-Type: application/json" \
      "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$PR_NUMBER/comments" \
      -d "{\"body\": $(echo "$COMMENT_BODY" | jq -Rs .)}" > /dev/null
  fi
  
  exit 1
else
  echo "✅ PR title is valid"
  
  COMMENT_MARKER="<!-- conventional-prs-validation -->"
  EXISTING_COMMENTS=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
    "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$PR_NUMBER/comments" | \
    jq -r "[.[] | select(.body | contains(\"$COMMENT_MARKER\"))] | .[].id")
  
  while IFS= read -r COMMENT_ID; do
    if [ -n "$COMMENT_ID" ] && [ "$COMMENT_ID" != "null" ]; then
      curl -s -X DELETE -H "Authorization: Bearer $GITHUB_TOKEN" \
        "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/comments/$COMMENT_ID" > /dev/null
    fi
  done <<< "$EXISTING_COMMENTS"
fi
