#!/usr/bin/env bash
# publish-crate.sh — publish ONE workspace crate to crates.io, idempotently, and do
# not return until crates.io actually serves that version.
#
# WHY this exists as a script rather than inline workflow YAML: every crates.io HTTP
# query needs a descriptive User-Agent. A bare curl gets 403, and the release workflow
# previously inlined that bare curl in four places, where it caused two separate
# failures that both looked like something else:
#
#   * the "already published, skip upload" guard could never see a 200, so it was
#     unreachable — making a re-run of an already-published version fail on upload;
#   * the dependency-visibility wait polled for a 200 that could never arrive, so it
#     timed out on every single release and the last crate was never published.
#
# Keeping one implementation means the User-Agent cannot be correct in some copies and
# missing in others.
#
# Usage: scripts/publish-crate.sh <crate-name>
# Env:   CARGO_REGISTRY_TOKEN must already be available to cargo (see `cargo login`).

set -euo pipefail

CRATE="${1:?usage: publish-crate.sh <crate-name>}"

readonly USER_AGENT="dig-network-release (github.com/DIG-Network/chia-block-listener)"
readonly VISIBILITY_ATTEMPTS=60
readonly VISIBILITY_INTERVAL_SECONDS=5

# Resolve a crate's version BY NAME.
#
# Reading packages[0] — as this workflow used to — returns whichever member cargo
# happens to list first, which is the root crate no matter which directory the step
# runs in. That is only ever correct while every workspace member shares one version,
# and fails silently the moment they diverge, publishing each crate under the root
# crate's number.
version_of() {
  cargo metadata --no-deps --format-version 1 |
    python3 -c "
import json, sys
packages = json.load(sys.stdin)['packages']
match = next((p for p in packages if p['name'] == '$1'), None)
if match is None:
    sys.exit(\"no workspace member named '$1'\")
print(match['version'])
"
}

# 200 means this exact version is live. Every other outcome — 404 not yet indexed, a
# 5xx, a connection failure — means "not confirmed", never "definitely absent".
is_published() {
  local code
  code="$(curl -sS -o /dev/null -w '%{http_code}' -A "$USER_AGENT" \
    "https://crates.io/api/v1/crates/$1/$2" || echo "000")"
  [ "$code" = "200" ]
}

await_published() {
  local crate="$1" version="$2" attempt
  for attempt in $(seq 1 "$VISIBILITY_ATTEMPTS"); do
    if is_published "$crate" "$version"; then
      echo "crates.io is serving ${crate} ${version}."
      return 0
    fi
    echo "Waiting for crates.io to serve ${crate} ${version}... (${attempt}/${VISIBILITY_ATTEMPTS})"
    sleep "$VISIBILITY_INTERVAL_SECONDS"
  done

  echo "::error::crates.io did not serve ${crate} ${version} within $((VISIBILITY_ATTEMPTS * VISIBILITY_INTERVAL_SECONDS))s." >&2
  return 1
}

VERSION="$(version_of "$CRATE")"
echo "Crate: ${CRATE}  Version: ${VERSION}"

if is_published "$CRATE" "$VERSION"; then
  echo "Already published: ${CRATE} ${VERSION} — nothing to upload."
  exit 0
fi

# Validate packaging and that the crate builds from its packaged form before uploading.
cargo publish -p "$CRATE" --dry-run

cargo publish -p "$CRATE" --no-verify

# Do not return until the registry actually serves it. The next crate in the release
# order depends on this one, and cargo resolves that dependency from the index — so
# returning early here is what turns a propagation delay into a failed release.
await_published "$CRATE" "$VERSION"
