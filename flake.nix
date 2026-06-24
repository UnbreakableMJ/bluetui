{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        bluetui = pkgs.callPackage ./package.nix {};
        # Runtime libraries the Beacon GUI (Slint + winit + FemtoVG) dlopens.
        beaconRuntimeLibs = with pkgs; [
          libGL
          libxkbcommon
          wayland
          fontconfig
          freetype
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
        ];
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [pkg-config];
          buildInputs = with pkgs;
            [
              dbus
              # Pinned via rust-toolchain.toml so the Nix dev shell and rustup
              # (rust-analyzer) share one rustc and don't churn the target/ dir.
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            ]
            # Beacon GUI build + runtime system dependencies.
            ++ beaconRuntimeLibs;
          # winit/FemtoVG load GL/X11/Wayland at runtime via dlopen.
          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath beaconRuntimeLibs}:''${LD_LIBRARY_PATH:-}"
          '';
        };
        packages = {
          default = bluetui;
          inherit bluetui;
        };
        legacyPackages = pkgs.extend(final: prev: {
          bluetui = final.callPackage ./package.nix {};
        });
      }
    );
}
