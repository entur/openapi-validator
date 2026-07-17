#!/usr/bin/env bash
# Bumps the GUI cask's version and per-arch sha256 checksums after a release.
#
# Usage: update-cask.sh <cask-file> <version> <dist-dir> <asset-prefix>
#
# <dist-dir> must contain <asset-prefix>-<version>-<target>.dmg.sha256 for
# each of: aarch64-apple-darwin (arm), x86_64-apple-darwin (intel).
set -euo pipefail

cask="$1"
version="$2"
dist_dir="$3"
asset_prefix="$4"

labels=(arm intel)
target_for() {
  case "$1" in
    arm) echo aarch64-apple-darwin ;;
    intel) echo x86_64-apple-darwin ;;
  esac
}

# Validate every checksum file exists before editing anything, so a missing
# artifact aborts the whole script instead of writing a partially-updated
# (or garbage) cask.
for label in "${labels[@]}"; do
  sha_file="${dist_dir}/${asset_prefix}-${version}-$(target_for "$label").dmg.sha256"
  if [[ ! -f "$sha_file" ]]; then
    echo "::error::missing checksum file ${sha_file}" >&2
    exit 1
  fi
done

perl -pi -e "s/^  version \".*\"\$/  version \"${version}\"/" "$cask"

for label in "${labels[@]}"; do
  sha="$(cut -d' ' -f1 "${dist_dir}/${asset_prefix}-${version}-$(target_for "$label").dmg.sha256")"
  perl -pi -e "s/(${label}:\\s*\")[a-f0-9]{64}(\")/\${1}${sha}\${2}/" "$cask"
done

echo "Updated ${cask} to ${version}"
