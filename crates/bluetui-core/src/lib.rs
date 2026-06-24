// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

//! `bluetui-core` — the front-end-agnostic foundation shared by every bluetui
//! front-end (the `bluetui` TUI/CLI and the `beacon` GUI).
//!
//! It owns the BlueZ domain model ([`bluetooth`]), a read-only session helper
//! ([`bluetooth_query`]), favorites persistence ([`favorite`]), and the
//! Spacecraft Software §11 [`palette`] as raw RGB. Each front-end maps the
//! palette to its own color type (ratatui `Color`, Slint `brush`, …).

pub mod bluetooth;
pub mod bluetooth_query;
pub mod favorite;
pub mod palette;

// Rust guideline compliant 2026-05-18
