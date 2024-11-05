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
    let
      # Define the module that can be imported by NixOS and nix-darwin
      yukiModule = { config, lib, pkgs, ... }: with lib; {
        options.programs.yuki = {
          enable = mkEnableOption "yuki package manager";
          settings = mkOption {
            type = types.submodule {
              options = {
                darwin_packages_path = mkOption {
                  type = types.str;
                  default = "~/dotfiles/hosts/darwin/apps.nix";
                  description = "Path to Darwin packages configuration";
                };
                linux_packages_path = mkOption {
                  type = types.str;
                  default = "~/dotfiles/hosts/nixos/apps.nix";
                  description = "Path to Linux packages configuration";
                };
                homebrew_packages_path = mkOption {
                  type = types.str;
                  default = "~/dotfiles/hosts/darwin/apps.nix";
                  description = "Path to Homebrew packages configuration";
                };
                auto_commit = mkOption {
                  type = types.bool;
                  default = true;
                  description = "Automatically commit changes";
                };
                auto_push = mkOption {
                  type = types.bool;
                  default = false;
                  description = "Automatically push changes";
                };
                install_message = mkOption {
                  type = types.str;
                  default = "installed <package>";
                  description = "Git commit message for package installation";
                };
                uninstall_message = mkOption {
                  type = types.str;
                  default = "removed <package>";
                  description = "Git commit message for package removal";
                };
                install_command = mkOption {
                  type = types.str;
                  default = "make";
                  description = "Command to run after package installation";
                };
                uninstall_command = mkOption {
                  type = types.str;
                  default = "make";
                  description = "Command to run after package removal";
                };
                update_command = mkOption {
                  type = types.str;
                  default = "make update";
                  description = "Command to run for updating packages";
                };
              };
            };
            default = {};
          };
        };

        config = mkIf config.programs.yuki.enable {
          environment.systemPackages = let
            system = pkgs.system;
          in [
            (self.packages.${system}.default.override {
              yukiConfig = config.programs.yuki.settings;
            })
          ];
        };
      };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Yuki configuration defaults
        yukiConfigDefault = {
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
          pkg-config
          openssl
          git
          nix
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
        
        src = craneLib.cleanCargoSource ./.;
        
        # Function to create build args with specific config
        mkBuildArgs = yukiConfig: {
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
        
        # Build dependencies with default config
        cargoArtifacts = craneLib.buildDepsOnly (mkBuildArgs yukiConfigDefault);
        
        # Function to create the package with specific config
        mkYuki = yukiConfig: craneLib.buildPackage ((mkBuildArgs yukiConfig) // {
          inherit cargoArtifacts;
        });
        
        # Default package with ability to override config
        defaultYuki = mkYuki yukiConfigDefault;
      in {
        checks = {
          inherit defaultYuki;
          clippy = craneLib.cargoClippy ((mkBuildArgs yukiConfigDefault) // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          test = craneLib.cargoTest ((mkBuildArgs yukiConfigDefault) // {
            inherit cargoArtifacts;
          });
        };
        
        packages = {
          default = defaultYuki;
        };
        
        apps.default = flake-utils.lib.mkApp {
          drv = defaultYuki;
        };
        
        devShells.default = pkgs.mkShell {
          inputsFrom = [ defaultYuki ];
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-edit
          ] ++ commonDeps;
        };
      }
    ) // {
      # Export the module for use in NixOS and nix-darwin
      nixosModules.default = yukiModule;
      darwinModules.default = yukiModule;
    };
}
