<div align="center">
  <img height="125" src="assets/bluetui-logo-anim.svg"/>
  <h2> TUI for managing bluetooth on Linux </h2>
  <img src="https://github.com/user-attachments/assets/f937535d-5675-4427-b347-8086c8830e23"/>
</div>

## 💡 Prerequisites

A Linux based OS with [bluez](https://www.bluez.org/) installed.

> [!NOTE]
> You might need to install [nerdfonts](https://www.nerdfonts.com/) for the icons to be displayed correctly.

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/bluetui/releases)

### 📦 crates.io

You can install `bluetui` from [crates.io](https://crates.io/crates/bluetui)

```shell
cargo install bluetui
```

### 🐧 Arch Linux

You can install `bluetui` from the [extra repository](https://archlinux.org/packages/extra/x86_64/bluetui/):

```shell
pacman -S bluetui
```

### 🐧 Gentoo

```sh
emerge net-wireless/bluetui
```

### 🧰 X-CMD

If you are a user of [x-cmd](https://x-cmd.com), you can run:

```shell
x install bluetui
```

### ⚒️ Build from source

Run the following command:

```shell
git clone https://github.com/pythops/bluetui
cd bluetui
cargo build --release
```

This will produce an executable file at `target/release/bluetui` that you can copy to a directory in your `$PATH`.

## 🪄 Usage

### Global

`Tab` or `l`: Scroll down between different sections.

`shift+Tab` or `h`: Scroll up between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`s`: Start/Stop scanning.

`ctrl+c` or `q`: Quit the app. (Note: `<Esc>` can also quit if `esc_quit = true` is set in config)

### Adapters

`p`: Enable/Disable the pairing.

`o`: Power on/off the adapter.

`d`: Enable/Disable the discovery.

### Paired devices

`u`: Unpair the device.

`Space or Enter`: Connect/Disconnect the device.

`t`: Trust/Untrust the device.

`f`: Favorite/Unfavorite the device.

`e`: Rename the device.

### New devices

`Space or Enter`: Pair the device.

### Editing text (rename, PIN, passkey)

In any text-entry field, standard CUA editing keys work: `Ctrl+C` copy,
`Ctrl+X` cut, `Ctrl+V` paste, `Ctrl+Z` undo, `Ctrl+S` save. List navigation
uses Vim-style `h`/`j`/`k`/`l` alongside the arrow keys.

## Command-line mode

Run bare in a terminal, `bluetui` launches the interactive TUI. With a
noun-verb subcommand it behaves as a structured, scriptable CLI:

```shell
bluetui adapter list --json
bluetui device list --adapter hci0 --fields address,is_connected
bluetui device get AA:BB:CC:DD:EE:FF
bluetui schema        # JSON Schema (Draft 2020-12) of commands and output
bluetui describe      # machine-readable capability manifest
```

Output mode is auto-detected: an interactive terminal gets human output; a pipe
or an agent environment (`AI_AGENT=1`) gets JSON. stdout carries data only —
errors are structured JSON on stderr with a runnable `hint` and a canonical exit
code. Every data command accepts `--json`/`--format`, `--fields`, and the other
global flags listed in `bluetui --help`.

> Read commands plus the write commands (`adapter power|pairable|discoverable`,
> `adapter scan`, and
> `device connect|disconnect|trust|untrust|pair|unpair|favorite|unfavorite|rename`)
> are available. Write commands honor `--dry-run`, and `device unpair` requires
> `--yes` outside a dry run. An MCP server is intentionally out of scope.

## Config

Keybindings can be customized in the default config file location `$HOME/.config/bluetui/config.toml` or from a custom path with `-c`

```toml
# Possible values: "Legacy", "Start", "End", "Center", "SpaceAround", "SpaceBetween"
layout = "SpaceAround"

# Window width
# Possible values: "auto" or a positive integer
width = "auto"

toggle_scanning = "s"
esc_quit = false  # Set to true to enable Esc key to quit the app

[adapter]
toggle_pairing = "p"
toggle_power = "o"
toggle_discovery = "d"

[paired_device]
unpair = "u"
toggle_trust = "t"
toggle_favorite = "f"
rename = "e"

# Canvas background: "navy" (default, Void Navy #000027) or "terminal"
[theme]
background = "navy"
# Optional per-token hex overrides (default to the Steelbore palette):
# foreground = "#D98E32"
# accent     = "#4B7EB0"
# success    = "#50FA7B"
# error      = "#FF5C5C"
# info       = "#8BE9FD"
```

## Contributing

- No AI slop.
- Only submit a pull request after having a prior issue or discussion.
- Keep PRs small and focused.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full guidelines (quality gates,
sign-off, signed commits, and Spacecraft Software conventions).

## 🎁 Note

If you like `bluetui` and you are looking for a TUI to manage WiFi, checkout out [impala](https://github.com/pythops/impala)

## Project Posture

`bluetui` (this fork) is a **Personal / Hobby** project under the
[Spacecraft Software](https://SpacecraftSoftware.org/) umbrella, governed by
The Steelbore Standard. No warranty, no SLAs; contributions are welcome but
accepted at the maintainer's discretion, and forking is encouraged. See
[`NOTICE.md`](NOTICE.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Maintainer

Mohamed Hammad — `Mohamed.Hammad [at] SpacecraftSoftware.org`
· <https://github.com/UnbreakableMJ/bluetui>

This is a fork of [pythops/bluetui](https://github.com/pythops/bluetui) by
Badr Badri; see [`CREDITS.md`](CREDITS.md).

## ⚖️ License

`bluetui` is licensed under **GPL-3.0-only** (GPLv3). Full license texts live in
[`LICENSES/`](LICENSES/), and the project is
[REUSE](https://reuse.software)-compliant — every file carries SPDX tags.
Documentation is licensed `CC-BY-SA-4.0`.

## ✍️ Credits

Bluetui logo: [Marco Bulgarelli](https://github.com/Bugg4)
