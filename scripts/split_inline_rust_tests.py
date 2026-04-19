#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
import textwrap
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ROOT / "crates"
CFG_TEST_RE = re.compile(r"(?m)^#\[cfg\(test\)\]\s*\nmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")
TEST_ATTR_RE = re.compile(r"(?m)^\s*#\[test\]\s*$")


@dataclass
class InlineTestModule:
    source_path: Path
    module_name: str
    start: int
    end: int
    body: str
    generated_name: str

    @property
    def test_count(self) -> int:
        return len(TEST_ATTR_RE.findall(self.body))


class SplitError(RuntimeError):
    pass


def find_matching_brace(text: str, open_brace_index: int) -> int:
    depth = 0
    i = open_brace_index
    in_string: str | None = None
    raw_hashes = 0
    in_line_comment = False
    in_block_comment_depth = 0

    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""

        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue

        if in_block_comment_depth:
            if ch == "/" and nxt == "*":
                in_block_comment_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                in_block_comment_depth -= 1
                i += 2
                continue
            i += 1
            continue

        if in_string is not None:
            if in_string == '"':
                if ch == "\\":
                    i += 2
                    continue
                if ch == '"':
                    in_string = None
                i += 1
                continue
            if in_string == "raw":
                if ch == '"':
                    end = '"' + ('#' * raw_hashes)
                    if text.startswith(end, i):
                        in_string = None
                        i += len(end)
                        continue
                i += 1
                continue

        if ch == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            in_block_comment_depth = 1
            i += 2
            continue
        if ch == '"':
            in_string = '"'
            i += 1
            continue
        if ch == "r":
            j = i + 1
            hashes = 0
            while j < len(text) and text[j] == "#":
                hashes += 1
                j += 1
            if j < len(text) and text[j] == '"':
                in_string = "raw"
                raw_hashes = hashes
                i = j + 1
                continue

        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1

    raise SplitError("unmatched brace while parsing inline test module")


def generated_name_for(source_path: Path, module_name: str) -> str:
    stem = source_path.stem
    if stem in {"lib", "main", "mod"}:
        return f"{module_name}.rs"
    return f"{stem}_{module_name}.rs"


def collect_inline_test_modules(source_path: Path) -> list[InlineTestModule]:
    text = source_path.read_text()
    modules: list[InlineTestModule] = []
    for match in CFG_TEST_RE.finditer(text):
        module_name = match.group(1)
        open_brace_index = text.find("{", match.end() - 1)
        close_brace_index = find_matching_brace(text, open_brace_index)
        body = text[open_brace_index + 1 : close_brace_index]
        end = close_brace_index + 1
        if end < len(text) and text[end] == "\n":
            end += 1
        modules.append(
            InlineTestModule(
                source_path=source_path,
                module_name=module_name,
                start=match.start(),
                end=end,
                body=body,
                generated_name=generated_name_for(source_path, module_name),
            )
        )
    return modules


def normalize_module_body(body: str) -> str:
    normalized = textwrap.dedent(body)
    normalized = normalized.lstrip("\n")
    return normalized.rstrip() + "\n"


def split_file(source_path: Path, dry_run: bool) -> list[InlineTestModule]:
    text = source_path.read_text()
    modules = collect_inline_test_modules(source_path)
    if not modules:
        return []

    for module in modules:
        target_path = source_path.parent / module.generated_name
        if target_path.exists():
            raise SplitError(f"refusing to overwrite existing file: {target_path}")

    updated = text
    for module in reversed(modules):
        replacement = (
            "#[cfg(test)]\n"
            f"#[path = \"{module.generated_name}\"]\n"
            f"mod {module.module_name};\n"
        )
        updated = updated[: module.start] + replacement + updated[module.end :]

    if not dry_run:
        source_path.write_text(updated)
        for module in modules:
            target_path = source_path.parent / module.generated_name
            target_path.write_text(normalize_module_body(module.body))

    return modules


def rust_sources() -> list[Path]:
    return sorted(CRATES.glob("*/src/**/*.rs"))


def scan() -> list[InlineTestModule]:
    found: list[InlineTestModule] = []
    for path in rust_sources():
        found.extend(collect_inline_test_modules(path))
    return found


def verify_no_inline_test_modules() -> list[InlineTestModule]:
    return scan()


def main() -> int:
    parser = argparse.ArgumentParser(description="Split inline Rust #[cfg(test)] modules into separate files.")
    parser.add_argument("--check", action="store_true", help="verify that no inline #[cfg(test)] modules remain")
    parser.add_argument("--dry-run", action="store_true", help="report changes without writing files")
    args = parser.parse_args()

    if args.check:
        remaining = verify_no_inline_test_modules()
        if remaining:
            print("inline test modules remain:", file=sys.stderr)
            for module in remaining:
                print(f"- {module.source_path.relative_to(ROOT)}::{module.module_name}", file=sys.stderr)
            return 1
        print("No inline #[cfg(test)] modules remain under crates/.")
        return 0

    all_split: list[InlineTestModule] = []
    for source_path in rust_sources():
        modules = split_file(source_path, dry_run=args.dry_run)
        all_split.extend(modules)

    if not all_split:
        print("No inline test modules found.")
        return 0

    total_tests = sum(module.test_count for module in all_split)
    for module in all_split:
        print(
            f"split {module.source_path.relative_to(ROOT)}::{module.module_name} -> "
            f"{module.source_path.parent.relative_to(ROOT) / module.generated_name} "
            f"({module.test_count} #[test] items)"
        )

    print(
        f"Migrated {len(all_split)} inline test modules containing {total_tests} #[test] items."
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except SplitError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
