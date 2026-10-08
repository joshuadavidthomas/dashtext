set dotenv-load
set unstable

# List all available commands
[private]
default:
    @just --list

build *ARGS:
    cargo build {{ ARGS }}

run *ARGS:
    cargo run --release -- {{ ARGS }}

check *ARGS:
    cargo check --workspace --locked --all-targets --all-features {{ ARGS }}

clippy *ARGS:
    cargo clippy --workspace --locked --all-targets --all-features {{ ARGS }} -- -D warnings

# rustfmt and cargo-hawk run on the toolchains pinned under tools/; see devenv.nix.

fmt *ARGS:
    @# stable rustfmt skips the unstable options with a warning and passes --check anyway
    @"${RUSTFMT:-rustfmt}" --version | grep -q nightly || { echo "error: just fmt needs the nightly rustfmt from devenv; run it in devenv shell" >&2; exit 1; }
    cargo fmt --manifest-path "{{ justfile_directory() }}/Cargo.toml" --all {{ ARGS }}

[positional-arguments]
hawk *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo hawk check \
        --manifest-path "{{ justfile_directory() }}/Cargo.toml" \
        --target-dir "{{ justfile_directory() }}/target/hawk" \
        -D warnings "$@"

# Run all pre-commit hooks against the repository.
lint *ARGS:
    @just --fmt
    prek run --all-files --show-diff-on-failure --color always {{ ARGS }}

test *ARGS:
    cargo test --workspace --locked --all-features {{ ARGS }}
