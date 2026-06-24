;;; SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
;;; SPDX-License-Identifier: GPL-3.0-only
;;;
;;; GNU Guix package definition (The Steelbore Standard §5.5).
;;; Before pushing a release tag, replace the base32 hash in the `sha256`
;;; field with the checksum of the tagged source archive.

(use-modules (guix packages)
             (guix git-download)
             (guix build-system cargo)
             ((guix licenses) #:prefix license:)
             (gnu packages bluetooth)
             (gnu packages glib)
             (gnu packages pkg-config)
             (gnu packages texinfo))

(define-public bluetui
  (package
    (name "bluetui")
    (version "0.8.1")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/UnbreakableMJ/bluetui")
             (commit (string-append "v" version))))
       (file-name (git-file-name name version))
       ;; TODO(release): set to the tagged checkout hash.
       (sha256
        (base32 "0000000000000000000000000000000000000000000000000000"))))
    (build-system cargo-build-system)
    (arguments
     (list #:install-source? #f))
    (native-inputs (list pkg-config texinfo))
    (inputs (list bluez dbus))
    (home-page "https://github.com/UnbreakableMJ/bluetui")
    (synopsis "Dual-mode CLI and TUI for managing bluetooth on Linux")
    (description
     "bluetui is a dual-mode tool for managing Bluetooth on Linux via BlueZ.
With no subcommand it launches an interactive terminal UI; with a noun-verb
subcommand it emits structured, machine-readable output for scripting and AI
agents.")
    (license license:gpl3)))

bluetui
