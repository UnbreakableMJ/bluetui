// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Spacecraft Software §11 color palette as raw `(r, g, b)` triples.
//!
//! This is the single source of truth for the Steelbore palette; front-ends map
//! each token to their own color type (the TUI to `ratatui::style::Color`, the
//! Beacon GUI to a Slint brush). Keeping the values here keeps every front-end
//! in agreement without depending on any UI toolkit.

/// An 8-bit-per-channel RGB triple.
pub type Rgb = (u8, u8, u8);

/// Void Navy — the mandated background canvas (`#000027`).
pub const VOID_NAVY: Rgb = (0, 0, 39);
/// Molten Amber — primary text / active readout (`#D98E32`).
pub const MOLTEN_AMBER: Rgb = (217, 142, 50);
/// Steel Blue — primary accent / structural (`#4B7EB0`).
pub const STEEL_BLUE: Rgb = (75, 126, 176);
/// Radium Green — success / safe status (`#50FA7B`).
pub const RADIUM_GREEN: Rgb = (80, 250, 123);
/// Red Oxide — warning / error status (`#FF5C5C`).
pub const RED_OXIDE: Rgb = (255, 92, 92);
/// Liquid Coolant — info / links (`#8BE9FD`).
pub const LIQUID_COOLANT: Rgb = (139, 233, 253);
/// Derived muted surface for input fields and selected rows (a dark Steel-Blue
/// tint toward Void Navy; §11.1 defines no neutral-surface token).
pub const SURFACE: Rgb = (30, 42, 58);

// Rust guideline compliant 2026-05-18
