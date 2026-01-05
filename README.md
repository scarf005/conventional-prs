# conventional-prs

Validates PR titles against [Conventional Commits](https://www.conventionalcommits.org/). Posts error comments on PRs.

```yaml
- uses: docker://ghcr.io/scarf005/conventional-prs:main
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}  # Required to post comments
```

## Local Usage

```bash
cargo install --git https://github.com/scarf005/conventional-prs
conventional-prs --input "feat: add feature"
```

## Configuration

Optional. Create `.github/semantic.yml`:

```yaml
types: [feat, fix, docs]
scopes: [api, ui]
```

Compatible with [semantic-prs](https://github.com/Ezard/semantic-prs).

## License

AGPL-3.0-only
test3
