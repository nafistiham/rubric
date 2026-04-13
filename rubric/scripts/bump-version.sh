#!/usr/bin/env bash
# bump-version.sh <new-version>
# Updates Cargo.toml workspace version, gem version, and CHANGELOG in one shot.
#
# Usage:
#   scripts/bump-version.sh 0.2.0

set -euo pipefail

NEW_VERSION="${1:-}"

if [[ -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <new-version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

# Validate semver format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be in semver format (e.g. 0.2.0)"
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# ── Detect current version ────────────────────────────────────────────────────

OLD_VERSION=$(grep -m1 '^version' rubric-cli/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "Bumping $OLD_VERSION → $NEW_VERSION"

# ── Update Cargo.toml files ───────────────────────────────────────────────────

for toml in rubric-core/Cargo.toml rubric-rules/Cargo.toml rubric-cli/Cargo.toml; do
  sed -i.bak "s/^version = \"${OLD_VERSION}\"/version = \"${NEW_VERSION}\"/" "$toml"
  rm -f "${toml}.bak"
  echo "  updated $toml"
done

# ── Update gem version ────────────────────────────────────────────────────────

GEM_VERSION_FILE="gem/lib/rubric/version.rb"
sed -i.bak "s/VERSION = \"${OLD_VERSION}\"/VERSION = \"${NEW_VERSION}\"/" "$GEM_VERSION_FILE"
rm -f "${GEM_VERSION_FILE}.bak"
echo "  updated $GEM_VERSION_FILE"

# ── Prepend CHANGELOG entry ───────────────────────────────────────────────────

CHANGELOG="CHANGELOG.md"
TODAY=$(date +%Y-%m-%d)

if [[ -f "$CHANGELOG" ]]; then
  # Prepend a new section header after the first line (assumes "# Changelog" on line 1)
  TMP=$(mktemp)
  awk -v ver="$NEW_VERSION" -v date="$TODAY" '
    NR == 1 { print; print ""; print "## [" ver "] - " date; print ""; print "### Added"; print ""; print "- "; next }
    { print }
  ' "$CHANGELOG" > "$TMP"
  mv "$TMP" "$CHANGELOG"
  echo "  prepended entry to $CHANGELOG"
fi

# ── Run cargo update to refresh Cargo.lock ───────────────────────────────────

~/.cargo/bin/cargo update -p rubric-cli -p rubric-rules -p rubric-core 2>/dev/null || true

echo ""
echo "Done. Next steps:"
echo "  1. Fill in the CHANGELOG entry for $NEW_VERSION"
echo "  2. git add -A && git commit -m \"chore: bump version to $NEW_VERSION\""
echo "  3. git tag v$NEW_VERSION && git push origin main v$NEW_VERSION"
