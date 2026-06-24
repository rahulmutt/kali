No renames — test basename set identical.

Note: co-location only changed module-path PREFIXES (e.g. `execute_tests::test_foo` →
`execute::execute_tests::test_foo`), which `--list` includes by design. A raw `--list`
diff is therefore non-empty, but the basename set (suffix after the final `::`) is
identical — confirming zero test renames, drops, or duplications across the refactor.
