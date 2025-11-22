Development environment setup
=============================

This document describes a minimal development environment for this Rust project on Linux (zsh). It covers installing the Rust toolchain and useful components, installing `pre-commit` and enabling the repository hooks, running basic checks, and optional editor recommendations.

Prerequisites
-------------

- A reasonably recent Rust toolchain. We recommend installing via `rustup`.
- Python 3 (for `pre-commit` and the commit-msg checker script).
- Git and a POSIX shell (zsh is used in examples).

Quick install (zsh)
-------------------

Run these commands to get a working environment quickly:

```bash
# Install rustup (if you don't have it)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Ensure components used by hooks are available
rustup component add rustfmt clippy

# Install pre-commit (one of these)
python3 -m pip install --user pre-commit
# or, if you use pipx:
# pipx install pre-commit

# Optional but recommended for security checks
cargo install cargo-audit || true
```

Enable repository hooks
-----------------------

From the repository root (`/home/thomas/pets/resourcepool`):

```bash
cd /home/thomas/pets/resourcepool
pre-commit install
pre-commit install --hook-type commit-msg
```

This installs the hooks defined in `.pre-commit-config.yaml` which will run on commit events.

Useful verification commands
----------------------------

- Run all pre-commit hooks against all files:

```bash
pre-commit run --all-files
```

- Format check (same as hook):

```bash
cargo fmt --all -- --check
```

- Run clippy (same as hook):

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

- Run tests and build:

```bash
cargo test
cargo build --release
```

Commit message convention
-------------------------

The repo enforces Conventional Commits for the commit message first line. The commit message checker is `scripts/commit-msg-check.py` and runs as a `commit-msg` hook. Example valid messages:

- `feat(parser): add support for X`
- `fix: correct panic on empty input`

Editor/IDE recommendations
--------------------------

- Visual Studio Code: install the `rust-analyzer` extension for IDE features, and the `rust-lang.rust` extension if you prefer the Rust extension.
- Configure your editor to run `rustfmt` on save and to use the workspace `rust-analyzer` settings if you have a workspace configuration.

CI and automation notes
-----------------------

It's a good idea to run the same pre-commit checks in CI. A simple GitHub Actions workflow can run `pre-commit run --all-files` (or run `cargo fmt -- --check`, `cargo clippy`, `cargo test`) on PRs. If you'd like, I can add a CI workflow that mirrors local hooks.

Troubleshooting
---------------

- If `pre-commit` fails to run system hooks that call `cargo`, ensure `$HOME/.cargo/bin` is on your `PATH` in non-interactive shells (for example, set it in your shell profile).
- If `cargo-audit` is missing the hook will skip it (the pre-commit config makes it optional). To enable it fully, install via `cargo install cargo-audit`.

Where things live
-----------------

- Pre-commit config: `.pre-commit-config.yaml`
- Commit message checker: `scripts/commit-msg-check.py`
- Doc: `docs/pre-commit.md` (conventional commit hook instructions)
- This guide: `docs/dev-setup.md`

If you'd like, I can also add a small `Makefile` with targets like `make fmt`, `make lint`, `make test` to standardize commands across developers.
