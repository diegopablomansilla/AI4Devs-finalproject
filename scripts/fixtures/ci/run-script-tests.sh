#!/usr/bin/env sh
# T-09-01: check-ci.sh integration tests against fixtures.
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT="${REPO_ROOT}/scripts/check-ci.sh"
FIXTURES="$(cd "$(dirname "$0")" && pwd)"
SCENARIOS="${FIXTURES}/scenarios"

assert_fails_with() {
  root=$1
  needle=$2
  out=$(mktemp "${TMPDIR:-/tmp}/cci-out.XXXXXX")
  trap 'rm -f "${out}"' EXIT INT HUP TERM
  if CHECK_ROOT="${root}" "${SCRIPT}" 2>"${out}"; then
    printf 'expected failure for %s\n' "${root}" >&2
    exit 1
  fi
  if ! grep -qF "${needle}" "${out}"; then
    printf 'stderr missing %s for %s:\n' "${needle}" "${root}" >&2
    cat "${out}" >&2
    exit 1
  fi
}

assert_ok() {
  root=$1
  CHECK_ROOT="${root}" "${SCRIPT}"
}

assert_fails_with "${SCENARIOS}/no-workflow" '.github/workflows/ci.yml'
assert_fails_with "${SCENARIOS}/missing-fmt-job" 'job: fmt'
assert_fails_with "${SCENARIOS}/ubuntu-runner" ubuntu
assert_fails_with "${SCENARIOS}/no-pr-trigger" pull_request
assert_fails_with "${SCENARIOS}/bad-fmt-command" 'fmt --all'
assert_fails_with "${SCENARIOS}/redocly-extra" redocly

CHECK_ROOT="${REPO_ROOT}" "${SCRIPT}"

printf 'check-ci script tests OK\n'
