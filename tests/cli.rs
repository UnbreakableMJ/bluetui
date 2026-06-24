// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! CLI compliance tests (Spacecraft Software CLI Standard).
//!
//! These exercise the BlueZ-free command surface (`schema`, `describe`, and
//! argument validation) so they run without Bluetooth hardware or a D-Bus
//! session, on any CI runner.

use std::process::{Command, Output};

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_bluetui");

fn run(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(BIN);
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("failed to run bluetui")
}

/// ISO 8601 UTC with the mandatory `Z` suffix and seconds precision.
fn is_iso8601_utc_z(s: &str) -> bool {
    let bytes = s.as_bytes();
    s.len() == 20
        && s.ends_with('Z')
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
}

#[test]
fn schema_is_draft_2020_12_json() {
    let out = run(&["schema"], &[]);
    assert!(out.status.success());
    let doc: Value = serde_json::from_slice(&out.stdout).expect("schema must be valid JSON");
    assert_eq!(
        doc["$schema"], "https://json-schema.org/draft/2020-12/schema",
        "schema must declare the Draft 2020-12 dialect"
    );
    assert!(doc.get("$defs").is_some(), "schema must define $defs");
    assert!(doc.get("x-commands").is_some(), "schema must list commands");
}

#[test]
fn describe_envelope_has_iso8601_utc_timestamp() {
    let out = run(&["describe", "--json"], &[]);
    assert!(out.status.success());
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["metadata"]["tool"], "bluetui");
    let timestamp = doc["metadata"]["timestamp"]
        .as_str()
        .expect("metadata.timestamp must be a string");
    assert!(
        is_iso8601_utc_z(timestamp),
        "timestamp {timestamp:?} must be YYYY-MM-DDTHH:MM:SSZ"
    );
    // Attribution must be present in machine output (Standard §15.2).
    assert!(
        doc["metadata"]["maintainer"]
            .as_str()
            .unwrap()
            .contains("Mohamed Hammad")
    );
}

#[test]
fn stdout_is_utf8_without_bom() {
    let out = run(&["describe", "--json"], &[]);
    assert!(
        !out.stdout.starts_with(&[0xEF, 0xBB, 0xBF]),
        "stdout must not start with a UTF-8 BOM"
    );
    assert!(
        std::str::from_utf8(&out.stdout).is_ok(),
        "stdout must be valid UTF-8"
    );
}

#[test]
fn agent_env_forces_machine_json() {
    let out = run(&["describe"], &[("AI_AGENT", "1")]);
    assert!(out.status.success());
    let doc: Value =
        serde_json::from_slice(&out.stdout).expect("AI_AGENT must force machine JSON output");
    // The invoking agent is recorded as telemetry but must not change the format.
    assert_eq!(doc["metadata"]["tool"], "bluetui");
}

#[test]
fn invalid_address_emits_structured_error_with_exit_2() {
    let out = run(&["device", "get", "NOTANADDR", "--json"], &[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "usage error must exit with code 2"
    );
    let err: Value =
        serde_json::from_slice(&out.stderr).expect("a structured error must be on stderr");
    assert_eq!(err["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(err["error"]["exit_code"], 2);
    let hint = err["error"]["hint"]
        .as_str()
        .expect("error must carry a hint");
    assert!(
        hint.starts_with("bluetui "),
        "hint must be a runnable command, got {hint:?}"
    );
    assert!(is_iso8601_utc_z(
        err["error"]["timestamp"].as_str().unwrap()
    ));
}

#[test]
fn fields_projection_limits_keys() {
    let out = run(&["describe", "--json", "--fields", "tool,version"], &[]);
    assert!(out.status.success());
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    let data = doc["data"].as_object().expect("data must be an object");
    assert!(data.contains_key("tool") && data.contains_key("version"));
    assert!(
        !data.contains_key("commands"),
        "--fields must drop unlisted keys"
    );
    // Metadata is never filtered by --fields.
    assert!(doc["metadata"].get("timestamp").is_some());
}

#[test]
fn version_carries_attribution() {
    let out = run(&["--version"], &[]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("bluetui"));
    assert!(text.contains("Mohamed.Hammad@SpacecraftSoftware.org"));
}

// --- Phase 2: write commands (BlueZ-free guard paths) ---

#[test]
fn unpair_requires_confirmation_in_non_tty() {
    // A valid address parses, then the destructive guard fires before any
    // BlueZ access, so this is hardware-independent.
    let out = run(&["device", "unpair", "AA:BB:CC:DD:EE:FF", "--json"], &[]);
    assert_eq!(out.status.code(), Some(2));
    let err: Value = serde_json::from_slice(&out.stderr).expect("structured error on stderr");
    assert_eq!(err["error"]["code"], "CONFIRMATION_REQUIRED");
    assert!(
        err["error"]["hint"].as_str().unwrap().contains("--yes"),
        "hint must point at --yes"
    );
}

#[test]
fn write_command_rejects_bad_address_before_bluez() {
    let out = run(&["device", "connect", "NOTANADDR", "--json"], &[]);
    assert_eq!(out.status.code(), Some(2));
    let err: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(err["error"]["code"], "INVALID_ARGUMENT");
}

#[test]
fn describe_lists_write_commands() {
    let out = run(&["describe", "--json"], &[]);
    assert!(out.status.success());
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    let commands = doc["data"]["commands"].as_array().expect("commands array");
    let connect = commands
        .iter()
        .find(|c| c["name"] == "device connect")
        .expect("device connect must be listed");
    assert_eq!(connect["writes"], true);
    let unpair = commands
        .iter()
        .find(|c| c["name"] == "device unpair")
        .expect("device unpair must be listed");
    assert_eq!(unpair["destructive"], true);
}
