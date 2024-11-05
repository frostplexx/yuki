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
          environment.systemPackages = [
            (self.packages.${pkgs.system}.default)
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
        
        # Setup crane with stable rust
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        defaultYuki = craneLib.buildPackage {
          pname = "yuki";
          inherit src;
          
          buildInputs = commonDeps;
          
          # Environment variables with default values
          YUKI_LINUX_PACKAGES_PATH = "~/dotfiles/hosts/nixos/apps.nix";
          YUKI_DARWIN_PACKAGES_PATH = "~/dotfiles/hosts/darwin/apps.nix";
          YUKI_HOMEBREW_PACKAGES_PATH = "~/dotfiles/hosts/darwin/apps.nix";
          YUKI_AUTO_COMMIT = "true";
          YUKI_AUTO_PUSH = "false";
          YUKI_UNINSTALL_MESSAGE = "removed <package>";
          YUKI_INSTALL_MESSAGE = "installed <package>";
          YUKI_INSTALL_COMMAND = "make";
          YUKI_UNINSTALL_COMMAND = "make";
          YUKI_UPDATE_COMMAND = "make update";
          
          cargoArtifacts = craneLib.buildDepsOnly {
            inherit src;
            buildInputs = commonDeps;
          };
        };
      in {
        packages.default = defaultYuki;

        checks = {
          inherit defaultYuki;
          clippy = craneLib.cargoClippy {
            inherit src;
            cargoArtifacts = craneLib.buildDepsOnly {
              inherit src;
              buildInputs = commonDeps;
            };
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };
          test = craneLib.cargoTest {
            inherit src;
            cargoArtifacts = craneLib.buildDepsOnly {
              inherit src;
              buildInputs = commonDeps;
            };
          };
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
      nixosModules.default = yukiModule;
      darwinModules.default = yukiModule;
    };
}
