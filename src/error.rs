// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Structured CLI error model.
//!
//! The interactive TUI keeps using [`anyhow`](crate::app::AppResult) internally
//! (per Microsoft Rust Guideline M-APP-ERROR). At the command-line boundary,
//! fallible operations are converted into an [`AppError`], which renders as a
//! structured JSON object on stderr in machine mode and as a concise
//! human-readable message on a terminal, per the Spacecraft Software CLI
//! Standard §1 (item 8) and §4 (canonical exit codes).

use serde::Serialize;

use crate::output::OutputMode;

/// Canonical process exit codes (CLI Standard §4).
///
/// Values `6..=125` are tool-specific; `bluetui` documents `10`, `11`, and `12`
/// in its `schema` output.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Success; parse stdout.
    Success = 0,
    /// General failure.
    General = 1,
    /// Usage error (bad arguments); do not retry without fixing the invocation.
    Usage = 2,
    /// Resource not found.
    NotFound = 3,
    /// Permission denied.
    Permission = 4,
    /// Conflict (already exists).
    Conflict = 5,
    /// Tool-specific: the BlueZ/D-Bus session could not be opened.
    BluetoothUnavailable = 10,
    /// Tool-specific: the Bluetooth device is soft- or hard-blocked by rfkill.
    RfkillBlocked = 11,
    /// Tool-specific: the configuration file is missing or invalid.
    ConfigInvalid = 12,
    /// Internal error (unexpected failure inside `bluetui`).
    Internal = 70,
}

impl ExitCode {
    /// Returns the numeric exit code.
    #[must_use]
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Machine-readable error code (CLI Standard §1, exit-codes-errors reference).
#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The requested resource does not exist.
    NotFound,
    /// The caller lacks permission for the operation.
    PermissionDenied,
    /// An argument value was malformed.
    InvalidArgument,
    /// A required argument was absent.
    MissingArgument,
    /// The operation conflicts with existing state.
    Conflict,
    /// A required external dependency is missing.
    DependencyMissing,
    /// An unexpected internal failure occurred.
    InternalError,
    /// The BlueZ/D-Bus session is unavailable.
    BluetoothUnavailable,
    /// The Bluetooth device is blocked by rfkill.
    RfkillBlocked,
    /// The configuration is missing or invalid.
    ConfigInvalid,
    /// A requested feature (e.g. an output format) is not yet implemented.
    FeatureUnavailable,
}

/// A structured, agent-friendly error.
///
/// The `hint` field is always a runnable command ("tips-thinking"), never prose,
/// so an agent can act on a failure without guessing.
#[derive(Debug, Serialize)]
pub struct AppError {
    /// Stable machine-readable error code.
    pub code: ErrorCode,
    /// The numeric process exit code.
    pub exit_code: i32,
    /// Human-readable description of what went wrong.
    pub message: String,
    /// A runnable command that helps recover from the error.
    pub hint: String,
    /// ISO 8601 UTC timestamp of when the error was produced.
    pub timestamp: String,
    /// The canonical command that failed, e.g. `bluetui adapter get`.
    pub command: String,
    /// Optional documentation URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

impl AppError {
    /// Builds a new error with the given code, exit code, message, and runnable hint.
    #[must_use]
    pub fn new(
        code: ErrorCode,
        exit: ExitCode,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code,
            exit_code: exit.as_i32(),
            message: message.into(),
            hint: hint.into(),
            timestamp: crate::time::now_iso8601(),
            command: String::new(),
            docs_url: None,
        }
    }

    /// Sets the canonical command string that produced this error.
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    /// Writes the error to stderr in the form appropriate for `mode` and returns
    /// the numeric exit code. Machine modes emit a single-line `{"error": …}`
    /// object; human modes emit a concise `error:`/`hint:` pair.
    #[must_use]
    pub fn emit(&self, mode: OutputMode) -> i32 {
        if mode.is_machine() {
            let payload = serde_json::json!({ "error": self });
            if let Ok(line) = serde_json::to_string(&payload) {
                eprintln!("{line}");
            }
        } else {
            eprintln!("error: {}", self.message);
            if !self.hint.is_empty() {
                eprintln!("       hint: {}", self.hint);
            }
        }
        self.exit_code
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<anyhow::Error> for AppError {
    /// Wraps an internal `anyhow` error (e.g. from the TUI event loop) as a
    /// generic internal failure. Command boundaries that need a specific code
    /// should use [`IntoAppError`] instead.
    fn from(error: anyhow::Error) -> Self {
        Self::new(
            ErrorCode::InternalError,
            ExitCode::Internal,
            error.to_string(),
            String::new(),
        )
    }
}

/// Converts an `anyhow`-based result into a [`Result<T, AppError>`] at the CLI
/// boundary, attaching a machine code, exit code, and runnable hint.
pub trait IntoAppError<T> {
    /// Maps the error case into an [`AppError`].
    ///
    /// # Errors
    ///
    /// Returns an [`AppError`] carrying `code`, `exit`, and `hint` when `self`
    /// is `Err`.
    fn into_app(
        self,
        code: ErrorCode,
        exit: ExitCode,
        hint: impl Into<String>,
    ) -> Result<T, AppError>;
}

impl<T> IntoAppError<T> for anyhow::Result<T> {
    fn into_app(
        self,
        code: ErrorCode,
        exit: ExitCode,
        hint: impl Into<String>,
    ) -> Result<T, AppError> {
        self.map_err(|e| AppError::new(code, exit, e.to_string(), hint))
    }
}

// Rust guideline compliant 2026-05-18
