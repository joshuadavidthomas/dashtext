set dotenv-load
set unstable

export PATH := env_var("HOME") + "/.cargo/bin:" + env_var("PATH")
manifest := justfile_directory() / "Cargo.toml"

# List all available commands
[private]
default:
    @just --list

build *ARGS:
    cargo build --manifest-path "{{ manifest }}" {{ ARGS }}

run *ARGS:
    cargo run --manifest-path "{{ manifest }}" --release -- {{ ARGS }}

check *ARGS:
    cargo check --manifest-path "{{ manifest }}" --workspace --locked --all-targets --all-features {{ ARGS }}

clippy *ARGS:
    cargo clippy --manifest-path "{{ manifest }}" --workspace --locked --all-targets --all-features {{ ARGS }} -- -D warnings

rustfmt_channel := `sed -n 's/^channel = "\([^"]*\)"/\1/p' tools/rustfmt/rust-toolchain.toml`

fmt *ARGS:
    cargo "+{{ rustfmt_channel }}" fmt --manifest-path "{{ manifest }}" --all {{ ARGS }}

# cargo-hawk must run on the toolchain it was built against.
# Keep this paired with the Hawk version in mise.toml.
hawk_channel := `sed -n 's/^channel = "\([^"]*\)"/\1/p' tools/hawk/rust-toolchain.toml`

[positional-arguments]
hawk *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo "+{{ hawk_channel }}" hawk check \
        --manifest-path "{{ manifest }}" \
        --target-dir "{{ justfile_directory() }}/target/hawk" \
        -D warnings "$@"

# Run all pre-commit hooks against the repository.
lint *ARGS:
    @just --fmt
    prek run --all-files --show-diff-on-failure --color always {{ ARGS }}

test *ARGS:
    cargo test --manifest-path "{{ manifest }}" --workspace --locked --all-features {{ ARGS }}
