// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Output-mode resolution and machine-readable rendering.
//!
//! Implements the Spacecraft Software CLI Standard §5 (output-mode detection
//! cascade) and §6 (the JSON envelope). stdout carries data only; diagnostics
//! and errors go to stderr.

use std::io::{IsTerminal, Write};

use serde::Serialize;
use serde_json::Value;

use crate::cli::{ColorWhen, Format, GlobalFlags};
use crate::error::{AppError, ErrorCode, ExitCode};

pub(crate) const TOOL: &str = env!("CARGO_PKG_NAME");
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MAINTAINER: &str = "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>";
pub(crate) const WEBSITE: &str = "https://github.com/UnbreakableMJ/bluetui";

/// The resolved output personality for one invocation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-readable output with ANSI color.
    HumanColor,
    /// Human-readable output without color.
    HumanPlain,
    /// Structured JSON document.
    Json,
    /// JSON Lines (one record per line).
    Jsonl,
    /// YAML document.
    Yaml,
    /// CSV.
    Csv,
    /// Interactive terminal UI.
    Explore,
}

impl OutputMode {
    /// Returns `true` for machine-readable formats (json/jsonl/yaml/csv).
    #[must_use]
    pub fn is_machine(self) -> bool {
        matches!(self, Self::Json | Self::Jsonl | Self::Yaml | Self::Csv)
    }

    /// Returns `true` when the interactive TUI should run.
    #[must_use]
    pub fn is_tui(self) -> bool {
        matches!(self, Self::Explore)
    }

    /// Returns `true` for the human-readable terminal formats.
    #[must_use]
    pub fn is_human(self) -> bool {
        matches!(self, Self::HumanColor | Self::HumanPlain)
    }
}

/// Resolves the output mode from flags, environment, and the terminal, applying
/// the CLI Standard §5 cascade (first match wins).
#[must_use]
pub fn resolve_mode(globals: &GlobalFlags) -> OutputMode {
    // 1. Explicit flag (`--json` is an alias for `--format json`).
    let explicit = if globals.json {
        Some(Format::Json)
    } else {
        globals.format
    };
    if let Some(fmt) = explicit {
        return match fmt {
            Format::Json => OutputMode::Json,
            Format::Jsonl => OutputMode::Jsonl,
            Format::Yaml => OutputMode::Yaml,
            Format::Csv => OutputMode::Csv,
            Format::Explore => resolve_explore(),
        };
    }

    // 2. Agent / CI environment forces non-interactive JSON.
    if is_agent_env() || is_ci_env() {
        return OutputMode::Json;
    }

    // 3/4. Interactive terminal => human; otherwise machine JSON.
    if std::io::stdout().is_terminal() {
        if use_color(globals) {
            OutputMode::HumanColor
        } else {
            OutputMode::HumanPlain
        }
    } else {
        OutputMode::Json
    }
}

/// Guards `--format explore`: never trap an agent or a non-interactive shell in
/// the TUI. Falls back to JSON with a one-line warning on stderr.
fn resolve_explore() -> OutputMode {
    if is_agent_env() || is_dumb_term() || !std::io::stdout().is_terminal() {
        eprintln!(
            r#"{{"warning":"--format explore is unavailable without an interactive terminal; falling back to --format json"}}"#
        );
        OutputMode::Json
    } else {
        OutputMode::Explore
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| !v.is_empty() && v != "0" && v != "false")
}

/// Returns `true` when an agent environment variable (`AI_AGENT`/`AGENT`) is set.
#[must_use]
pub fn is_agent_env() -> bool {
    env_truthy("AI_AGENT") || env_truthy("AGENT")
}

fn is_ci_env() -> bool {
    env_truthy("CI")
}

fn is_dumb_term() -> bool {
    std::env::var("TERM").as_deref() == Ok("dumb")
}

/// Identifies a known invoking agent for telemetry only; this never changes the
/// output format (CLI Standard §5 / agentic §4).
fn invoking_agent() -> Option<String> {
    for (var, label) in [
        ("CLAUDECODE", "claude-code"),
        ("CURSOR_AGENT", "cursor"),
        ("GEMINI_CLI", "gemini-cli"),
    ] {
        if env_truthy(var) {
            return Some(label.to_owned());
        }
    }
    None
}

fn use_color(globals: &GlobalFlags) -> bool {
    if globals.no_color {
        return false;
    }
    match globals.color {
        ColorWhen::Always => return true,
        ColorWhen::Never => return false,
        ColorWhen::Auto => {}
    }
    if env_truthy("FORCE_COLOR") {
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("CLICOLOR").as_deref() == Ok("0") {
        return false;
    }
    if is_dumb_term() {
        return false;
    }
    std::io::stdout().is_terminal()
}

/// The CLI Standard §6 response envelope: `{ metadata, data }`.
#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    /// Provenance and timing metadata.
    pub metadata: Metadata,
    /// The command payload.
    pub data: T,
}

impl<T: Serialize> Response<T> {
    /// Builds a response wrapping `data`, stamping the canonical `command`.
    pub fn new(command: impl Into<String>, data: T) -> Self {
        Self {
            metadata: Metadata::new(command.into()),
            data,
        }
    }
}

/// Envelope metadata (CLI Standard §6).
#[derive(Debug, Serialize)]
pub struct Metadata {
    /// The tool name (`bluetui`).
    pub tool: &'static str,
    /// The tool version.
    pub version: &'static str,
    /// The canonical command, e.g. `bluetui adapter list`.
    pub command: String,
    /// ISO 8601 UTC timestamp of the response.
    pub timestamp: String,
    /// Maintainer attribution (Standard §15).
    pub maintainer: &'static str,
    /// Project URL (Standard §15).
    pub website: &'static str,
    /// The detected invoking agent, when known (telemetry only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_agent: Option<String>,
}

impl Metadata {
    fn new(command: String) -> Self {
        Self {
            tool: TOOL,
            version: VERSION,
            command,
            timestamp: crate::time::now_iso8601(),
            maintainer: MAINTAINER,
            website: WEBSITE,
            tool_agent: invoking_agent(),
        }
    }
}

/// Renders a [`Response`] to stdout in the given mode, applying optional
/// `--fields` projection.
///
/// # Errors
///
/// Returns an [`AppError`] if serialization or writing fails, or if the
/// requested format is not yet implemented (yaml/csv in Phase 1).
pub fn render<T: Serialize>(
    resp: &Response<T>,
    mode: OutputMode,
    fields: Option<&[String]>,
    print0: bool,
) -> Result<(), AppError> {
    let mut value = serde_json::to_value(resp).map_err(serialize_err)?;
    if let Some(fields) = fields {
        project_fields(&mut value, fields);
    }
    emit_value(&value, mode, print0)
}

/// Writes a raw JSON [`Value`] to stdout in the given mode (used by `schema`,
/// which emits a bare JSON Schema rather than an enveloped response).
///
/// # Errors
///
/// Returns an [`AppError`] on serialization/IO failure or an unsupported format.
pub fn emit_value(value: &Value, mode: OutputMode, print0: bool) -> Result<(), AppError> {
    match mode {
        OutputMode::Jsonl => emit_jsonl(value, print0),
        OutputMode::Yaml | OutputMode::Csv => Err(format_unsupported(mode)),
        // Machine JSON is compact; human modes pretty-print the same document.
        _ => emit_json(value, !mode.is_machine()),
    }
}

fn emit_json(value: &Value, pretty: bool) -> Result<(), AppError> {
    let rendered = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(serialize_err)?;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    writeln!(lock, "{rendered}").map_err(io_err)
}

fn emit_jsonl(value: &Value, print0: bool) -> Result<(), AppError> {
    let data = value.get("data").unwrap_or(value);
    let terminator = if print0 { '\0' } else { '\n' };
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    match data {
        Value::Array(items) => {
            for item in items {
                let line = serde_json::to_string(item).map_err(serialize_err)?;
                write!(lock, "{line}{terminator}").map_err(io_err)?;
            }
        }
        other => {
            let line = serde_json::to_string(other).map_err(serialize_err)?;
            write!(lock, "{line}{terminator}").map_err(io_err)?;
        }
    }
    Ok(())
}

/// Retains only the requested `fields` within each record of `data.*`, leaving
/// `metadata` intact (CLI Standard §6 token economy).
fn project_fields(value: &mut Value, fields: &[String]) {
    let Some(data) = value.get_mut("data") else {
        return;
    };
    match data {
        Value::Array(items) => {
            for item in items.iter_mut() {
                retain_fields(item, fields);
            }
        }
        object @ Value::Object(_) => retain_fields(object, fields),
        _ => {}
    }
}

fn retain_fields(value: &mut Value, fields: &[String]) {
    if let Value::Object(map) = value {
        map.retain(|key, _| fields.iter().any(|field| field == key));
    }
}

fn serialize_err(e: serde_json::Error) -> AppError {
    AppError::new(
        ErrorCode::InternalError,
        ExitCode::Internal,
        format!("failed to serialize output: {e}"),
        "bluetui describe --json",
    )
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::new(
        ErrorCode::InternalError,
        ExitCode::Internal,
        format!("failed to write output: {e}"),
        String::new(),
    )
}

fn format_unsupported(mode: OutputMode) -> AppError {
    AppError::new(
        ErrorCode::FeatureUnavailable,
        ExitCode::General,
        format!("output format {mode:?} is not yet implemented in this release"),
        "bluetui --json <command>",
    )
}

// Rust guideline compliant 2026-05-18
