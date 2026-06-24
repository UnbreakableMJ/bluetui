# AGENTS.md

Project-specific guidance for AI coding agents working in `bluetui`. This
complements the Spacecraft Software CLI Standard (which agents already carry as
a skill) — it records only what is specific to *this* repository.

## What this is

`bluetui` is a dual-mode Rust tool for managing Bluetooth on Linux:

- **No subcommand + interactive terminal** → launches the ratatui TUI (the
  human experience).
- **Noun-verb subcommand** → a structured, machine-readable CLI.

## Build / test / lint

```sh
cargo build --release                                              # release binary
cargo test                                                         # unit + snapshot + CLI tests
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Requires BlueZ + D-Bus + `pkg-config` on the host (`libdbus` is vendored).

## CLI contract (don't break these)

- **stdout is data only; stderr is everything else.** Never print logs/banners to stdout.
- Every data command supports `--json` and `--fields`. The JSON envelope is
  `{ "metadata": {...}, "data": ... }` with snake_case keys and an ISO 8601 UTC
  (`Z`) `metadata.timestamp`.
- Errors in machine mode are a single-line `{"error": {...}}` on stderr, with a
  **runnable** `hint` (a command, not prose) and a canonical `exit_code`
  (see `bluetui schema` → `x-exit-codes`).
- Output mode is resolved once in `main()` via `output::resolve_mode` (flags →
  `AI_AGENT`/`AGENT`/`CI` → TTY → pipe). The TUI must never run under an agent
  env or a non-TTY.
- `bluetui schema` is valid JSON Schema Draft 2020-12; `bluetui describe`
  is the capability manifest. Both are pure (no BlueZ) and safe to invoke.

## Invariants

- **No bare color literals in UI logic.** Use `Steelbore` theme tokens
  (`src/theme.rs`). Audit: `rg 'Color::(Green|Yellow|Red|Blue|White|DarkGray)' src/` → only `theme.rs`.
- Internal TUI errors use `anyhow` (`AppResult`); the CLI boundary converts to
  the typed `error::AppError` (carries code/exit/hint). Don't leak `anyhow`
  strings into machine output.
- Timestamps via `jiff`, ISO 8601 UTC `Z` only (`src/time.rs`).
- Concurrency is a single-threaded tokio runtime (`current_thread`) on purpose —
  the workload is I/O-bound D-Bus calls; blocking BlueZ work is `tokio::spawn`ed
  and reports back over the event channel.
- Commits to the Spacecraft Software remote must be signed and "Verified".

## Phase status

Phase 1 (current): read-only commands (`adapter list|get`, `device list|get`),
`schema`, `describe`. Mutating verbs (connect/pair/trust/power/scan/rename/favorite)
and the `bluetui mcp` server are planned for later phases — the command tree is
shaped so they slot in without breaking the envelope or exit-code map.
