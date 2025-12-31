#!/bin/bash
set -e

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
      echo "❌ PR title validation failed:"
      echo ""
      echo '```'
      echo "$OUTPUT"
      echo '```'
      exit 1
    else
      echo "✅ PR title is valid"
    fi
  fi
  
  if [ "$INPUT_VALIDATE_COMMITS" = "true" ] || [ "$INPUT_VALIDATE_BOTH" = "true" ]; then
    echo "Validating commits..."
    COMMITS=$(gh pr view "$PR_NUMBER" --json commits --jq '.commits[].messageHeadline')
    
    FAILED=0
    while IFS= read -r commit; do
      set +e
      OUTPUT=$(conventional-prs --input "$commit" --format github $CONFIG_FLAG 2>&1)
      EXIT_CODE=$?
      set -e
      
      if [ $EXIT_CODE -ne 0 ]; then
        echo "❌ Commit validation failed: $commit"
        echo ""
        echo '```'
        echo "$OUTPUT"
        echo '```'
        FAILED=1
      fi
    done <<< "$COMMITS"
    
    if [ $FAILED -eq 1 ]; then
      exit 1
    else
      echo "✅ All commits are valid"
    fi
  fi
fi
