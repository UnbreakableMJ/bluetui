---
name: bluetui
description: >-
  Dual-mode CLI + TUI for managing Bluetooth on Linux (BlueZ). With no
  subcommand it launches an interactive terminal UI; with a noun-verb
  subcommand it emits structured, machine-readable output (JSON envelope,
  structured errors, canonical exit codes) for agents and automation.
license: GPL-3.0-only
---

# bluetui

A Spacecraft Software dual-mode Bluetooth manager. Use the noun-verb CLI for
scripting and agents; run it bare in a terminal for the interactive TUI.

## Capability surface (Phase 1, read-only)

| Command | Purpose |
|---------|---------|
| `bluetui adapter list` | List all Bluetooth adapters (controllers). |
| `bluetui adapter get <name>` | Show one adapter (e.g. `hci0`) with its devices. |
| `bluetui device list [--adapter <name>]` | List devices, optionally scoped to an adapter. |
| `bluetui device get <addr>` | Show one device by address (`AA:BB:CC:DD:EE:FF`). |
| `bluetui schema [noun [verb]]` | JSON Schema (Draft 2020-12) of commands and output. |
| `bluetui describe` | Machine-readable capability manifest. |

Mutating verbs (connect, pair, trust, power, scan, rename, favorite) and an MCP
server (`bluetui mcp`) are planned for later phases.

## Global flags

`--json` · `--format json|jsonl|yaml|csv|explore` · `--fields a,b` · `--dry-run`
· `--verbose` · `--quiet` · `--no-color` · `--color auto|always|never`
· `--absolute-time` · `--print0` · `--yes`/`--force` · `--config-path <path>`

## Conventions

- stdout is data only; diagnostics and errors go to stderr.
- JSON envelope: `{ "metadata": {...}, "data": ... }`; ISO 8601 UTC (`Z`) timestamps.
- Errors: single-line `{"error": {code, exit_code, message, hint, ...}}` on
  stderr, where `hint` is a runnable command.
- Exit codes: `0` ok, `2` usage, `3` not-found, `10` bluetooth-unavailable,
  `11` rfkill-blocked, `12` invalid-config (full map in `bluetui schema`).
- Set `AI_AGENT=1` to force non-interactive JSON output.

## Examples

```sh
bluetui adapter list --json
bluetui device list --adapter hci0 --fields address,is_connected
bluetui device get AA:BB:CC:DD:EE:FF --json
bluetui schema | jaq '."x-commands"'
AI_AGENT=1 bluetui describe
```
