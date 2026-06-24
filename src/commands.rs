// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! CLI command handlers (Phases 1–2).
//!
//! Each handler returns the numeric exit code on success or an [`AppError`] on
//! failure. Data commands reuse [`bluetooth_query`] and project onto the DTOs in
//! [`crate::dto`]; write commands reuse the same `bluer` calls as the TUI and
//! honor `--dry-run`; `schema`/`describe` are pure and never touch BlueZ.

use std::str::FromStr;
use std::time::Duration;

use bluer::Address;
use futures::StreamExt;
use serde_json::{Value, json};

use crate::bluetooth::Controller;
use crate::bluetooth_query;
use crate::cli::{AdapterCommand, Command, DeviceCommand, GlobalFlags, OnOff, SchemaArgs};
use crate::dto::{AdapterDto, DeviceDto};
use crate::error::{AppError, ErrorCode, ExitCode, IntoAppError};
use crate::favorite::{read_favorite_devices_from_disk, save_favorite_devices_to_disk};
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
        AdapterCommand::Power { name, state } => {
            let controller = find_adapter(&controllers, name)?;
            if !globals.dry_run {
                controller
                    .adapter
                    .set_powered(state.is_on())
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui adapter list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui adapter power",
                "power",
                name,
                Some(*state),
            )?;
        }
        AdapterCommand::Pairable { name, state } => {
            let controller = find_adapter(&controllers, name)?;
            if !globals.dry_run {
                controller
                    .adapter
                    .set_pairable(state.is_on())
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui adapter list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui adapter pairable",
                "pairable",
                name,
                Some(*state),
            )?;
        }
        AdapterCommand::Discoverable { name, state } => {
            let controller = find_adapter(&controllers, name)?;
            if !globals.dry_run {
                controller
                    .adapter
                    .set_discoverable(state.is_on())
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui adapter list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui adapter discoverable",
                "discoverable",
                name,
                Some(*state),
            )?;
        }
        AdapterCommand::Scan { name, duration } => {
            let controller = find_adapter(&controllers, name)?;
            if globals.dry_run {
                emit_action(globals, mode, "bluetui adapter scan", "scan", name, None)?;
            } else {
                let adapter = controller.adapter.clone();
                let mut stream = adapter.discover_devices().await.into_app(
                    ErrorCode::OperationFailed,
                    ExitCode::General,
                    "bluetui adapter list --json",
                )?;
                // Drive the discovery stream for the requested window; the
                // timeout elapsing is the expected, successful end of the scan.
                let _ = tokio::time::timeout(Duration::from_secs(*duration), async {
                    while stream.next().await.is_some() {}
                })
                .await;
                drop(stream);

                // Re-read to report what discovery surfaced.
                let refreshed = load_controllers().await?;
                let controller = find_adapter(&refreshed, name)?;
                let discovered: Vec<DeviceDto> = controller
                    .new_devices
                    .iter()
                    .map(|d| DeviceDto::with_adapter(d, &controller.name))
                    .collect();
                let discovered = serde_json::to_value(&discovered).map_err(|e| {
                    AppError::new(
                        ErrorCode::InternalError,
                        ExitCode::Internal,
                        format!("failed to serialize discovered devices: {e}"),
                        "bluetui describe --json",
                    )
                })?;
                let response = Response::new(
                    "bluetui adapter scan",
                    json!({
                        "adapter": name.as_str(),
                        "duration_seconds": *duration,
                        "discovered": discovered,
                    }),
                );
                output::render(&response, mode, globals.fields.as_deref(), globals.print0)?;
            }
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
            let addr = parse_addr(address, "bluetui device get")?;
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
        DeviceCommand::Connect { address } => {
            let addr = parse_addr(address, "bluetui device connect")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device connect")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?.connect().await.into_app(
                    ErrorCode::OperationFailed,
                    ExitCode::General,
                    "bluetui device list --json",
                )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device connect",
                "connect",
                address,
                None,
            )?;
        }
        DeviceCommand::Disconnect { address } => {
            let addr = parse_addr(address, "bluetui device disconnect")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device disconnect")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?
                    .disconnect()
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device disconnect",
                "disconnect",
                address,
                None,
            )?;
        }
        DeviceCommand::Trust { address } => {
            let addr = parse_addr(address, "bluetui device trust")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device trust")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?
                    .set_trusted(true)
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device trust",
                "trust",
                address,
                None,
            )?;
        }
        DeviceCommand::Untrust { address } => {
            let addr = parse_addr(address, "bluetui device untrust")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device untrust")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?
                    .set_trusted(false)
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device untrust",
                "untrust",
                address,
                None,
            )?;
        }
        DeviceCommand::Pair { address } => {
            let addr = parse_addr(address, "bluetui device pair")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device pair")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?.pair().await.into_app(
                    ErrorCode::OperationFailed,
                    ExitCode::General,
                    "bluetui device list --json",
                )?;
            }
            emit_action(globals, mode, "bluetui device pair", "pair", address, None)?;
        }
        DeviceCommand::Unpair { address } => {
            let addr = parse_addr(address, "bluetui device unpair")?;
            // Destructive: require explicit confirmation outside of a dry run.
            if !globals.yes && !globals.dry_run {
                return Err(AppError::new(
                    ErrorCode::ConfirmationRequired,
                    ExitCode::Usage,
                    format!("unpairing {address} is destructive"),
                    "bluetui device unpair <addr> --yes",
                )
                .with_command("bluetui device unpair"));
            }
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device unpair")?;
            if !globals.dry_run {
                controller.adapter.remove_device(addr).await.into_app(
                    ErrorCode::OperationFailed,
                    ExitCode::General,
                    "bluetui device list --json",
                )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device unpair",
                "unpair",
                address,
                None,
            )?;
        }
        DeviceCommand::Rename { address, alias } => {
            let addr = parse_addr(address, "bluetui device rename")?;
            let controllers = load_controllers().await?;
            let controller = resolve_device(&controllers, addr, "bluetui device rename")?;
            if !globals.dry_run {
                bluer_device(controller, addr)?
                    .set_alias(alias.clone())
                    .await
                    .into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
            }
            emit_action(
                globals,
                mode,
                "bluetui device rename",
                "rename",
                address,
                None,
            )?;
        }
        DeviceCommand::Favorite { address } => {
            let addr = parse_addr(address, "bluetui device favorite")?;
            let controllers = load_controllers().await?;
            resolve_device(&controllers, addr, "bluetui device favorite")?;
            if !globals.dry_run {
                let mut favorites = read_favorite_devices_from_disk().unwrap_or_default();
                if !favorites.contains(&addr) {
                    favorites.push(addr);
                    save_favorite_devices_to_disk(&favorites).into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
                }
            }
            emit_action(
                globals,
                mode,
                "bluetui device favorite",
                "favorite",
                address,
                None,
            )?;
        }
        DeviceCommand::Unfavorite { address } => {
            let addr = parse_addr(address, "bluetui device unfavorite")?;
            // Unfavoriting is local-only, idempotent state; no BlueZ lookup needed.
            if !globals.dry_run {
                let mut favorites = read_favorite_devices_from_disk().unwrap_or_default();
                if let Some(pos) = favorites.iter().position(|a| *a == addr) {
                    favorites.swap_remove(pos);
                    save_favorite_devices_to_disk(&favorites).into_app(
                        ErrorCode::OperationFailed,
                        ExitCode::General,
                        "bluetui device list --json",
                    )?;
                }
            }
            emit_action(
                globals,
                mode,
                "bluetui device unfavorite",
                "unfavorite",
                address,
                None,
            )?;
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

/// Parses a device address argument, mapping a malformed value to a usage error.
fn parse_addr(address: &str, command: &str) -> Result<Address, AppError> {
    Address::from_str(address).map_err(|_| {
        AppError::new(
            ErrorCode::InvalidArgument,
            ExitCode::Usage,
            format!("invalid device address {address:?}; expected AA:BB:CC:DD:EE:FF"),
            "bluetui device list --json",
        )
        .with_command(command)
    })
}

/// Finds an adapter by name, or returns a not-found error.
fn find_adapter<'a>(controllers: &'a [Controller], name: &str) -> Result<&'a Controller, AppError> {
    controllers.iter().find(|c| c.name == name).ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            ExitCode::NotFound,
            format!("adapter {name:?} not found"),
            "bluetui adapter list --json",
        )
        .with_command("bluetui adapter")
    })
}

/// Finds the controller that owns `addr` (among paired or discovered devices).
fn resolve_device<'a>(
    controllers: &'a [Controller],
    addr: Address,
    command: &str,
) -> Result<&'a Controller, AppError> {
    controllers
        .iter()
        .find(|c| {
            c.paired_devices
                .iter()
                .chain(c.new_devices.iter())
                .any(|d| d.addr == addr)
        })
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::NotFound,
                ExitCode::NotFound,
                format!("device {addr} not found"),
                "bluetui device list --json",
            )
            .with_command(command)
        })
}

/// Obtains a `bluer::Device` handle for `addr` on the given controller's adapter.
fn bluer_device(controller: &Controller, addr: Address) -> Result<bluer::Device, AppError> {
    controller.adapter.device(addr).into_app(
        ErrorCode::OperationFailed,
        ExitCode::General,
        "bluetui device list --json",
    )
}

/// Emits a structured result for a write command, honoring `--dry-run`.
fn emit_action(
    globals: &GlobalFlags,
    mode: OutputMode,
    command: &str,
    action: &str,
    target: &str,
    state: Option<OnOff>,
) -> Result<(), AppError> {
    let mut object = serde_json::Map::new();
    object.insert("action".to_owned(), json!(action));
    object.insert("target".to_owned(), json!(target));
    if let Some(state) = state {
        object.insert(
            "state".to_owned(),
            json!(if state.is_on() { "on" } else { "off" }),
        );
    }
    object.insert("dry_run".to_owned(), json!(globals.dry_run));
    object.insert("applied".to_owned(), json!(!globals.dry_run));
    let response = Response::new(command, Value::Object(object));
    output::render(&response, mode, globals.fields.as_deref(), globals.print0)
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
        { "name": "adapter power", "summary": "Power an adapter on or off.",
          "arguments": [{ "name": "name", "required": true }, { "name": "state", "required": true, "values": ["on", "off"] }], "reads": false, "writes": true },
        { "name": "adapter pairable", "summary": "Enable or disable pairability of an adapter.",
          "arguments": [{ "name": "name", "required": true }, { "name": "state", "required": true, "values": ["on", "off"] }], "reads": false, "writes": true },
        { "name": "adapter discoverable", "summary": "Enable or disable discoverability of an adapter.",
          "arguments": [{ "name": "name", "required": true }, { "name": "state", "required": true, "values": ["on", "off"] }], "reads": false, "writes": true },
        { "name": "adapter scan", "summary": "Scan for nearby devices for a bounded window, then report discoveries.",
          "arguments": [{ "name": "name", "required": true }, { "name": "--duration", "required": false }], "reads": true, "writes": true },
        { "name": "device connect", "summary": "Connect to a paired device.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device disconnect", "summary": "Disconnect a connected device.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device trust", "summary": "Mark a device as trusted.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device untrust", "summary": "Remove trust from a device.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device pair", "summary": "Pair with a device.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device unpair", "summary": "Remove (unpair) a device. Requires --yes in non-TTY mode.",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true, "destructive": true },
        { "name": "device rename", "summary": "Set a device's alias.",
          "arguments": [{ "name": "address", "required": true }, { "name": "alias", "required": true }], "reads": false, "writes": true },
        { "name": "device favorite", "summary": "Mark a device as a favorite (local state).",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
        { "name": "device unfavorite", "summary": "Remove a device's favorite mark (local state).",
          "arguments": [{ "name": "address", "required": true }], "reads": false, "writes": true },
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
