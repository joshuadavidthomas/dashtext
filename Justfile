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

rustfmt_channel := `sed -n 's/^channel = "\([^"]*\)"/\1/p' tools/rustfmt/rust-toolchain.toml`

fmt *ARGS:
    cargo "+{{ rustfmt_channel }}" fmt --manifest-path "{{ justfile_directory() }}/Cargo.toml" --all {{ ARGS }}

# cargo-hawk must run on the toolchain it was built against.
# Keep this paired with the Hawk version in mise.toml.
hawk_channel := `sed -n 's/^channel = "\([^"]*\)"/\1/p' tools/hawk/rust-toolchain.toml`

[positional-arguments]
hawk *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo "+{{ hawk_channel }}" hawk check \
        --manifest-path "{{ justfile_directory() }}/Cargo.toml" \
        --target-dir "{{ justfile_directory() }}/target/hawk" \
        -D warnings "$@"

# Run all pre-commit hooks against the repository.
lint *ARGS:
    @just --fmt
    prek run --all-files --show-diff-on-failure --color always {{ ARGS }}

test *ARGS:
    cargo test --workspace --locked --all-features {{ ARGS }}
