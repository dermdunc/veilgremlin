#!/usr/bin/env bash
# sync-mirror-to-vault.sh: Scoped, deliberate sync of this project's mirror
# summary files into the live Obsidian vault.
#
# This is not autonomous vault mutation. It copies only mirror-owned summaries
# for one project, shows the diff, and stages/commits only that vault folder.
#
# Usage:
#   scripts/sync-mirror-to-vault.sh
#   scripts/sync-mirror-to-vault.sh --dry-run
#   scripts/sync-mirror-to-vault.sh --no-commit
set -uo pipefail

DRY_RUN=false
DO_COMMIT=true
case "${1:-}" in
  --dry-run) DRY_RUN=true ;;
  --no-commit) DO_COMMIT=false ;;
  "") ;;
  *) echo "Unknown option: $1" >&2; exit 1 ;;
esac

VAULT_ROOT="$HOME/vaults/hekton-mind-palace"

ROOT=""
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while [[ "$dir" != "/" ]]; do
  if [[ -f "$dir/.hekton/project.yaml" ]]; then ROOT="$dir"; break; fi
  dir="$(dirname "$dir")"
done
if [[ -z "$ROOT" ]]; then
  echo "ERROR: no .hekton/project.yaml found above this script." >&2
  exit 1
fi

MP_REL="$(grep '^mind_palace_path:' "$ROOT/.hekton/project.yaml" | sed 's/mind_palace_path: *//' | tr -d '"')"
PROJECT="$(grep '^project_name:' "$ROOT/.hekton/project.yaml" | sed 's/project_name: *//' | tr -d '"')"
MIRROR="$ROOT/mind-palace/${MP_REL}"
VAULT="$VAULT_ROOT/${MP_REL}"

echo "=== Mirror to live vault sync: $PROJECT ==="
echo "  mirror: $MIRROR"
echo "  vault:  $VAULT"
echo ""

if [[ ! -d "$MIRROR" ]]; then
  echo "ERROR: mirror not found: $MIRROR" >&2
  exit 1
fi
if [[ ! -d "$VAULT_ROOT/.git" ]]; then
  echo "ERROR: vault is not a git repo: $VAULT_ROOT" >&2
  exit 1
fi

mkdir -p "$VAULT"

if [[ -x "$ROOT/scripts/check-mirror-drift.sh" ]]; then
  if ! "$ROOT/scripts/check-mirror-drift.sh" --check >/dev/null 2>&1; then
    echo "Mirror has drifted from repo docs. Refresh the mirror before syncing."
    echo "Run: scripts/check-mirror-drift.sh"
    exit 1
  fi
fi

# Boundary rule (2026-06-28): vault holds the card + session-log only.
# decisions.md is repo source-of-truth, linked from the card — not mirrored.
SUMMARY_FILES=(index.md session-log.md)
CHANGED=()

for f in "${SUMMARY_FILES[@]}"; do
  src="$MIRROR/$f"
  dst="$VAULT/$f"
  [[ -f "$src" ]] || { echo "  skip (no mirror $f)"; continue; }
  if [[ -f "$dst" ]] && diff -q "$src" "$dst" >/dev/null; then
    echo "  unchanged: $f"
    continue
  fi
  CHANGED+=("$f")
  echo "  -- diff: $f --"
  diff "$dst" "$src" 2>/dev/null | sed 's/^/    /' | head -40 || true
  [[ "$DRY_RUN" == false ]] && cp "$src" "$dst"
done

echo ""
if [[ "${#CHANGED[@]}" -eq 0 ]]; then
  echo "Vault already current for $PROJECT. Nothing to do."
  exit 0
fi

if [[ "$DRY_RUN" == true ]]; then
  echo "(dry-run) would update: ${CHANGED[*]}"
  exit 0
fi

git -C "$VAULT_ROOT" add "${MP_REL}/index.md" "${MP_REL}/decisions.md" "${MP_REL}/session-log.md" 2>/dev/null
echo "Staged, scoped to $PROJECT:"
git -C "$VAULT_ROOT" diff --cached --name-only | sed 's/^/  /'
echo ""

if [[ "$DO_COMMIT" == true ]]; then
  git -C "$VAULT_ROOT" commit -q -m "sync($PROJECT): refresh control-plane summaries from mirror

Scoped mirror-to-vault sync: index, decisions, session-log for $PROJECT only.
Other in-flight vault changes left untouched."
  echo "Committed scoped vault update for $PROJECT."
else
  echo "Staged but not committed (--no-commit). Review and commit when ready."
fi
