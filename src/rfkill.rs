// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

//! Pre-flight rfkill check.
//!
//! Before the TUI starts, verify the Bluetooth radio is not soft- or
//! hard-blocked. A block returns a structured [`AppError`] (Spacecraft Software
//! CLI Standard §1) with a runnable hint, rather than terminating the process
//! directly, so the caller can render it per the active output mode.

use std::fs;

use crate::error::{AppError, ErrorCode, ExitCode};

/// Scans `/sys/class/rfkill/` and returns an error if the Bluetooth device is
/// soft- or hard-blocked.
///
/// The scan degrades gracefully: a missing rfkill directory or an unreadable
/// entry is skipped rather than treated as a failure.
///
/// # Errors
///
/// Returns [`ErrorCode::RfkillBlocked`] when a Bluetooth rfkill entry reports a
/// soft (`state == 0`) or hard (`state == 2`) block.
pub fn check() -> Result<(), AppError> {
    let Ok(entries) = fs::read_dir("/sys/class/rfkill/") else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();

        let Ok(kind) = fs::read_to_string(entry_path.join("type")) else {
            continue;
        };
        if kind.trim() != "bluetooth" {
            continue;
        }

        let Ok(raw_state) = fs::read_to_string(entry_path.join("state")) else {
            continue;
        };
        let Ok(state) = raw_state.trim().parse::<u8>() else {
            continue;
        };

        // https://www.kernel.org/doc/Documentation/ABI/stable/sysfs-class-rfkill
        match state {
            0 => {
                return Err(AppError::new(
                    ErrorCode::RfkillBlocked,
                    ExitCode::RfkillBlocked,
                    "the Bluetooth device is soft blocked",
                    "sudo rfkill unblock bluetooth",
                ));
            }
            2 => {
                return Err(AppError::new(
                    ErrorCode::RfkillBlocked,
                    ExitCode::RfkillBlocked,
                    "the Bluetooth device is hard blocked by a hardware switch",
                    "rfkill list bluetooth",
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

// Rust guideline compliant 2026-05-18
