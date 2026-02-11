{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    rust = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system}.extend rust.overlays.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            ((pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
              extensions = [
                "clippy"
                "rust-analyzer"
                "rust-src"
              ];

              targets = [
                "x86_64-unknown-linux-musl"
              ];
            })
          ];

          packages = with pkgs; [
            cargo-nextest
          ];
        };

        packages = rec {
          default = geo-track-server;

          geo-track-server =
            let
              manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
            in
            pkgs.rustPlatform.buildRustPackage {
              pname = "geo-track-server";
              version = manifest.version;
              cargoLock.lockFile = ./Cargo.lock;
              src = pkgs.lib.cleanSource ./.;
            };
        };
      }
    );
}
