// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

use core::fmt;
use std::path::PathBuf;

use ratatui::layout::Flex;
use toml;

use dirs;
use serde::{
    Deserialize, Deserializer,
    de::{self, Unexpected, Visitor},
};

use crate::error::{AppError, ErrorCode, ExitCode};

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
