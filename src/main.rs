// SPDX-FileCopyrightText: 2024 Badr Badri <contact@pythops.com>
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-only

//! `bluetui` entry point.
//!
//! Dual-mode dispatch (Spacecraft Software CLI Standard): a noun-verb
//! subcommand runs the structured, machine-readable CLI; no subcommand launches
//! the interactive TUI on a terminal, or returns a structured error when no
//! interactive terminal is available (agent / piped output).

use bluetui::{
    app::{App, FocusedBlock},
    cli,
    config::Config,
    error::{AppError, ErrorCode, ExitCode},
    event::{Event, EventHandler},
    handler::handle_key_events,
    output::{self, OutputMode},
    rfkill,
    tui::Tui,
};
use clap::Parser;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, process::ExitCode as ProcessExitCode, sync::Arc};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ProcessExitCode {
    let args = cli::Args::parse();

    // Resolve the output mode before any side effect so early errors (rfkill,
    // config) are emitted in the right form.
    let mode = output::resolve_mode(&args.globals);

    let code = match &args.command {
        Some(command) => match bluetui::commands::dispatch(command, &args.globals, mode).await {
            Ok(code) => code,
            Err(error) => error.emit(mode),
        },
        None => run_default(&args.globals, mode).await,
    };

    ProcessExitCode::from(u8::try_from(code).unwrap_or(1))
}

/// Handles the no-subcommand case: launch the TUI on an interactive terminal,
/// otherwise emit a structured error (an agent or pipe cannot drive the TUI).
async fn run_default(globals: &cli::GlobalFlags, mode: OutputMode) -> i32 {
    if mode.is_human() || mode.is_tui() {
        match run_tui(globals).await {
            Ok(()) => ExitCode::Success.as_i32(),
            Err(error) => error.emit(mode),
        }
    } else {
        AppError::new(
            ErrorCode::MissingArgument,
            ExitCode::Usage,
            "no subcommand was given and no interactive terminal is available",
            "bluetui describe --json",
        )
        .emit(mode)
    }
}

/// Sets up the terminal, builds the [`App`], and runs the event loop. This is
/// the original single-mode entry point, unchanged except that startup errors
/// now flow through [`AppError`] instead of `exit(1)`.
async fn run_tui(globals: &cli::GlobalFlags) -> Result<(), AppError> {
    rfkill::check()?;

    let config = Arc::new(Config::new(globals.config_path.clone())?);

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(anyhow::Error::from)?;
    let events = EventHandler::new(1_000);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    let mut app = App::new(config.clone(), tui.events.sender.clone())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorCode::BluetoothUnavailable,
                ExitCode::BluetoothUnavailable,
                e.to_string(),
                "rfkill list bluetooth",
            )
        })?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                handle_key_events(
                    key_event,
                    &mut app,
                    tui.events.sender.clone(),
                    config.clone(),
                )
                .await?;
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
            Event::NewPairedDevice(address) => {
                if app
                    .requests
                    .display_passkey
                    .as_ref()
                    .is_some_and(|req| req.device == address)
                {
                    app.requests.display_passkey = None;
                }

                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::ToggleFavorite(address) => {
                if let Some(pos) = app
                    .favorite_devices
                    .iter()
                    .position(|favorite| *favorite == address)
                {
                    app.favorite_devices.swap_remove(pos);
                } else {
                    app.favorite_devices.push(address);
                }
            }

            Event::RequestConfirmation(request) => {
                app.requests.init_confirmation(request);
                app.focused_block = FocusedBlock::RequestConfirmation;
            }

            Event::ConfirmationSubmitted => {
                app.requests.confirmation = None;
                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::RequestEnterPinCode(request) => {
                app.requests.init_enter_pin_code(request);
                app.focused_block = FocusedBlock::EnterPinCode;
            }

            Event::PinCodeSumitted => {
                app.requests.enter_pin_code = None;
                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::RequestEnterPasskey(request) => {
                app.requests.init_enter_passkey(request);
                app.focused_block = FocusedBlock::EnterPasskey;
            }

            Event::PasskeySumitted => {
                app.requests.enter_passkey = None;
                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::RequestDisplayPinCode(request) => {
                app.requests.init_display_pin_code(request);
                app.focused_block = FocusedBlock::DisplayPinCode;
            }

            Event::DisplayPinCodeSeen => {
                app.requests.display_pin_code = None;
                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::RequestDisplayPasskey(request) => {
                app.requests.init_display_passkey(request);
                app.focused_block = FocusedBlock::DisplayPasskey;
            }

            Event::DisplayPasskeyCanceled => {
                app.requests.display_passkey = None;
                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::FailedPairing(address) => {
                if app
                    .requests
                    .display_passkey
                    .as_ref()
                    .is_some_and(|req| req.device == address)
                {
                    app.requests.display_passkey = None;
                }

                app.focused_block = FocusedBlock::PairedDevices;
            }

            Event::Mouse(_) | Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
