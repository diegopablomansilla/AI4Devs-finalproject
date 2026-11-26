#!/usr/bin/env sh
# T-00-02-01: cargo tree oracles for domain-deps fixtures (red/green harness).
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCENARIOS="${SCRIPT_DIR}/scenarios"

oracle_tree_lists() {
  root=$1
  needle=$2
  manifest="${root}/vortex-core/Cargo.toml"
  cargo tree -q --manifest-path "${manifest}" -p domain -e normal,build --prefix none --format '{p}' \
    | awk -F ' ' '{print $1}' | grep -Fx "${needle}"
}

oracle_tree_missing() {
  root=$1
  needle=$2
  manifest="${root}/vortex-core/Cargo.toml"
  ! cargo tree -q --manifest-path "${manifest}" -p domain -e normal,build --prefix none --format '{p}' \
    | awk -F ' ' '{print $1}' | grep -Fx "${needle}"
}

oracle_no_domain_fails() {
  root=$1
  manifest="${root}/vortex-core/Cargo.toml"
  if cargo tree -q --manifest-path "${manifest}" -p domain -e normal,build >/dev/null 2>&1; then
    printf 'expected cargo tree -p domain to fail for no-domain fixture\n' >&2
    return 1
  fi
}

printf 'oracle surrealdb-direct...\n'
oracle_tree_lists "${SCENARIOS}/surrealdb-direct" surrealdb

printf 'oracle reqwest-direct...\n'
oracle_tree_lists "${SCENARIOS}/reqwest-direct" reqwest

printf 'oracle reqwest-transitive...\n'
oracle_tree_lists "${SCENARIOS}/reqwest-transitive" reqwest

printf 'oracle storage-member...\n'
oracle_tree_lists "${SCENARIOS}/storage-member" storage

printf 'oracle no-domain...\n'
oracle_no_domain_fails "${SCENARIOS}/no-domain"

printf 'oracle surrealdb-dev-only...\n'
oracle_tree_missing "${SCENARIOS}/surrealdb-dev-only" surrealdb

printf 'all domain-deps oracles OK\n'
