// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Serializable data-transfer objects for machine-readable output.
//!
//! The live [`Controller`](crate::bluetooth::Controller) and
//! [`Device`](crate::bluetooth::Device) types hold non-serializable handles
//! (`Arc<bluer::Adapter>`, `bluer::Device`, `Arc<AtomicBool>`), so the CLI
//! projects them onto these flat DTOs before emitting JSON.

use std::sync::atomic::Ordering;

use serde::Serialize;

use crate::bluetooth::{Controller, Device};

/// A Bluetooth adapter (controller) as rendered by the CLI.
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the BlueZ adapter boolean state flags one-to-one"
)]
#[derive(Debug, Serialize)]
pub struct AdapterDto {
    /// Adapter name, e.g. `hci0`.
    pub name: String,
    /// Human-friendly adapter alias.
    pub alias: String,
    /// Whether the adapter is powered on.
    pub is_powered: bool,
    /// Whether the adapter is pairable.
    pub is_pairable: bool,
    /// Whether the adapter is discoverable.
    pub is_discoverable: bool,
    /// Whether the adapter is currently scanning.
    pub is_scanning: bool,
    /// Number of paired devices.
    pub paired_device_count: usize,
    /// Number of newly discovered (unpaired) devices.
    pub new_device_count: usize,
    /// Paired devices (populated only by `adapter get`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paired_devices: Vec<DeviceDto>,
    /// Newly discovered devices (populated only by `adapter get`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub new_devices: Vec<DeviceDto>,
}

impl AdapterDto {
    /// Builds a summary view (counts only, no device lists) for `adapter list`.
    #[must_use]
    pub fn summary(controller: &Controller) -> Self {
        Self {
            name: controller.name.clone(),
            alias: controller.alias.clone(),
            is_powered: controller.is_powered,
            is_pairable: controller.is_pairable,
            is_discoverable: controller.is_discoverable,
            is_scanning: controller.is_scanning.load(Ordering::Relaxed),
            paired_device_count: controller.paired_devices.len(),
            new_device_count: controller.new_devices.len(),
            paired_devices: Vec::new(),
            new_devices: Vec::new(),
        }
    }

    /// Builds a detailed view (with full device lists) for `adapter get`.
    #[must_use]
    pub fn detailed(controller: &Controller) -> Self {
        let mut dto = Self::summary(controller);
        dto.paired_devices = controller
            .paired_devices
            .iter()
            .map(|d| DeviceDto::with_adapter(d, &controller.name))
            .collect();
        dto.new_devices = controller
            .new_devices
            .iter()
            .map(|d| DeviceDto::with_adapter(d, &controller.name))
            .collect();
        dto
    }
}

/// A Bluetooth device as rendered by the CLI.
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the BlueZ device boolean state flags one-to-one"
)]
#[derive(Debug, Serialize)]
pub struct DeviceDto {
    /// Device address, e.g. `AA:BB:CC:DD:EE:FF`.
    pub address: String,
    /// Human-friendly device alias.
    pub alias: String,
    /// Glyph hint for the device class (Nerd Font icon).
    pub icon: String,
    /// Whether the device is paired.
    pub is_paired: bool,
    /// Whether the device is trusted.
    pub is_trusted: bool,
    /// Whether the device is connected.
    pub is_connected: bool,
    /// Whether the device is marked as a favorite.
    pub is_favorite: bool,
    /// Battery percentage, when reported by the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_percentage: Option<u8>,
    /// The owning adapter name, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
}

impl DeviceDto {
    /// Builds a device DTO tagged with its owning adapter name.
    #[must_use]
    pub fn with_adapter(device: &Device, adapter: &str) -> Self {
        Self {
            adapter: Some(adapter.to_owned()),
            ..Self::from(device)
        }
    }
}

impl From<&Device> for DeviceDto {
    fn from(device: &Device) -> Self {
        Self {
            address: device.addr.to_string(),
            alias: device.alias.clone(),
            icon: device.icon.trim().to_owned(),
            is_paired: device.is_paired,
            is_trusted: device.is_trusted,
            is_connected: device.is_connected,
            is_favorite: device.is_favorite,
            battery_percentage: device.battery_percentage,
            adapter: None,
        }
    }
}

// Rust guideline compliant 2026-05-18
