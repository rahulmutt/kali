#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../proofs"

expected_files=(
  "BOUNDARY.md"
  "lean-toolchain"
  "lakefile.lean"
  "KaliCore.lean"
  "KaliCore/Types.lean"
  "KaliCore/Semantics.lean"
  "KaliCore/Soundness.lean"
  "KaliCore/Safety.lean"
  "KaliIR.lean"
  "KaliIR/HIRModel.lean"
  "KaliIR/LoweringCorrectness.lean"
)

mapfile -t actual_files < <(
  find . \
    \( -path './.lake' -o -path './build' \) -prune -o \
    -type f \
    \( -name '*.lean' -o -name 'BOUNDARY.md' -o -name 'lakefile.lean' -o -name 'lean-toolchain' \) \
    -print \
    | sed 's#^./##' \
    | sort
)

mapfile -t expected_sorted < <(printf '%s\n' "${expected_files[@]}" | sort)

if ! diff -u <(printf '%s\n' "${expected_sorted[@]}") <(printf '%s\n' "${actual_files[@]}"); then
  echo "Unexpected proof tree layout under proofs/"
  exit 1
fi

mapfile -t expected_roots < <(
  printf '%s\n' "${expected_files[@]}" \
    | awk -F/ 'NF > 1 { print $1 }' \
    | sort -u
)
mapfile -t actual_roots < <(
  sed -n 's/^lean_lib[[:space:]]\+\([A-Za-z0-9_][A-Za-z0-9_]*\).*/\1/p' lakefile.lean | sort -u
)

if ! diff -u <(printf '%s\n' "${expected_roots[@]}") <(printf '%s\n' "${actual_roots[@]}"); then
  echo "proof lakefile roots do not match the proof source directories"
  exit 1
fi

for file in "${expected_files[@]}"; do
  case "$file" in
    BOUNDARY.md|lean-toolchain)
      continue
      ;;
    *)
      if ! grep -Fq "$file" BOUNDARY.md; then
        echo "proof boundary does not mention $file"
        exit 1
      fi
      ;;
  esac
done
