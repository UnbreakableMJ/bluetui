// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `Steelbore` theme — the single source of truth for all TUI colors.
//!
//! Per The Steelbore Standard §11/§11.1, palette colors are accessed only
//! through named theme tokens, never as bare color literals in UI logic. This
//! module exposes the six canonical tokens plus a derived `surface` fill and
//! returns ready-to-use [`Style`]s for every recurring rendering role.

use ratatui::style::{Color, Modifier, Style};

use crate::notification::NotificationLevel;

/// Void Navy — the mandated background canvas (`#000027`).
pub const VOID_NAVY: Color = Color::Rgb(0, 0, 39);
/// Molten Amber — primary text / active readout (`#D98E32`).
pub const MOLTEN_AMBER: Color = Color::Rgb(217, 142, 50);
/// Steel Blue — primary accent / structural (`#4B7EB0`).
pub const STEEL_BLUE: Color = Color::Rgb(75, 126, 176);
/// Radium Green — success / safe status (`#50FA7B`).
pub const RADIUM_GREEN: Color = Color::Rgb(80, 250, 123);
/// Red Oxide — warning / error status (`#FF5C5C`).
pub const RED_OXIDE: Color = Color::Rgb(255, 92, 92);
/// Liquid Coolant — info / links (`#8BE9FD`).
pub const LIQUID_COOLANT: Color = Color::Rgb(139, 233, 253);

/// Derived muted surface for input fields, selected rows, and code readouts.
///
/// §11.1 defines no neutral-surface token, and the original `DarkGray` fill is
/// off-palette. This is Steel Blue pulled most of the way toward Void Navy so
/// the fill reads as a darker shade of the accent rather than a foreign gray,
/// while keeping enough contrast against the Navy canvas to remain legible.
pub const SURFACE: Color = Color::Rgb(30, 42, 58);

/// The resolved `Steelbore` theme for one run.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Canvas background. `None` falls back to the terminal's own background.
    pub background: Option<Color>,
    /// Primary text color (Molten Amber).
    pub foreground: Color,
    /// Accent / structural color used for focus and selection (Steel Blue).
    pub accent: Color,
    /// Success / safe status (Radium Green).
    pub success: Color,
    /// Error status (Red Oxide).
    pub error: Color,
    /// Informational / link color (Liquid Coolant).
    pub info: Color,
    /// Muted surface fill for inputs and highlighted rows.
    pub surface: Color,
}

impl Theme {
    /// The canonical `Steelbore` theme with the mandated Void Navy background.
    #[must_use]
    pub const fn steelbore() -> Self {
        Self {
            background: Some(VOID_NAVY),
            foreground: MOLTEN_AMBER,
            accent: STEEL_BLUE,
            success: RADIUM_GREEN,
            error: RED_OXIDE,
            info: LIQUID_COOLANT,
            surface: SURFACE,
        }
    }

    /// `Steelbore` with the terminal's native background instead of Void Navy.
    #[must_use]
    pub const fn steelbore_terminal_bg() -> Self {
        Self {
            background: None,
            ..Self::steelbore()
        }
    }

    /// App-wide base style. Paints the configured background when set; apply it
    /// once over the root area before rendering the rest of the UI.
    #[must_use]
    pub fn base(&self) -> Style {
        let style = Style::default().fg(self.foreground);
        match self.background {
            Some(bg) => style.bg(bg),
            None => style,
        }
    }

    /// Border style for the focused block.
    #[must_use]
    pub fn focused_border(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Title style for the focused block.
    #[must_use]
    pub fn focused_title(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    /// Table header style for the focused block.
    #[must_use]
    pub fn focused_header(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Table header style for an unfocused block.
    #[must_use]
    pub fn unfocused_header(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .add_modifier(Modifier::BOLD)
    }

    /// Highlight style for the selected row in a focused table.
    #[must_use]
    pub fn row_highlight(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.surface)
    }

    /// Style for a text-input field surface (and code/passkey readouts).
    #[must_use]
    pub fn input_surface(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.surface)
    }

    /// Style for an active text-input label.
    #[must_use]
    pub fn input_label_active(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Border style for a modal dialog.
    #[must_use]
    pub fn dialog_border(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Style for the currently selected dialog choice.
    #[must_use]
    pub fn choice_selected(&self) -> Style {
        Style::default()
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for an unselected dialog choice.
    #[must_use]
    pub fn choice_unselected(&self) -> Style {
        Style::default()
    }

    /// Style for an active submit button.
    #[must_use]
    pub fn submit_active(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for the help banner text.
    #[must_use]
    pub fn help(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Style for inline error text.
    #[must_use]
    pub fn error_text(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Accent color for a notification of the given severity.
    ///
    /// §11.1 has no dedicated warning token, so warnings use Molten Amber
    /// (`foreground`), which reads as "caution" and stays on-palette.
    #[must_use]
    pub fn notification(&self, level: &NotificationLevel) -> Color {
        match level {
            NotificationLevel::Info => self.info,
            NotificationLevel::Warning => self.foreground,
            NotificationLevel::Error => self.error,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::steelbore()
    }
}

// Rust guideline compliant 2026-05-18
