Pre-commit hooks
================

This project includes a `.pre-commit-config.yaml` that runs the following checks:

- `cargo fmt --all -- --check` — ensures code is formatted.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — runs clippy and treats warnings as errors.
- `cargo audit` (optional) — checks for vulnerable dependencies if `cargo-audit` is installed.
- A `commit-msg` hook that checks commit messages follow the Conventional Commits format (see below).

Installing and enabling
-----------------------

1. Install `pre-commit` (recommended):

```bash
python3 -m pip install --user pre-commit
# or: pipx install pre-commit
```

2. Ensure Rust tools are available:

```bash
rustup component add rustfmt clippy
# optional: cargo install cargo-audit
```

3. Install the git hooks into this repo:

```bash
pre-commit install
pre-commit install --hook-type commit-msg
```

Run checks locally
------------------

Run all configured hooks against all files:

```bash
pre-commit run --all-files
```

Conventional Commits
--------------------

The commit message checker validates the first line against a pattern like:

```
<type>(optional-scope): <description>
```

Allowed types: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test

Examples:

- `feat(parser): add support for foo`
- `fix: correct crash on empty input`
