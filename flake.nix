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
                "rustfmt"
              ];

              targets = [
                "x86_64-unknown-linux-musl"
              ];
            })
          ];

          packages = with pkgs; [
            cargo-edit
            cargo-nextest
            cargo-outdated
            cargo-update
          ];
        };
      }
    );
}
