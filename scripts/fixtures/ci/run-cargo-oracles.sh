#!/usr/bin/env sh
# T-09-01: cargo fmt/clippy/test unhappy-path oracles on temp workspace copies.
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SRC_WS="${REPO_ROOT}/vortex-core"
TOOLCHAIN="${REPO_ROOT}/rust-toolchain.toml"
RUSTFMT="${REPO_ROOT}/rustfmt.toml"

TMP="$(mktemp -d "${TMPDIR:-/tmp}/ci-cargo-oracle.XXXXXX")"
trap 'rm -rf "${TMP}"' EXIT INT HUP TERM

cp -R "${SRC_WS}" "${TMP}/vortex-core"
cp "${TOOLCHAIN}" "${TMP}/rust-toolchain.toml"
cp "${RUSTFMT}" "${TMP}/rustfmt.toml"

run_in_tmp() {
  (
    cd "${TMP}/vortex-core"
    export CARGO_TARGET_DIR="${TMP}/vortex-core/target"
    "$@"
  )
}

# happy
run_in_tmp cargo fmt --all -- --check
run_in_tmp cargo clippy --workspace -- -D warnings
run_in_tmp cargo test --workspace

# unhappy: formato
printf '\nfn   badly_indented(  ) { }\n' >>"${TMP}/vortex-core/crates/domain/src/lib.rs"
if run_in_tmp cargo fmt --all -- --check 2>/dev/null; then
  printf 'expected fmt --check to fail on bad format\n' >&2
  exit 1
fi

# reset lib for clippy test
printf '//! Crate `domain` (placeholder for T-00-01-02).\n' >"${TMP}/vortex-core/crates/domain/src/lib.rs"
printf '\nfn clippy_bait() {\n    let unused_binding = 1;\n}\n' >>"${TMP}/vortex-core/crates/domain/src/lib.rs"
if run_in_tmp cargo clippy --workspace -- -D warnings 2>/dev/null; then
  printf 'expected clippy -D warnings to fail on unused binding\n' >&2
  exit 1
fi

# reset for test
printf '//! Crate `domain` (placeholder for T-00-01-02).\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn boom() {\n        panic!();\n    }\n}\n' >"${TMP}/vortex-core/crates/domain/src/lib.rs"
if run_in_tmp cargo test --workspace 2>/dev/null; then
  printf 'expected cargo test to fail on panicking test\n' >&2
  exit 1
fi

printf 'check-ci cargo oracles OK\n'
