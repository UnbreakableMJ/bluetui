// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

// The BlueZ domain, read-only query helper, and favorites persistence now live
// in `bluetui-core` and are shared with the Beacon GUI. Re-export them so the
// existing `crate::bluetooth` / `crate::favorite` paths keep resolving.
pub use bluetui_core::{bluetooth, bluetooth_query, favorite};

pub mod agent;
mod alias;
pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod dto;
pub mod error;
pub mod event;
pub mod handler;
mod help;
pub mod notification;
pub mod output;
pub mod requests;
pub mod rfkill;
pub mod spinner;
pub mod string_ref;
pub mod text_edit;
pub mod theme;
pub mod time;
pub mod tui;
pub mod ui;
