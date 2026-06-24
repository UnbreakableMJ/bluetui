# Credits

`bluetui` builds substantially on the work of others. This file is the
human-readable record of that debt (The Steelbore Standard §15.3); the
machine-readable license metadata lives in per-file SPDX headers,
[`REUSE.toml`](REUSE.toml), and [`Cargo.toml`](Cargo.toml).

## Fork base

| Field | Value |
|-------|-------|
| Name | bluetui |
| Author | Badr Badri ([@pythops](https://github.com/pythops)) |
| License | GPL-3.0 |
| Source | <https://github.com/pythops/bluetui> |
| Scope | This project is a fork of pythops/bluetui. The TUI, BlueZ integration, pairing-agent flow, and event loop originate upstream; the Spacecraft Software work adds a dual-mode CLI, the Steelbore theme, CUA keybindings, structured errors, and compliance scaffolding. |

## Logo

| Field | Value |
|-------|-------|
| Name | bluetui logo (`assets/bluetui-logo-anim.svg`) |
| Author | Marco Bulgarelli ([@Bugg4](https://github.com/Bugg4)) |
| Scope | Project logo and animated banner. |

## Notable dependencies

These are surfaced mechanically through Cargo and are listed here only for
convenience — their licenses are not redistributed by this project:

- [`bluer`](https://github.com/bluez/bluer) — BlueZ D-Bus bindings.
- [`ratatui`](https://github.com/ratatui/ratatui) + `crossterm` — terminal UI.
- [`clap`](https://github.com/clap-rs/clap) — argument parsing.
- [`tui-input`](https://github.com/sayanarijit/tui-input) — text input widget.
- [`jiff`](https://github.com/BurntSushi/jiff) — ISO 8601 UTC timestamps.
