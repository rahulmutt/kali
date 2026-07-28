#!/usr/bin/env bash
# Re-run the R-35 switch-boundary fixtures under both kali and node.
#
# Exit status is captured UNPIPED for each side: `cmd | tail` makes $? the status
# of `tail` and erases the fail-closed / silent distinction this matrix exists to
# record (register S4, warning 4).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
kali="${KALI:-$repo/target/debug/kali}"
[ -x "$kali" ] || { echo "no binary at $kali -- run: cargo build --bin kali" >&2; exit 1; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
for f in "$here"/*.js "$here"/disc/*.js; do
  [ -e "$f" ] || continue
  timeout 120 "$kali" run "$f" > "$tmp/ko" 2> "$tmp/ke"; kx=$?
  timeout 20  node        "$f" > "$tmp/no" 2> "$tmp/ne"; nx=$?
  echo "===== ${f#$here/}"
  echo "--- kali exit=$kx"; echo "kali_stdout:"; cat "$tmp/ko"
  echo "kali_stderr:"; head -c 600 "$tmp/ke"
  echo "--- node exit=$nx"; echo "node_stdout:"; cat "$tmp/no"
  echo "node_stderr:"; head -c 400 "$tmp/ne"
done
