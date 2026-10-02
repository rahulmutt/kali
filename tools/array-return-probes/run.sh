#!/usr/bin/env bash
# Node-vs-kali probe runner for the array-return project
# (docs/superpowers/specs/2026-10-02-array-return-design.md §4.1).
# Usage: tools/array-return-probes/run.sh OUT.tsv
set -u
here="$(cd "$(dirname "$0")" && pwd)"
kali="${KALI:-$here/../../target/debug/kali}"
out="$1"
: > "$out"
for p in "$here"/probes/*.js; do
  name="$(basename "$p" .js)"
  node_out="$(node "$p" 2>&1)"
  kali_out="$("$kali" run "$p" 2>/tmp/array-return-probe.err)"
  kali_exit=$?
  kali_err="$(cat /tmp/array-return-probe.err)"
  if [ $kali_exit -eq 0 ] && [ "$kali_out" = "$node_out" ]; then
    verdict=CORRECT
  elif [ $kali_exit -ne 0 ] && grep -q E5506 /tmp/array-return-probe.err; then
    verdict=REFUSES
  elif [ $kali_exit -ne 0 ] && grep -q 'kali: array index out of bounds' /tmp/array-return-probe.err; then
    verdict=TRAPS
  elif [ $kali_exit -eq 0 ]; then
    verdict=SILENT
  else
    verdict=OTHER
  fi
  shown="$kali_out"
  [ $kali_exit -ne 0 ] && shown="$(printf '%s' "$kali_err" | head -1)"
  printf '%s\t%s\t%s\t%s\n' "$name" "$verdict" \
    "$(printf '%s' "$node_out" | tr '\n' '|')" \
    "$(printf '%s' "$shown" | tr '\n' '|')" >> "$out"
done
