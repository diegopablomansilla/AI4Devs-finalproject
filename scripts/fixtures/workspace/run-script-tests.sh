#!/usr/bin/env sh
# T-00-04 / T-00-05 / T-00-08: check-workspace allowlist for domain, storage and vortex.
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT="${REPO_ROOT}/scripts/check-workspace.sh"

assert_ok() {
  label=$1
  root=$2
  if ! CHECK_ROOT="${root}" "${SCRIPT}"; then
    printf 'workspace fixture: expected success for %s\n' "${label}" >&2
    exit 1
  fi
}

assert_fails_with() {
  label=$1
  root=$2
  needle=$3
  set +e
  out=$(CHECK_ROOT="${root}" "${SCRIPT}" 2>&1)
  status=$?
  set -e
  if [ "${status}" -eq 0 ]; then
    printf 'workspace fixture: expected failure for %s\n' "${label}" >&2
    exit 1
  fi
  if ! printf '%s' "${out}" | grep -F "${needle}" >/dev/null; then
    printf 'workspace fixture: %s missing needle %s in:\n%s\n' "${label}" "${needle}" "${out}" >&2
    exit 1
  fi
}

copy_workspace() {
  tmp="${REPO_ROOT}/.fixture-tmp/workspace-$$"
  rm -rf "${tmp}"
  mkdir -p "${tmp}/vortex-core"
  cp "${REPO_ROOT}/vortex-core/Cargo.toml" "${tmp}/vortex-core/"
  for member in api domain catalog search recommend ai speech vision sync storage supervisor; do
    mkdir -p "${tmp}/vortex-core/crates/${member}/src"
    cp "${REPO_ROOT}/vortex-core/crates/${member}/Cargo.toml" "${tmp}/vortex-core/crates/${member}/" 2>/dev/null || true
    cp "${REPO_ROOT}/vortex-core/crates/${member}/src/lib.rs" "${tmp}/vortex-core/crates/${member}/src/" 2>/dev/null || true
  done
  mkdir -p "${tmp}/vortex-core/bin/vortex/src"
  cp "${REPO_ROOT}/vortex-core/bin/vortex/Cargo.toml" "${tmp}/vortex-core/bin/vortex/"
  cp "${REPO_ROOT}/vortex-core/bin/vortex/src/main.rs" "${tmp}/vortex-core/bin/vortex/src/"
  printf '%s' "${tmp}"
}

# happy: repo with domain allowlist
assert_ok 'repo con domain allowlist' "${REPO_ROOT}"

tmp=$(copy_workspace)
cleanup() {
  rm -rf "${tmp}"
}
trap cleanup EXIT HUP INT TERM

# happy: domain declares full allowlist
cat >"${tmp}/vortex-core/crates/domain/Cargo.toml" <<'EOF'
[package]
name = "domain"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
thiserror = { workspace = true }
ulid = { workspace = true }
rust_decimal = { workspace = true }
serde = { workspace = true }

[dev-dependencies]
serde_json = { workspace = true }

[lints]
workspace = true
EOF
assert_ok 'domain con allowlist completa' "${tmp}"

# unhappy: domain declares clap
cat >"${tmp}/vortex-core/crates/domain/Cargo.toml" <<'EOF'
[package]
name = "domain"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
thiserror = { workspace = true }
clap = { workspace = true }

[lints]
workspace = true
EOF
assert_fails_with 'domain con clap' "${tmp}" 'domain'

# unhappy: api declares dependencies
cp "${REPO_ROOT}/vortex-core/crates/domain/Cargo.toml" "${tmp}/vortex-core/crates/domain/Cargo.toml"
cat >"${tmp}/vortex-core/crates/api/Cargo.toml" <<'EOF'
[package]
name = "api"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
serde = { workspace = true }

[lints]
workspace = true
EOF
assert_fails_with 'api con dependencies' "${tmp}" 'api'

# happy: storage declares surrealdb allowlist
cp "${REPO_ROOT}/vortex-core/crates/api/Cargo.toml" "${tmp}/vortex-core/crates/api/Cargo.toml"
cp "${REPO_ROOT}/vortex-core/crates/domain/Cargo.toml" "${tmp}/vortex-core/crates/domain/Cargo.toml"
cat >"${tmp}/vortex-core/crates/storage/Cargo.toml" <<'EOF'
[package]
name = "storage"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
surrealdb = { workspace = true, default-features = false, features = ["kv-rocksdb"] }
thiserror = { workspace = true }
sha2 = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }

[lints]
workspace = true
EOF
assert_ok 'storage con allowlist surrealdb' "${tmp}"

# unhappy: storage declares axum
cat >"${tmp}/vortex-core/crates/storage/Cargo.toml" <<'EOF'
[package]
name = "storage"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
surrealdb = { workspace = true, features = ["kv-rocksdb"] }
axum = { workspace = true }

[lints]
workspace = true
EOF
assert_fails_with 'storage con axum' "${tmp}" 'storage'

# happy: vortex declares config allowlist
cp "${REPO_ROOT}/vortex-core/crates/api/Cargo.toml" "${tmp}/vortex-core/crates/api/Cargo.toml"
cp "${REPO_ROOT}/vortex-core/crates/domain/Cargo.toml" "${tmp}/vortex-core/crates/domain/Cargo.toml"
cp "${REPO_ROOT}/vortex-core/crates/storage/Cargo.toml" "${tmp}/vortex-core/crates/storage/Cargo.toml"
cp "${REPO_ROOT}/vortex-core/bin/vortex/Cargo.toml" "${tmp}/vortex-core/bin/vortex/Cargo.toml"
assert_ok 'vortex con allowlist de configuración' "${tmp}"

# unhappy: vortex declares metrics
cat >"${tmp}/vortex-core/bin/vortex/Cargo.toml" <<'EOF'
[package]
name = "vortex"
version = "0.1.0"
edition.workspace = true
publish = false

[dependencies]
toml = { workspace = true }
metrics = { workspace = true }

[lints]
workspace = true
EOF
assert_fails_with 'vortex con metrics' "${tmp}" 'vortex'

printf 'workspace fixture: all scenarios passed\n'
