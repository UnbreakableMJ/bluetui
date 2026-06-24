// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! ISO 8601 UTC timestamp helpers.
//!
//! Per The Steelbore Standard §14 and the Spacecraft Software CLI Standard §1,
//! every stored, transmitted, or logged timestamp is rendered in UTC with a
//! mandatory `Z` suffix and seconds precision (`2026-06-24T14:30:00Z`).

use jiff::Timestamp;

/// Returns the current instant as an ISO 8601 UTC timestamp string.
///
/// The output always ends in `Z` and never carries a numeric offset or
/// fractional seconds, matching the CLI Standard's machine-readable contract.
#[must_use]
pub fn now_iso8601() -> String {
    // `Timestamp` is an absolute instant; `strftime` renders its broken-down
    // fields in UTC, so the literal `Z` below is always correct.
    Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// Rust guideline compliant 2026-05-18
