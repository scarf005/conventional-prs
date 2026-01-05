#!/bin/bash
set -e

git config --global --add safe.directory /github/workspace

CONFIG_FLAG=""
if [ -n "$INPUT_CONFIG" ]; then
  CONFIG_FLAG="--config $INPUT_CONFIG"
fi

if [ "$GITHUB_EVENT_NAME" = "pull_request" ]; then
  PR_NUMBER=$(jq -r .pull_request.number "$GITHUB_EVENT_PATH")
  
  if [ "$INPUT_VALIDATE_PR_TITLE" = "true" ] || [ "$INPUT_VALIDATE_BOTH" = "true" ]; then
    echo "Validating PR title..."
    PR_TITLE=$(gh pr view "$PR_NUMBER" --json title --jq '.title')
    
    set +e
    OUTPUT=$(conventional-prs --input "$PR_TITLE" --format github $CONFIG_FLAG 2>&1)
    EXIT_CODE=$?
    set -e
    
    if [ $EXIT_CODE -ne 0 ]; then
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

      EXISTING_COMMENT=$(gh pr view "$PR_NUMBER" --json comments --jq "[.comments[] | select(.body | contains(\"$COMMENT_MARKER\")) | select(.author.login == \"github-actions\")] | first | .id")
      
      if [ -n "$EXISTING_COMMENT" ] && [ "$EXISTING_COMMENT" != "null" ]; then
        gh api -X PATCH "/repos/$GITHUB_REPOSITORY/issues/comments/$EXISTING_COMMENT" -f body="$COMMENT_BODY"
      else
        gh pr comment "$PR_NUMBER" --body "$COMMENT_BODY"
      fi
      
      exit 1
    else
      echo "✅ PR title is valid"
      
      COMMENT_MARKER="<!-- conventional-prs-title-validation -->"
      EXISTING_COMMENT=$(gh pr view "$PR_NUMBER" --json comments --jq "[.comments[] | select(.body | contains(\"$COMMENT_MARKER\")) | select(.author.login == \"github-actions\")] | first | .id")
      
      if [ -n "$EXISTING_COMMENT" ] && [ "$EXISTING_COMMENT" != "null" ]; then
        gh api -X DELETE "/repos/$GITHUB_REPOSITORY/issues/comments/$EXISTING_COMMENT"
      fi
    fi
  fi
  
  if [ "$INPUT_VALIDATE_COMMITS" = "true" ] || [ "$INPUT_VALIDATE_BOTH" = "true" ]; then
    echo "Validating commits..."
    COMMITS=$(gh pr view "$PR_NUMBER" --json commits --jq '.commits[].messageHeadline')
    
    FAILED=0
    FAILED_COMMITS=""
    while IFS= read -r commit; do
      set +e
      OUTPUT=$(conventional-prs --input "$commit" --format github $CONFIG_FLAG 2>&1)
      EXIT_CODE=$?
      set -e
      
      if [ $EXIT_CODE -ne 0 ]; then
        echo "❌ Commit validation failed: $commit"
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

      EXISTING_COMMENT=$(gh pr view "$PR_NUMBER" --json comments --jq "[.comments[] | select(.body | contains(\"$COMMENT_MARKER\")) | select(.author.login == \"github-actions\")] | first | .id")
      
      if [ -n "$EXISTING_COMMENT" ] && [ "$EXISTING_COMMENT" != "null" ]; then
        gh api -X PATCH "/repos/$GITHUB_REPOSITORY/issues/comments/$EXISTING_COMMENT" -f body="$COMMENT_BODY"
      else
        gh pr comment "$PR_NUMBER" --body "$COMMENT_BODY"
      fi
      
      exit 1
    else
      echo "✅ All commits are valid"
      
      COMMENT_MARKER="<!-- conventional-prs-commits-validation -->"
      EXISTING_COMMENT=$(gh pr view "$PR_NUMBER" --json comments --jq "[.comments[] | select(.body | contains(\"$COMMENT_MARKER\")) | select(.author.login == \"github-actions\")] | first | .id")
      
      if [ -n "$EXISTING_COMMENT" ] && [ "$EXISTING_COMMENT" != "null" ]; then
        gh api -X DELETE "/repos/$GITHUB_REPOSITORY/issues/comments/$EXISTING_COMMENT"
      fi
    fi
  fi
fi
