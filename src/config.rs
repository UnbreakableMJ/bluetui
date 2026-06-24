// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

use core::fmt;
use std::path::PathBuf;

use ratatui::layout::Flex;
use ratatui::style::Color;
use toml;

use dirs;
use serde::{
    Deserialize, Deserializer,
    de::{self, Unexpected, Visitor},
};

use crate::error::{AppError, ErrorCode, ExitCode};
use crate::theme::Theme;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_layout", deserialize_with = "deserialize_layout")]
    pub layout: Flex,

    #[serde(default = "Width::default")]
    pub width: Width,

    #[serde(default = "default_toggle_scanning")]
    pub toggle_scanning: char,

    #[serde(default = "default_esc_quit")]
    pub esc_quit: bool,

    #[serde(default)]
    pub adapter: Adapter,

    #[serde(default)]
    pub paired_device: PairedDevice,

    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Default)]
pub enum Width {
    #[default]
    Auto,
    Size(u16),
}

struct WidthVisitor;

impl Visitor<'_> for WidthVisitor {
    type Value = Width;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("the string \"auto\" or a positive integer (u16)")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "auto" => Ok(Width::Auto),
            _ => value
                .parse::<u16>()
                .map(Width::Size)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self)),
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match u16::try_from(value) {
            Ok(v) => Ok(Width::Size(v)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match u16::try_from(value) {
            Ok(v) => Ok(Width::Size(v)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Signed(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Width {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(WidthVisitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct Adapter {
    #[serde(default = "default_toggle_adapter_pairing")]
    pub toggle_pairing: char,

    #[serde(default = "default_toggle_adapter_power")]
    pub toggle_power: char,

    #[serde(default = "default_toggle_adapter_discovery")]
    pub toggle_discovery: char,
}

impl Default for Adapter {
    fn default() -> Self {
        Self {
            toggle_pairing: 'p',
            toggle_power: 'o',
            toggle_discovery: 'd',
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PairedDevice {
    #[serde(default = "default_unpair_device")]
    pub unpair: char,

    #[serde(default = "default_toggle_device_trust")]
    pub toggle_trust: char,

    #[serde(default = "default_toggle_device_favorite")]
    pub toggle_favorite: char,

    #[serde(default = "default_set_new_name")]
    pub rename: char,
}

impl Default for PairedDevice {
    fn default() -> Self {
        Self {
            unpair: 'u',
            toggle_trust: 't',
            toggle_favorite: 'f',
            rename: 'e',
        }
    }
}

/// The `[theme]` configuration section (Standard §11.1).
///
/// All fields default, so existing configs without a `[theme]` table keep
/// working. `background` chooses the canvas; the optional per-token hex
/// overrides let a user retheme without touching application logic.
#[derive(Deserialize, Debug, Default)]
pub struct ThemeConfig {
    #[serde(default, deserialize_with = "deserialize_background")]
    pub background: BackgroundChoice,

    #[serde(default)]
    pub foreground: Option<HexColor>,

    #[serde(default)]
    pub accent: Option<HexColor>,

    #[serde(default)]
    pub success: Option<HexColor>,

    #[serde(default)]
    pub error: Option<HexColor>,

    #[serde(default)]
    pub info: Option<HexColor>,

    #[serde(default)]
    pub surface: Option<HexColor>,
}

impl ThemeConfig {
    /// Builds the resolved [`Theme`], starting from the canonical Steelbore
    /// theme (with the chosen background) and applying any per-token overrides.
    #[must_use]
    pub fn to_theme(&self) -> Theme {
        let base = match self.background {
            BackgroundChoice::Navy => Theme::steelbore(),
            BackgroundChoice::Terminal => Theme::steelbore_terminal_bg(),
        };
        Theme {
            foreground: self.foreground.map_or(base.foreground, |c| c.0),
            accent: self.accent.map_or(base.accent, |c| c.0),
            success: self.success.map_or(base.success, |c| c.0),
            error: self.error.map_or(base.error, |c| c.0),
            info: self.info.map_or(base.info, |c| c.0),
            surface: self.surface.map_or(base.surface, |c| c.0),
            ..base
        }
    }
}

/// Canvas background choice for the `[theme]` section.
#[derive(Debug, Default, Clone, Copy)]
pub enum BackgroundChoice {
    /// Void Navy (`#000027`), the Standard-mandated default.
    #[default]
    Navy,
    /// Inherit the terminal's own background.
    Terminal,
}

fn deserialize_background<'de, D>(deserializer: D) -> Result<BackgroundChoice, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "navy" => Ok(BackgroundChoice::Navy),
        "terminal" => Ok(BackgroundChoice::Terminal),
        other => Err(de::Error::custom(format!(
            "unknown background {other:?}; valid values: navy, terminal"
        ))),
    }
}

/// A `#RRGGBB` hex color usable as a per-token theme override.
#[derive(Debug, Clone, Copy)]
pub struct HexColor(pub Color);

struct HexColorVisitor;

impl Visitor<'_> for HexColorVisitor {
    type Value = HexColor;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(r##"a hex color string like "#D98E32""##)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_hex_color(value)
            .map(HexColor)
            .ok_or_else(|| de::Error::invalid_value(Unexpected::Str(value), &self))
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexColorVisitor)
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn deserialize_layout<'de, D>(deserializer: D) -> Result<Flex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.as_str() {
        "Legacy" => Ok(Flex::Legacy),
        "Start" => Ok(Flex::Start),
        "End" => Ok(Flex::End),
        "Center" => Ok(Flex::Center),
        "SpaceAround" => Ok(Flex::SpaceAround),
        "SpaceBetween" => Ok(Flex::SpaceBetween),
        other => Err(de::Error::custom(format!(
            "unknown layout variant {other:?}; valid values: Legacy, Start, End, Center, SpaceAround, SpaceBetween"
        ))),
    }
}

fn default_layout() -> Flex {
    Flex::SpaceAround
}

fn default_set_new_name() -> char {
    'e'
}

fn default_toggle_scanning() -> char {
    's'
}

fn default_esc_quit() -> bool {
    false
}

fn default_toggle_adapter_pairing() -> char {
    'p'
}

fn default_toggle_adapter_power() -> char {
    'o'
}

fn default_toggle_adapter_discovery() -> char {
    'd'
}

fn default_unpair_device() -> char {
    'u'
}

fn default_toggle_device_trust() -> char {
    't'
}

fn default_toggle_device_favorite() -> char {
    'f'
}

impl Config {
    /// Loads the configuration from `config_file_path`, or from the default
    /// `$XDG_CONFIG_HOME/bluetui/config.toml` when `None`.
    ///
    /// A missing default config is not an error (built-in defaults apply); a
    /// missing *explicitly requested* file, an unreadable file, or invalid TOML
    /// is reported as a structured [`AppError`] (CLI Standard §1).
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::ConfigInvalid`] when the requested file cannot be
    /// read or the TOML fails to parse.
    pub fn new(config_file_path: Option<PathBuf>) -> Result<Self, AppError> {
        let explicit = config_file_path.is_some();
        let conf_path = if let Some(path) = config_file_path {
            path
        } else {
            let dir = dirs::config_dir().ok_or_else(|| {
                AppError::new(
                    ErrorCode::ConfigInvalid,
                    ExitCode::ConfigInvalid,
                    "could not determine the user configuration directory",
                    "bluetui --config-path <path-to-config.toml>",
                )
            })?;
            dir.join("bluetui").join("config.toml")
        };

        let contents = match std::fs::read_to_string(&conf_path) {
            Ok(contents) => contents,
            // A missing default config simply means "use built-in defaults".
            Err(_) if !explicit => String::new(),
            Err(e) => {
                return Err(AppError::new(
                    ErrorCode::ConfigInvalid,
                    ExitCode::ConfigInvalid,
                    format!("could not read config file {}: {e}", conf_path.display()),
                    "bluetui --config-path <existing-config.toml>",
                ));
            }
        };

        toml::from_str(&contents).map_err(|e| {
            AppError::new(
                ErrorCode::ConfigInvalid,
                ExitCode::ConfigInvalid,
                format!("invalid configuration: {e}"),
                "bluetui describe --json",
            )
        })
    }
}
