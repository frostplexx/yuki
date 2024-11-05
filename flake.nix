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
        
        # Yuki configuration
        yukiConfig = {
          linux_packages_path = "~/dotfiles/hosts/nixos/apps.nix";
          darwin_packages_path = "~/dotfiles/hosts/darwin/apps.nix";
          homebrew_packages_path = "~/dotfiles/hosts/darwin/apps.nix";
          auto_commit = true;
          auto_push = false;
          uninstall_message = "removed <package>";
          install_message = "installed <package>";
          install_command = "make";
          uninstall_command = "make";
          update_command = "make update";
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
          # Pass configuration as environment variables
          YUKI_LINUX_PACKAGES_PATH = yukiConfig.linux_packages_path;
          YUKI_DARWIN_PACKAGES_PATH = yukiConfig.darwin_packages_path;
          YUKI_HOMEBREW_PACKAGES_PATH = yukiConfig.homebrew_packages_path;
          YUKI_AUTO_COMMIT = toString yukiConfig.auto_commit;
          YUKI_AUTO_PUSH = toString yukiConfig.auto_push;
          YUKI_UNINSTALL_MESSAGE = yukiConfig.uninstall_message;
          YUKI_INSTALL_MESSAGE = yukiConfig.install_message;
          YUKI_INSTALL_COMMAND = yukiConfig.install_command;
          YUKI_UNINSTALL_COMMAND = yukiConfig.uninstall_command;
          YUKI_UPDATE_COMMAND = yukiConfig.update_command;
        };
        
        # Build dependencies separately to improve caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        
        # The package itself with configuration
        yuki = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
        
        # Wrapper script to ensure environment variables are set
        yukiWrapper = pkgs.writeShellScriptBin "yuki" ''
          export YUKI_LINUX_PACKAGES_PATH="${yukiConfig.linux_packages_path}"
          export YUKI_DARWIN_PACKAGES_PATH="${yukiConfig.darwin_packages_path}"
          export YUKI_HOMEBREW_PACKAGES_PATH="${yukiConfig.homebrew_packages_path}"
          export YUKI_AUTO_COMMIT="${toString yukiConfig.auto_commit}"
          export YUKI_AUTO_PUSH="${toString yukiConfig.auto_push}"
          export YUKI_UNINSTALL_MESSAGE="${yukiConfig.uninstall_message}"
          export YUKI_INSTALL_MESSAGE="${yukiConfig.install_message}"
          export YUKI_INSTALL_COMMAND="${yukiConfig.install_command}"
          export YUKI_UNINSTALL_COMMAND="${yukiConfig.uninstall_command}"
          export YUKI_UPDATE_COMMAND="${yukiConfig.update_command}"
          exec ${yuki}/bin/yuki "$@"
        '';
      in {
        checks = {
          inherit yuki;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
        };
        packages = {
          default = yukiWrapper;
          unwrapped = yuki;
        };
        apps.default = flake-utils.lib.mkApp {
          drv = yukiWrapper;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ yuki ];
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
          ] ++ commonDeps;
          # Include configuration in development shell
          shellHook = ''
            export YUKI_LINUX_PACKAGES_PATH="${yukiConfig.linux_packages_path}"
            export YUKI_DARWIN_PACKAGES_PATH="${yukiConfig.darwin_packages_path}"
            export YUKI_HOMEBREW_PACKAGES_PATH="${yukiConfig.homebrew_packages_path}"
            export YUKI_AUTO_COMMIT="${toString yukiConfig.auto_commit}"
            export YUKI_AUTO_PUSH="${toString yukiConfig.auto_push}"
            export YUKI_UNINSTALL_MESSAGE="${yukiConfig.uninstall_message}"
            export YUKI_INSTALL_MESSAGE="${yukiConfig.install_message}"
            export YUKI_INSTALL_COMMAND="${yukiConfig.install_command}"
            export YUKI_UNINSTALL_COMMAND="${yukiConfig.uninstall_command}"
            export YUKI_UPDATE_COMMAND="${yukiConfig.update_command}"
          '';
        };
      }
    );
}
