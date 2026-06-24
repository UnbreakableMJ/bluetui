// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

//! Command-line argument parsing.
//!
//! `bluetui` is a dual-mode tool (Spacecraft Software CLI Standard). With no
//! subcommand it launches the interactive TUI; with a noun-verb subcommand it
//! behaves as a structured, machine-readable CLI. Every global flag mandated by
//! the Standard §3 is accepted on both the bare invocation and every subcommand.

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Attribution footer shown by `--help` and `--version` (Standard §15.2).
const ATTRIBUTION: &str = "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>\nhttps://github.com/UnbreakableMJ/bluetui";

/// `--version` output: the crate version followed by the attribution footer.
const VERSION_LONG: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nMaintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
    "\nhttps://github.com/UnbreakableMJ/bluetui"
);

/// Top-level parsed arguments.
#[derive(Parser, Debug)]
#[command(
    name = "bluetui",
    version = VERSION_LONG,
    about = "TUI for managing bluetooth on Linux",
    long_about = None,
    after_help = ATTRIBUTION
)]
pub struct Args {
    /// Flags shared by the TUI and every subcommand.
    #[command(flatten)]
    pub globals: GlobalFlags,

    /// A noun-verb subcommand. When absent, the interactive TUI launches.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Global flags accepted everywhere (CLI Standard §3).
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is a distinct CLI Standard §3 global flag"
)]
#[derive(ClapArgs, Debug, Clone)]
pub struct GlobalFlags {
    /// Path to the configuration file (TUI keybindings, layout, theme).
    #[arg(short, long, global = true)]
    pub config_path: Option<PathBuf>,

    /// Shortcut for `--format json`.
    #[arg(long, global = true)]
    pub json: bool,

    /// Output format for data-returning commands.
    #[arg(long, global = true, value_enum)]
    pub format: Option<Format>,

    /// Restrict output to the listed fields (comma-separated), reducing token cost.
    #[arg(long, global = true, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Emit an action plan instead of performing side effects.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Increase diagnostic output on stderr.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-error stderr output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable ANSI color (equivalent to `--color never`).
    #[arg(long, global = true)]
    pub no_color: bool,

    /// When to use ANSI color.
    #[arg(long, global = true, value_enum, default_value = "auto")]
    pub color: ColorWhen,

    /// Render absolute UTC timestamps in human mode instead of relative time.
    #[arg(long, global = true)]
    pub absolute_time: bool,

    /// Use NUL record separators for filename-safe piping.
    #[arg(short = '0', long, global = true)]
    pub print0: bool,

    /// Skip interactive confirmation in non-TTY mode.
    #[arg(long = "yes", visible_alias = "force", global = true)]
    pub yes: bool,
}

/// Machine-readable output formats (`--format`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Structured JSON document (default machine format).
    Json,
    /// JSON Lines: one record per line, for streaming.
    Jsonl,
    /// YAML document.
    Yaml,
    /// Comma-separated values.
    Csv,
    /// Interactive terminal UI (the default human experience).
    Explore,
}

/// Color policy (`--color`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    /// Decide based on the output mode and terminal.
    Auto,
    /// Always emit color.
    Always,
    /// Never emit color.
    Never,
}

/// The noun-verb command tree. Phase 1 exposes read-only data commands plus
/// introspection; mutating verbs and the MCP surface land in later phases.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Bluetooth adapter (controller) operations.
    Adapter {
        /// The adapter verb to run.
        #[command(subcommand)]
        verb: AdapterCommand,
    },
    /// Bluetooth device operations.
    Device {
        /// The device verb to run.
        #[command(subcommand)]
        verb: DeviceCommand,
    },
    /// Print the JSON Schema (Draft 2020-12) describing commands and output.
    Schema(SchemaArgs),
    /// Print the machine-readable capability manifest.
    Describe,
}

/// Verbs under `bluetui adapter`.
#[derive(Subcommand, Debug)]
pub enum AdapterCommand {
    /// List all Bluetooth adapters.
    #[command(visible_alias = "ls")]
    List,
    /// Show one adapter by name (e.g. `hci0`), including its devices.
    Get {
        /// Adapter name, e.g. `hci0`.
        name: String,
    },
}

/// Verbs under `bluetui device`.
#[derive(Subcommand, Debug)]
pub enum DeviceCommand {
    /// List devices, optionally scoped to one adapter.
    #[command(visible_alias = "ls")]
    List {
        /// Restrict to a single adapter by name (e.g. `hci0`).
        #[arg(long)]
        adapter: Option<String>,
    },
    /// Show one device by address (`AA:BB:CC:DD:EE:FF`).
    Get {
        /// Device address, e.g. `AA:BB:CC:DD:EE:FF`.
        address: String,
    },
}

/// Arguments for `bluetui schema`.
#[derive(ClapArgs, Debug)]
pub struct SchemaArgs {
    /// Optional noun to scope the schema (e.g. `adapter`).
    pub noun: Option<String>,
    /// Optional verb to scope the schema (e.g. `list`).
    pub verb: Option<String>,
}

// Rust guideline compliant 2026-05-18
