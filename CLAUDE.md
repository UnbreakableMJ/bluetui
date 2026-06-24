# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`bluetui` is a Rust TUI for managing Bluetooth on Linux: `ratatui` + `crossterm` for the UI, the `bluer` crate (BlueZ D-Bus bindings) for Bluetooth. License GPL-3.0, edition 2024.

This checkout is a **fork** — `origin` is `UnbreakableMJ/bluetui`, `upstream` is `pythops/bluetui` (the canonical project). The README's contributing rules are explicit and binding: **"No AI slop. Only submit a pull request after having a prior issue or discussion. Keep PRs small and focused."** These norms are stricter than the umbrella defaults — honor them.

## Commands

Needs BlueZ, D-Bus, and `pkg-config` on the host (`libdbus` is vendored via the `libdbus-sys` "vendored" feature). On Nix, `nix develop` provides the dev shell; the repo uses direnv (`.envrc` is `use flake`).

- Build (release): `cargo build --release` → `target/release/bluetui`
- Run: `cargo run -- [-c <config_path>]`
- Test: `cargo test`
- Run one test: `cargo test confirmation` / `cargo test render` — tests live in `#[cfg(test)]` modules and `cargo test <substring>` filters by the full path
- Lint (must be clean — CI and pre-commit deny warnings): `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all`
- Nix build: `nix build`

Clippy runs with `pedantic` on plus a curated allow/deny list in `Cargo.toml` (`[lints.clippy]`); `trivially_copy_pass_by_ref` is denied. Match that posture rather than adding ad-hoc `#[allow]`s. `.pre-commit-config.yaml` and `.github/workflows/ci.yaml` both run fmt + clippy + test (fail-fast / on push+PR to `master`) — run all three before pushing.

## Snapshot tests (insta)

UI tests render a widget to a `ratatui` `TestBackend` and snapshot the terminal buffer with `insta` (`assert_snapshot!`), parameterized via `rstest`. `.snap` files live in `src/snapshots/` and `src/requests/snapshots/`.

The help and request widgets are snapshotted across **terminal-width breakpoints** (80/81, 120/121) and every `FocusedBlock` — this is how the responsive layout (e.g. the help banner switching form at width 120) is pinned. Layout or help-text changes will break these snapshots by design. Review/accept with `cargo insta review` (or `cargo insta accept`); without `cargo-insta`, set `INSTA_UPDATE=always` or rename the generated `.snap.new` files.

## Architecture

The whole app is one `tokio` **current-thread** runtime (`#[tokio::main(flavor = "current_thread")]`) driving a single ratatui draw/event loop. Bluetooth work never blocks the UI: BlueZ operations are `tokio::spawn`ed and report back over a channel.

**The event channel is the spine.** `event::EventHandler` spawns one task that merges a 1s tick timer with crossterm's `EventStream` into an `mpsc::UnboundedSender<Event>`. The `Event` enum carries *both* terminal events (`Tick`, `Key`, `Resize`, …) and application/agent events (`Notification`, `NewPairedDevice`, `ToggleFavorite`, `RequestConfirmation`, `PinCodeSumitted`, …). Every async path — spawned BlueZ tasks and the pairing-agent callbacks — pushes state changes back to the main loop by `send`ing on this channel. The big `match` in `main.rs` is the application-level state machine that consumes those events.

The main loop (`main.rs`): `tui.draw(&app)` → `tui.events.next()` → dispatch. `Event::Key` is delegated to `handler::handle_key_events`; every other variant mutates `App` directly in `main.rs`.

**`App` (`app.rs`)** owns all state and all rendering (`render_controllers`, `render_paired_devices`, `render_new_devices`). `App::tick()` (fired every `Tick`) ages out notifications, advances the spinner, and calls `refresh()`, which re-reads all controllers/devices from BlueZ and *reconciles* them into existing state — preserving table selection and handling adapters being plugged/unplugged (e.g. resume from suspend). `FocusedBlock` is the central UI mode enum; it drives both render highlighting and which key-handling branch runs.

**`handler.rs`** (`handle_key_events`) is the key dispatcher. It branches on `app.focused_block` **first** — modal request dialogs (`RequestConfirmation`, `EnterPinCode`, `SetDeviceAliasBox`, …) intercept keys before the global bindings — then on key code. Mutating BlueZ actions (connect, trust, pair, power/pairable/discoverable toggles) are `tokio::spawn`ed and surface success/failure as `Notification`s on the event channel; they do **not** mutate `App` directly (the next `refresh()` reflects the change). Most keybindings come from `config` (chars), not hardcoded literals.

**`bluetooth.rs`** is the domain/read model over `bluer`. `Controller` wraps an `Adapter`; `Device` wraps a BlueZ `Device`. `Controller::get_all` is the sole read path from BlueZ. Devices split into `paired_devices` vs `new_devices`; discovered devices whose alias is just a MAC address sort to the bottom. `Device::get_icon` maps freedesktop icon names → Nerd Font glyphs (hence the nerdfonts prerequisite).

**Pairing-agent bridge (`agent.rs` + `requests/`).** `AuthAgent` holds `async_channel` sender/receiver pairs, one per interaction type (confirmation, pin, passkey, display-pin, display-passkey) plus a cancel channel, and is registered as a BlueZ `Agent` on the `Session` at startup. When BlueZ invokes a callback (e.g. `request_confirmation`), the callback sends a `Request*` `Event` to the UI (so the modal renders) and then `select!`s on the matching receiver for the user's answer or a cancel. The UI side (`requests/<kind>.rs`) is a self-contained widget with `new` / `render` / `submit` / `cancel`; `submit`/`cancel` send the answer back over the `AuthAgent` channel, unblocking the callback. `Requests` (`requests.rs`) holds an `Option<…>` per modal — at most one is `Some` at a time.

**Persistence & startup.** `favorite.rs` stores favorited device addresses one-per-line in `$XDG_DATA_HOME/bluetui/favorites.txt` (read at startup, written on quit). `config.rs` parses `$XDG_CONFIG_HOME/bluetui/config.toml` (or `-c <path>`) via serde + custom deserializers for `Width` (`"auto"` | u16), `layout` (ratatui `Flex` variant names), and the `[theme]` section. `Config::new` and `rfkill::check` return a structured `error::AppError` (not `exit(1)`); the resolved output mode decides whether a startup failure renders as human prose or JSON-on-stderr. `tui.rs` handles terminal raw-mode/alt-screen setup and teardown plus a panic hook that restores the terminal.

## Dual-mode CLI, theme & compliance (Spacecraft Software)

This fork is governed by The Steelbore Standard and the Spacecraft Software CLI Standard. Beyond the TUI above, the binary is now **dual-mode** (see `AGENTS.md` for the full contract):

- **Entry point (`main.rs`).** `output::resolve_mode` runs first; then a three-way branch: a noun-verb subcommand → `commands::dispatch` (machine output); `schema`/`describe` → pure introspection (no BlueZ); no subcommand → `run_tui` (the event loop, unchanged from upstream). The TUI never runs under `AI_AGENT`/`AGENT`/`CI` or a non-TTY.
- **CLI layer.** `cli.rs` (clap: `Args` + flattened `GlobalFlags` + `Command` tree), `output.rs` (mode cascade + `Response<T>` envelope + `--fields` projection), `error.rs` (`AppError`/`ErrorCode`/`ExitCode`, `IntoAppError` anyhow bridge), `dto.rs` (serializable `AdapterDto`/`DeviceDto` — the live `Controller`/`Device` are not serializable), `bluetooth_query.rs` (read-only session reusing `Controller::get_all`), `commands.rs` (read + write handlers + hand-built JSON Schema/manifest), `time.rs` (jiff, ISO 8601 UTC `Z`). Phases 1–2 implement read and write commands; an MCP server is intentionally out of scope.
- **Theme (`theme.rs`).** The `Steelbore` theme is the single source of truth for colors (Standard §11.1) — **no bare color literals in UI logic** (audit: the `rg` in `CONTRIBUTING.md` must match only `theme.rs`). `App` owns a `Theme`, built from `config.theme.to_theme()`, threaded by `&Theme` into every render fn. Background defaults to Void Navy, configurable to terminal-default.
- **Text editing (`text_edit.rs`).** `handle_cua` provides CUA `Ctrl+C/X/V/Z/S` for the rename/PIN/passkey fields, with an app-local clipboard (no system-clipboard dep, per PFA §9).
- **Snapshots.** `TestBackend` snapshots capture the character grid only (not colors), so theme changes don't churn them; layout/help-text changes do. CLI behavior is covered by `tests/cli.rs`.
- **Licensing/REUSE.** Project is `GPL-3.0-only` (preserving upstream); new original files carry `GPL-3.0-or-later` SPDX tags; docs are `CC-BY-SA-4.0`. Coverage via per-file headers + `REUSE.toml`; `reuse lint` is a gate. Texinfo manual is `doc/bluetui.texi` (`make info|html|pdf`); packaging defs in `packaging/`.
