// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Read-only CLI command handlers (Phase 1).
//!
//! Each handler returns the numeric exit code on success or an [`AppError`] on
//! failure. Data commands reuse [`bluetooth_query`] and project onto the DTOs in
//! [`crate::dto`]; `schema`/`describe` are pure and never touch BlueZ.

use std::str::FromStr;

use bluer::Address;
use serde_json::{Value, json};

use crate::bluetooth_query;
use crate::cli::{AdapterCommand, Command, DeviceCommand, GlobalFlags, SchemaArgs};
use crate::dto::{AdapterDto, DeviceDto};
use crate::error::{AppError, ErrorCode, ExitCode, IntoAppError};
use crate::output::{self, MAINTAINER, OutputMode, Response, TOOL, VERSION, WEBSITE};

/// Hint pointing at the most useful recovery command for BlueZ failures.
const BLUEZ_HINT: &str = "rfkill list bluetooth";

/// Routes a parsed [`Command`] to its handler.
///
/// # Errors
///
/// Returns an [`AppError`] when the command fails; the caller emits it and uses
/// the carried exit code.
pub async fn dispatch(
    command: &Command,
    globals: &GlobalFlags,
    mode: OutputMode,
) -> Result<i32, AppError> {
    match command {
        Command::Adapter { verb } => adapter(verb, globals, mode).await,
        Command::Device { verb } => device(verb, globals, mode).await,
        Command::Schema(args) => schema(args, mode, globals.print0),
        Command::Describe => describe(globals, mode),
    }
}

async fn adapter(
    verb: &AdapterCommand,
    globals: &GlobalFlags,
    mode: OutputMode,
) -> Result<i32, AppError> {
    let controllers = load_controllers().await?;
    match verb {
        AdapterCommand::List => {
            let data: Vec<AdapterDto> = controllers.iter().map(AdapterDto::summary).collect();
            let response = Response::new("bluetui adapter list", data);
            output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
        }
        AdapterCommand::Get { name } => {
            let Some(controller) = controllers.iter().find(|c| &c.name == name) else {
                return Err(AppError::new(
                    ErrorCode::NotFound,
                    ExitCode::NotFound,
                    format!("adapter {name:?} not found"),
                    "bluetui adapter list --json",
                )
                .with_command("bluetui adapter get"));
            };
            let response = Response::new("bluetui adapter get", AdapterDto::detailed(controller));
            output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
        }
    }
    Ok(ExitCode::Success.as_i32())
}

async fn device(
    verb: &DeviceCommand,
    globals: &GlobalFlags,
    mode: OutputMode,
) -> Result<i32, AppError> {
    match verb {
        DeviceCommand::List { adapter } => {
            let controllers = load_controllers().await?;
            if let Some(name) = adapter
                && !controllers.iter().any(|c| &c.name == name)
            {
                return Err(AppError::new(
                    ErrorCode::NotFound,
                    ExitCode::NotFound,
                    format!("adapter {name:?} not found"),
                    "bluetui adapter list --json",
                )
                .with_command("bluetui device list"));
            }
            let mut data: Vec<DeviceDto> = Vec::new();
            for controller in &controllers {
                if let Some(name) = adapter
                    && &controller.name != name
                {
                    continue;
                }
                for d in controller
                    .paired_devices
                    .iter()
                    .chain(controller.new_devices.iter())
                {
                    data.push(DeviceDto::with_adapter(d, &controller.name));
                }
            }
            let response = Response::new("bluetui device list", data);
            output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
        }
        DeviceCommand::Get { address } => {
            let addr = Address::from_str(address).map_err(|_| {
                AppError::new(
                    ErrorCode::InvalidArgument,
                    ExitCode::Usage,
                    format!("invalid device address {address:?}; expected AA:BB:CC:DD:EE:FF"),
                    "bluetui device list --json",
                )
                .with_command("bluetui device get")
            })?;
            let controllers = load_controllers().await?;
            for controller in &controllers {
                for d in controller
                    .paired_devices
                    .iter()
                    .chain(controller.new_devices.iter())
                {
                    if d.addr == addr {
                        let response = Response::new(
                            "bluetui device get",
                            DeviceDto::with_adapter(d, &controller.name),
                        );
                        output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
                        return Ok(ExitCode::Success.as_i32());
                    }
                }
            }
            return Err(AppError::new(
                ErrorCode::NotFound,
                ExitCode::NotFound,
                format!("device {address} not found"),
                "bluetui device list --json",
            )
            .with_command("bluetui device get"));
        }
    }
    Ok(ExitCode::Success.as_i32())
}

async fn load_controllers() -> Result<Vec<crate::bluetooth::Controller>, AppError> {
    let query = bluetooth_query::open().await.into_app(
        ErrorCode::BluetoothUnavailable,
        ExitCode::BluetoothUnavailable,
        BLUEZ_HINT,
    )?;
    query.controllers().await.into_app(
        ErrorCode::BluetoothUnavailable,
        ExitCode::BluetoothUnavailable,
        BLUEZ_HINT,
    )
}

/// Emits a JSON Schema (Draft 2020-12) describing commands, output, and exit codes.
fn schema(args: &SchemaArgs, mode: OutputMode, print0: bool) -> Result<i32, AppError> {
    let mut document = schema_document();
    if let Some(noun) = &args.noun {
        narrow_schema(&mut document, noun, args.verb.as_deref());
    }
    output::emit_value(&document, mode, print0)?;
    Ok(ExitCode::Success.as_i32())
}

/// Emits the machine-readable capability manifest.
fn describe(globals: &GlobalFlags, mode: OutputMode) -> Result<i32, AppError> {
    let response = Response::new("bluetui describe", describe_manifest());
    output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
    Ok(ExitCode::Success.as_i32())
}

fn exit_codes() -> Value {
    json!({
        "0": "success",
        "1": "general failure",
        "2": "usage error",
        "3": "resource not found",
        "4": "permission denied",
        "5": "conflict",
        "10": "bluetooth unavailable",
        "11": "rfkill blocked",
        "12": "invalid configuration",
        "70": "internal error"
    })
}

fn commands_manifest() -> Value {
    json!([
        { "name": "adapter list", "summary": "List all Bluetooth adapters.", "reads": true, "writes": false },
        { "name": "adapter get", "summary": "Show one adapter by name, including its devices.",
          "arguments": [{ "name": "name", "required": true }], "reads": true, "writes": false },
        { "name": "device list", "summary": "List devices, optionally scoped to one adapter.",
          "arguments": [{ "name": "--adapter", "required": false }], "reads": true, "writes": false },
        { "name": "device get", "summary": "Show one device by address.",
          "arguments": [{ "name": "address", "required": true }], "reads": true, "writes": false },
        { "name": "schema", "summary": "Print the JSON Schema for commands and output.", "reads": true, "writes": false },
        { "name": "describe", "summary": "Print this capability manifest.", "reads": true, "writes": false }
    ])
}

fn global_flags() -> Value {
    json!([
        "--json",
        "--format",
        "--fields",
        "--dry-run",
        "--verbose",
        "--quiet",
        "--no-color",
        "--color",
        "--absolute-time",
        "--print0",
        "--yes",
        "--config-path"
    ])
}

fn describe_manifest() -> Value {
    json!({
        "tool": TOOL,
        "version": VERSION,
        "maintainer": MAINTAINER,
        "website": WEBSITE,
        "commands": commands_manifest(),
        "global_flags": global_flags(),
        "output_formats": ["json", "jsonl", "yaml", "csv", "explore"],
        "exit_codes": exit_codes()
    })
}

fn schema_document() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://github.com/UnbreakableMJ/bluetui/schema/v1",
        "title": "bluetui",
        "description": "Dual-mode CLI and TUI for managing Bluetooth on Linux.",
        "type": "object",
        "$defs": {
            "device": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "Device address AA:BB:CC:DD:EE:FF." },
                    "alias": { "type": "string" },
                    "icon": { "type": "string" },
                    "is_paired": { "type": "boolean" },
                    "is_trusted": { "type": "boolean" },
                    "is_connected": { "type": "boolean" },
                    "is_favorite": { "type": "boolean" },
                    "battery_percentage": { "type": ["integer", "null"], "minimum": 0, "maximum": 100 },
                    "adapter": { "type": ["string", "null"] }
                },
                "required": ["address", "alias", "icon", "is_paired", "is_trusted", "is_connected", "is_favorite"]
            },
            "adapter": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Adapter name, e.g. hci0." },
                    "alias": { "type": "string" },
                    "is_powered": { "type": "boolean" },
                    "is_pairable": { "type": "boolean" },
                    "is_discoverable": { "type": "boolean" },
                    "is_scanning": { "type": "boolean" },
                    "paired_device_count": { "type": "integer", "minimum": 0 },
                    "new_device_count": { "type": "integer", "minimum": 0 },
                    "paired_devices": { "type": "array", "items": { "$ref": "#/$defs/device" } },
                    "new_devices": { "type": "array", "items": { "$ref": "#/$defs/device" } }
                },
                "required": ["name", "alias", "is_powered", "is_pairable", "is_discoverable", "is_scanning"]
            }
        },
        "x-commands": commands_manifest(),
        "x-global-flags": global_flags(),
        "x-exit-codes": exit_codes()
    })
}

/// Narrows `x-commands` to the entries matching `noun` (and optional `verb`).
fn narrow_schema(document: &mut Value, noun: &str, verb: Option<&str>) {
    let wanted = match verb {
        Some(verb) => format!("{noun} {verb}"),
        None => noun.to_owned(),
    };
    if let Some(Value::Array(commands)) = document.get_mut("x-commands") {
        commands.retain(|entry| {
            entry
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| {
                    name == wanted || name.starts_with(&format!("{noun} ")) && verb.is_none()
                })
        });
    }
}

// Rust guideline compliant 2026-05-18
