// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Beacon — a Material-design GUI for managing Bluetooth on Linux.
//!
//! Built on `bluetui-core` (shared with the bluetui TUI/CLI). Phase 2 skeleton.

// Slint-generated code is exempt from our pedantic lint set.
mod ui {
    #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
    slint::include_modules!();
}

use slint::ComponentHandle;
use ui::AppWindow;

fn main() -> anyhow::Result<()> {
    let window = AppWindow::new()?;
    window.run()?;
    Ok(())
}
