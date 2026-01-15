# GitHub App & Action Setup

This guide explains how to publish and use the Conventional PR Validator as a GitHub Action.

## Quick Start: Using as a GitHub Action

Add this workflow to `.github/workflows/pr-validation.yml` in your repository:

```yaml
name: Validate PR

on:
  # Use pull_request_target so the action can post PR comments on fork PRs.
  pull_request_target:
    types: [opened, edited, synchronize, reopened]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Validate PR Title
        uses: scarf005/conventional-prs@v0.1.0-alpha
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          validate_pr_title: 'true'
          validate_commits: 'false'
```

## Publishing as Alpha

### Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- Rust toolchain installed
- Repository pushed to GitHub

### Steps to Publish

1. **Run the publish script:**

   ```bash
   ./scripts/publish-alpha.sh
   ```

   This script will:
   - Build the release binary
   - Create an alpha tag (`v0.1.0-alpha`)
   - Push the tag to GitHub
   - Trigger the release workflow

2. **Wait for the release workflow to complete:**

   The workflow will build binaries for:
   - Linux (GNU and MUSL)
   - macOS (Intel and ARM)
   - Windows

3. **Verify the release:**

   ```bash
   gh release view v0.1.0-alpha
   ```

## Manual Publishing with GitHub CLI

If you prefer manual control:

```bash
# Build the project
cargo build --release

# Create and push tag
git tag -a v0.1.0-alpha -m "Alpha release"
git push origin v0.1.0-alpha

# The release workflow will automatically trigger
```

## Configuration Options

The action accepts these inputs:

| Input | Description | Default | Required |
|-------|-------------|---------|----------|
| `github_token` | GitHub token for API access | `${{ github.token }}` | Yes |
| `config` | Path to config file | - | No |
| `validate_pr_title` | Validate PR title | `true` | No |
| `validate_commits` | Validate commit messages | `false` | No |
| `validate_both` | Validate both | `false` | No |

## Example Configurations

### Validate PR Title Only

```yaml
- uses: scarf005/conventional-prs@v0.1.0-alpha
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    validate_pr_title: 'true'
```

### Validate All Commits

```yaml
- uses: scarf005/conventional-prs@v0.1.0-alpha
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    validate_commits: 'true'
```

### Use Custom Configuration

```yaml
- uses: scarf005/conventional-prs@v0.1.0-alpha
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    config: .github/conventional-config.yml
```

## Creating a GitHub App (Optional)

If you want to create a full GitHub App instead of just an Action:

1. Go to https://github.com/settings/apps/new

2. Fill in the form:
   - **GitHub App name:** `Conventional PR Validator (Alpha)`
   - **Homepage URL:** Your repository URL
   - **Webhook URL:** Your webhook endpoint (if you have one)

3. Set permissions:
   - **Repository permissions:**
     - Contents: Read
     - Pull requests: Read & Write
     - Checks: Read & Write
     - Statuses: Read & Write

4. Subscribe to events:
   - Pull request
   - Pull request review

5. Create the app and note the App ID

6. Generate a private key and download it

7. Install the app on your repositories

## Updating the Action

To publish a new version:

1. Update version in `Cargo.toml`
2. Create a new tag:
   ```bash
   git tag -a v0.2.0-alpha -m "Alpha release 0.2.0"
   git push origin v0.2.0-alpha
   ```

## Troubleshooting

### Action fails to build

Make sure the repository includes:
- `Cargo.toml` and `Cargo.lock`
- All source files in `src/`
- The `action.yml` file

### Permission denied errors

Ensure the `GITHUB_TOKEN` has sufficient permissions. You may need to update your workflow:

```yaml
permissions:
  contents: read
  pull-requests: write
  checks: write
```

### Validation not running

Check that:
1. The workflow file is in `.github/workflows/`
2. The trigger events are correct (`pull_request`)
3. The action version/tag exists

## Support

For issues and feature requests, visit: https://github.com/scarf005/conventional-prs/issues
