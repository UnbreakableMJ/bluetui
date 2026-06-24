// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compiles the Slint UI with the Material style (The Steelbore Standard §13).

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("material".to_string());
    slint_build::compile_with_config("ui/app.slint", config).expect("Slint UI failed to compile");
}
