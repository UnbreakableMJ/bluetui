// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Read-only BlueZ access shared by the front-ends.
//!
//! Reuses [`Controller::get_all`] but deliberately avoids constructing the full
//! application/TUI state (which registers a pairing agent and builds widgets);
//! it opens a bare session and reads adapters.

use std::sync::Arc;

use anyhow::Result;
use bluer::{Address, Session};

use crate::bluetooth::Controller;
use crate::favorite::read_favorite_devices_from_disk;

/// A read-only handle onto the BlueZ session plus the persisted favorites list.
#[derive(Debug)]
pub struct Query {
    session: Arc<Session>,
    favorites: Vec<Address>,
}

/// Opens a BlueZ session and loads the persisted favorites.
///
/// # Errors
///
/// Returns an error if the D-Bus/BlueZ session cannot be established.
pub async fn open() -> Result<Query> {
    let session = Arc::new(Session::new().await?);
    let favorites = read_favorite_devices_from_disk().unwrap_or_default();
    Ok(Query { session, favorites })
}

impl Query {
    /// Reads all adapters and their devices from BlueZ.
    ///
    /// # Errors
    ///
    /// Returns an error if querying any adapter or device over D-Bus fails.
    pub async fn controllers(&self) -> Result<Vec<Controller>> {
        Controller::get_all(self.session.clone(), &self.favorites).await
    }
}

// Rust guideline compliant 2026-05-18
