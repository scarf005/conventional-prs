#!/bin/bash
set -e

# Script to publish the GitHub Action as alpha using GitHub CLI

echo "📦 Publishing Conventional PR Validator as Alpha..."

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed. Please install it first:"
    echo "   https://cli.github.com/"
    exit 1
fi

# Check if logged in
if ! gh auth status &> /dev/null; then
    echo "❌ Not logged in to GitHub CLI. Run: gh auth login"
    exit 1
fi

# Get current repo
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
echo "📍 Repository: $REPO"

# Build the project
echo "🔨 Building release binary..."
cargo build --release

# Create alpha tag if it doesn't exist
ALPHA_VERSION="v0.1.0-alpha"
echo "🏷️  Creating tag: $ALPHA_VERSION"

# Check if tag exists
if git rev-parse "$ALPHA_VERSION" >/dev/null 2>&1; then
    echo "⚠️  Tag $ALPHA_VERSION already exists. Deleting old tag..."
    git tag -d "$ALPHA_VERSION" || true
    git push origin ":refs/tags/$ALPHA_VERSION" || true
fi

# Create new tag
git tag -a "$ALPHA_VERSION" -m "Alpha release $ALPHA_VERSION"
git push origin "$ALPHA_VERSION"

echo ""
echo "✅ Alpha version published!"
echo ""
echo "📋 Next steps:"
echo "   1. The release workflow will build binaries for multiple platforms"
echo "   2. Users can now use this action in their workflows:"
echo ""
echo "   - uses: $REPO@$ALPHA_VERSION"
echo "     with:"
echo "       github_token: \${{ secrets.GITHUB_TOKEN }}"
echo ""
echo "📖 To create a GitHub App (optional):"
echo "   1. Go to: https://github.com/settings/apps/new"
echo "   2. Upload the manifest from: .github/app-manifest.json"
echo "   3. Configure webhook URL and permissions"
echo ""
