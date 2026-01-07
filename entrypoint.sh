#!/bin/bash
set -e

if [ -d /github/workspace/.git ]; then
  git config --global --add safe.directory /github/workspace
fi

CONFIG_FLAG=""
if [ -n "$INPUT_CONFIG" ]; then
  CONFIG_FLAG="--config $INPUT_CONFIG"
fi

if [ "$GITHUB_EVENT_NAME" = "pull_request" ]; then
  PR_NUMBER=$(jq -r .pull_request.number "$GITHUB_EVENT_PATH")
  
  if [ "$INPUT_VALIDATE_PR_TITLE" = "true" ] || [ "$INPUT_VALIDATE_BOTH" = "true" ]; then
    echo "Validating PR title..."
    PR_TITLE=$(jq -r .pull_request.title "$GITHUB_EVENT_PATH")
    
    set +e
    OUTPUT=$(conventional-prs --input "$PR_TITLE" --format github $CONFIG_FLAG 2>&1)
    EXIT_CODE=$?
    set -e
    
    if [ $EXIT_CODE -ne 0 ]; then
      echo "$OUTPUT"
      {
        echo '```'
        echo "$OUTPUT"
        echo '```'
      } >> "$GITHUB_STEP_SUMMARY"
      COMMENT_MARKER="<!-- conventional-prs-title-validation -->"
      COMMENT_BODY="${COMMENT_MARKER}
## ❌ PR Title Validation Failed

\`\`\`
${OUTPUT}
\`\`\`

---
<details>
<summary>ℹ️ How to fix</summary>

Your PR title must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

**Format**: \`<type>[optional scope]: <description>\`

**Valid types**: \`feat\`, \`fix\`, \`docs\`, \`style\`, \`refactor\`, \`perf\`, \`test\`, \`build\`, \`ci\`, \`chore\`, \`revert\`

**Examples**:
- \`feat: add user authentication\`
- \`fix(api): resolve CORS issue\`
- \`docs: update README\`
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
      
      COMMENT_MARKER="<!-- conventional-prs-title-validation -->"
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
  fi
  
  if [ "$INPUT_VALIDATE_COMMITS" = "true" ] || [ "$INPUT_VALIDATE_BOTH" = "true" ]; then
    echo "Validating commits..."
    COMMITS=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
      "https://api.github.com/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/commits" | \
      jq -r '.[].commit.message' | head -1)
    
    FAILED=0
    FAILED_COMMITS=""
    while IFS= read -r commit; do
      set +e
      OUTPUT=$(conventional-prs --input "$commit" --format github $CONFIG_FLAG 2>&1)
      EXIT_CODE=$?
      set -e
      
      if [ $EXIT_CODE -ne 0 ]; then
        echo "❌ Commit validation failed: $commit"
        echo "$OUTPUT"
        {
          echo '```'
          echo "$OUTPUT"
          echo '```'
        } >> "$GITHUB_STEP_SUMMARY"
        FAILED=1
        FAILED_COMMITS="${FAILED_COMMITS}
### Commit: \`${commit}\`

\`\`\`
${OUTPUT}
\`\`\`
"
      fi
    done <<< "$COMMITS"
    
    if [ $FAILED -eq 1 ]; then
      COMMENT_MARKER="<!-- conventional-prs-commits-validation -->"
      COMMENT_BODY="${COMMENT_MARKER}
## ❌ Commit Validation Failed

One or more commits do not follow the Conventional Commits specification:

${FAILED_COMMITS}

---
<details>
<summary>ℹ️ How to fix</summary>

Each commit message must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

**Format**: \`<type>[optional scope]: <description>\`

**Valid types**: \`feat\`, \`fix\`, \`docs\`, \`style\`, \`refactor\`, \`perf\`, \`test\`, \`build\`, \`ci\`, \`chore\`, \`revert\`
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
      echo "✅ All commits are valid"
      
      COMMENT_MARKER="<!-- conventional-prs-commits-validation -->"
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
  fi
fi
