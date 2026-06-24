// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! CUA keybindings for single-line text inputs (The Steelbore Standard §10).
//!
//! Every text-entry context (device rename, PIN entry, passkey entry) routes
//! keys through [`handle_cua`] first, giving `Ctrl+C/X/V/Z/S` their conventional
//! meanings. The clipboard is app-local (a plain [`String`]) — no system
//! clipboard dependency, satisfying the Privacy-Friendly Application policy (§9).
//! Vim-style `h`/`j`/`k`/`l` navigation already exists at the list level.

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use tui_input::{Input, backend::crossterm::EventHandler};

/// The result of routing a key event through the CUA editing layer.
#[derive(Debug, PartialEq, Eq)]
pub enum CuaOutcome {
    /// A clipboard or undo action was performed; no further handling needed.
    Handled,
    /// `Ctrl+S` was pressed; the caller should save/submit the field.
    Save,
    /// Not a CUA control key; the caller should handle the key normally.
    Ignored,
}

/// Routes a key through the standard CUA bindings for a single-line [`Input`],
/// using an app-local `clipboard` and a one-step `undo` snapshot.
///
/// - `Ctrl+C` copies the field contents to the clipboard.
/// - `Ctrl+X` cuts: snapshots for undo, copies, then clears the field.
/// - `Ctrl+V` pastes the clipboard at the cursor (snapshots for undo).
/// - `Ctrl+Z` restores the last cut/paste snapshot.
/// - `Ctrl+S` requests a save (returned as [`CuaOutcome::Save`]).
///
/// Returns [`CuaOutcome::Ignored`] for any non-`Ctrl` key so the caller can fall
/// back to normal text editing.
pub fn handle_cua(
    input: &mut Input,
    clipboard: &mut String,
    undo: &mut Option<String>,
    key_event: KeyEvent,
) -> CuaOutcome {
    if key_event.modifiers != KeyModifiers::CONTROL {
        return CuaOutcome::Ignored;
    }

    match key_event.code {
        KeyCode::Char('c') => {
            input.value().clone_into(clipboard);
            CuaOutcome::Handled
        }
        KeyCode::Char('x') => {
            *undo = Some(input.value().to_owned());
            input.value().clone_into(clipboard);
            input.reset();
            CuaOutcome::Handled
        }
        KeyCode::Char('v') => {
            *undo = Some(input.value().to_owned());
            // Re-feed each character through the input so it lands at the
            // cursor and the cursor advances naturally.
            for ch in clipboard.clone().chars() {
                input.handle_event(&CrosstermEvent::Key(KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )));
            }
            CuaOutcome::Handled
        }
        KeyCode::Char('z') => {
            if let Some(previous) = undo.take() {
                *input = Input::new(previous);
            }
            CuaOutcome::Handled
        }
        KeyCode::Char('s') => CuaOutcome::Save,
        _ => CuaOutcome::Ignored,
    }
}

// Rust guideline compliant 2026-05-18
