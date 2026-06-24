# Contributing to bluetui

Thank you for your interest. `bluetui` is a personal/hobby project under the
Spacecraft Software umbrella (see the **Project Posture** section of
[`Readme.md`](Readme.md)). Contributions are welcome but accepted at the
maintainer's discretion.

## Ground rules (from upstream, still binding)

- **No AI slop.** Submit work you understand and have reviewed.
- **Open an issue or discussion first.** Only submit a pull request after a
  prior issue or discussion.
- **Keep PRs small and focused.** One concern per pull request.

## Quality gates

Every change must pass the same gates CI enforces:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
```

Pedantic clippy is enabled (see `Cargo.toml`); do not silence lints with ad-hoc
`#[allow]` — use `#[expect(..., reason = "...")]` where a deviation is justified.

## Spacecraft Software conventions

- **Colors:** never hardcode color literals in UI logic. Use the `Steelbore`
  theme tokens in `src/theme.rs` (Standard §11.1). The audit
  `rg 'Color::(Green|Yellow|Red|Blue|White|DarkGray)|\.(green|yellow|red|blue|white|on_dark_gray)\(' src/`
  must match only `src/theme.rs`.
- **CLI behavior:** the machine surface follows the Spacecraft Software CLI
  Standard — stdout is data only, errors are structured JSON on stderr with a
  runnable `hint`, timestamps are ISO 8601 UTC (`Z`), and every data command
  supports `--json`/`--fields`. See [`AGENTS.md`](AGENTS.md).
- **Time:** use `jiff` and emit ISO 8601 UTC with a `Z` suffix; never local time
  in machine output.

## Sign-off and signed commits

- Sign off your commits (`git commit -s`) to certify the Developer Certificate
  of Origin.
- Commits pushed to the Spacecraft Software remote **must be cryptographically
  signed and show "Verified"** (Standard §6.3). Rebases/amends must preserve
  signatures.

## Licensing of contributions

By contributing, you agree your contributions are licensed under
**GPL-3.0-only** (matching the project), or **GPL-3.0-or-later** for wholly new
files, consistent with the existing per-file SPDX headers.

## Security

Report security issues privately to
`Mohamed.Hammad [at] SpacecraftSoftware.org` rather than opening a public issue.
