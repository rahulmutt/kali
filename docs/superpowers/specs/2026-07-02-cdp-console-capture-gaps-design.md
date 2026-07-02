# CDP driver console-capture gaps — design

Date: 2026-07-02
Status: approved-pending-user-review
Scope: test-only (`crates/kali_cli/tests/cdp_driver/`, gated browser tests)

## Problem

The test-only CDP smoke driver has gaps in how it observes `console.log`
output from pages running in real Chromium:

1. **Event-drop race (primary).** `CdpBrowser::call()` discards every event
   frame that arrives while it waits for a command response. `run_page()`
   awaits the `Page.navigate` response through `call()`, but the page starts
   executing before that response is read. Any `Runtime.consoleAPICalled` —
   and even the `Runtime.bindingCalled` completion signal — emitted in that
   window is silently swallowed. Symptoms: missing console lines, or a
   spurious full-timeout run reported as `completed: false`.
2. **`stdout()` is not node-parity.** It documents "node-style stdout" but
   includes only `log` lines; node prints `info` and `debug` to stdout too.
3. **No session filtering.** `run_page()` accepts console/binding events from
   any session, so another target's output could bleed into the captured
   console or falsely signal completion.
4. **Uncaught page errors are invisible.** `Runtime.exceptionThrown` is not
   captured, so a page that crashes before logging yields an empty console
   and an unexplained timeout.

Out of scope (recorded, not fixed here): the production browser-glue gap
noted in `browser_cdp_smoke.rs` — `console.log` from a bare top-level
`main()` program does not route through the glue's `console_log` import in a
browser; only exported-function calls do. That is production code and
deserves its own spec.

## Approach

Chosen: **pending-event buffer** on `CdpBrowser`. Alternatives considered:
restructuring `run_page` into a unified send-then-pump loop (fixes only the
navigate window, duplicates response matching) and a background reader
thread (correct but structurally heavy for a test-only driver).

## Design

### 1. Event buffering

`CdpBrowser` gains `pending_events: VecDeque<CdpIncoming>`. In `call()`, the
discard arm changes: `CdpIncoming::Event` frames are pushed onto
`pending_events`; unmatched result/error frames are still discarded.
`run_page()`'s event loop pops `pending_events` first and falls back to
`conn.read()` when the buffer is empty. This fixes the race for every
`call()` made while a page is live, not just `Page.navigate`.

### 2. Session filtering

`run_page()` only accepts `Runtime.consoleAPICalled`, `Runtime.bindingCalled`
and `Runtime.exceptionThrown` events whose `session_id` equals the page's
session. Non-matching events are ignored silently.

### 3. Exception capture

`Runtime.exceptionThrown` (same session) is recorded into `outcome.console`
as `CdpConsoleLine { kind: "exception", text }`, where `text` combines
`exceptionDetails.text` with the exception object's description when present.
Exception lines never appear in `stdout()`; they exist so failing runs are
diagnosable from existing assertion messages that print `outcome.console`.

### 4. stdout/stderr node parity

- `stdout()` includes kinds `log`, `info`, `debug` (node's stdout set).
- New `stderr()` includes kinds `warn`, `error` (node's stderr set).

### 5. Testing

- Extract the per-event routing decision into a pure helper (event JSON +
  expected session → console line / completed / ignore) and unit-test it
  without Chromium: session mismatch ignored, exception captured, binding
  recognized, console kinds preserved.
- Unit-test `stdout()`/`stderr()` over hand-built `CdpPageOutcome` values.
- Add one gated Chromium test: a `data:` URL page whose inline script logs
  synchronously at document start and immediately calls the completion
  binding — the exact shape the race swallowed. Assert all lines and
  `completed: true`.
- Existing gated tests (`mise run browser-smoke`) must keep passing.

## Error handling

Unchanged: timeout ends the page loop with `completed: false`;
transport/protocol errors propagate. Pending events buffered at
`close()`/`Drop` are dropped with the browser — acceptable for a per-test
driver.
