#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=0
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.project-setup/logs"
LOG_FILE="$LOG_DIR/verify-project.log"
failures=0

usage() {
  echo "Usage: $0 [--dry-run]"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 1 ;;
  esac
done

log() {
  echo "$1"
  if [ "$DRY_RUN" -eq 0 ]; then
    mkdir -p "$LOG_DIR"
    printf "%s %s\n" "$(date '+%Y-%m-%d %H:%M:%S %Z')" "$1" >> "$LOG_FILE"
  fi
}

check_file() {
  if [ -f "$ROOT_DIR/$1" ]; then
    log "OK: $1"
  else
    log "MISSING: $1"
    failures=$((failures + 1))
  fi
}

for path in \
  docs/setup.md \
  docs/local-assumptions.md \
  docs/reproducibility.md \
  .env.example \
  scripts/check-prereqs.sh \
  scripts/bootstrap-project.sh \
  scripts/verify-project.sh \
  .hekton/project.yaml; do
  check_file "$path"
done

# Task T01 (2026-07-14) made this a real Cargo workspace. Presence checks alone would
# pass on a machine with no Rust toolchain at all — actually run the workspace's own
# verify command (matches the DAG's verify_command in .control-tower/sessions/T01.jsonl).
if [ -f "$ROOT_DIR/Cargo.toml" ]; then
  if [ "$DRY_RUN" -eq 1 ]; then
    log "DRY-RUN: would run cargo build --locked && cargo fmt --check in $ROOT_DIR"
  elif (cd "$ROOT_DIR" && cargo build --locked && cargo fmt --all --check); then
    log "OK: cargo build --locked && cargo fmt --check"
  else
    log "MISSING: cargo build --locked && cargo fmt --check did not pass"
    failures=$((failures + 1))
  fi
fi

if [ "$failures" -ne 0 ]; then
  log "Verification failed with $failures issue(s)"
  exit 1
fi

log "Verification passed"
