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

grep -Fq 'lean_lib KaliCore' lakefile.lean
grep -Fq 'lean_lib KaliIR' lakefile.lean
