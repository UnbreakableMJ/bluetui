# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-only
#
# Nix package definition (The Steelbore Standard §5.5). Builds from the local
# tree via the committed Cargo.lock; for a tagged release, pin `src` to the
# release archive with its SHA-256 instead of the local path.
{
  lib,
  rustPlatform,
  dbus,
  pkg-config,
}:
let
  cargo = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = cargo.package.name;
  version = cargo.package.version;
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  buildInputs = [dbus];
  nativeBuildInputs = [pkg-config];

  meta = {
    description = cargo.package.description;
    homepage = "https://github.com/UnbreakableMJ/bluetui";
    license = lib.licenses.gpl3Only;
    mainProgram = "bluetui";
  };
}
