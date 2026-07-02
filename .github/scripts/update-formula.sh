#!/usr/bin/env bash
# Bumps a Homebrew formula's version and sha256 checksums after a release.
#
# Usage: update-formula.sh <formula-file> <version> <dist-dir> <asset-prefix>
#
# <dist-dir> must contain <asset-prefix>-<version>-<target>.tar.gz.sha256 for
# each of: x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu.
set -euo pipefail

formula="$1"
version="$2"
dist_dir="$3"
asset_prefix="$4"

targets=(x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu)

# Validate every checksum file exists before editing anything, so a missing
# artifact aborts the whole script instead of writing a partially-updated
# (or garbage) formula. Checking this inside a function called via command
# substitution wouldn't work: `exit` there only kills the subshell.
for target in "${targets[@]}"; do
  sha_file="${dist_dir}/${asset_prefix}-${version}-${target}.tar.gz.sha256"
  if [[ ! -f "$sha_file" ]]; then
    echo "::error::missing checksum file ${sha_file}" >&2
    exit 1
  fi
done

sha_for() {
  cut -d' ' -f1 "${dist_dir}/${asset_prefix}-${version}-${1}.tar.gz.sha256"
}

set_sha() {
  local target="$1"
  local sha="$2"
  perl -0777 -pi -e "s/(url \"[^\"]*${target}\\.tar\\.gz\"\\s*\\n\\s*sha256 \")[a-f0-9]+(\")/\${1}${sha}\${2}/" "$formula"
}

perl -pi -e "s/^  version \".*\"\$/  version \"${version}\"/" "$formula"

for target in "${targets[@]}"; do
  set_sha "$target" "$(sha_for "$target")"
done

echo "Updated ${formula} to ${version}"
