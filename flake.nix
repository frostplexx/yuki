{
  description = "yuki - A meta package manager for Nix and Homebrew";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Setup crane with stable rust
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common dependencies for building and development
        commonDeps = with pkgs; [
          # Build dependencies
          pkg-config
          openssl
          
          # Runtime dependencies
          git
          nix
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Project source with only the necessary files
        src = craneLib.cleanCargoSource ./.;

        # Common arguments for building
        commonArgs = {
          inherit src;
          buildInputs = commonDeps;
        };

        # Build dependencies separately to improve caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # The package itself
        yuki = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

      in {
        checks = {
          # Build the package
          inherit yuki;

          # Run clippy
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Run tests
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        packages.default = yuki;

        apps.default = flake-utils.lib.mkApp {
          drv = yuki;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ yuki ];
          buildInputs = with pkgs; [
            # Development tools
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
          ] ++ commonDeps;
        };
      }
    );
}
