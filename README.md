# nixp

A unified package manager for Nix and Homebrew, designed to help you manage your system packages declaratively across both Linux and macOS.

## Features

- 🔍 Search and install packages from both Nixpkgs and Homebrew
- 🔄 Declarative package management using Nix configuration files
- 🍎 Seamless integration with Homebrew on macOS (both formulae and casks)
- 🛠️ Automated Git integration for tracking changes
- 🔧 Configurable post-install hooks
- 📦 Support for Nix flakes experimental features
- 🩺 Built-in system diagnostics

## Installation

```bash
cargo install nixp
```

## Prerequisites

- Nix package manager
- Git
- Make
- Homebrew (for macOS only)

## Configuration

nixp uses a simple configuration file that can be placed in either:
- `~/.nixprc`
- `~/.config/nixp/config.conf`

Example configuration:
```conf
# Path to linux system packages nix file 
linux_packages_path ~/dotfiles/hosts/nixos/apps.nix
# Path to darwin system packages nix file 
darwin_packages_path ~/dotfiles/hosts/darwin/apps.nix
# Path to homebrew packages file
homebrew_packages_path ~/dotfiles/hosts/darwin/apps.nix

# Git setup
# Automatically add a commit when installing or uninstalling packages
auto_commit true
auto_push false

# Commit messages. Use <package> to insert the package name
uninstall_message "removed <package>"
install_message "installed <package>"

# Commands that will be run after package operations
install_command "make"
uninstall_command "make"
update_command "make update"
```

## Usage

### Search for a package
```bash
nixp search neovim
```

### Install a package
```bash
nixp install neovim
```

### List installed packages
```bash
nixp list
```

### Uninstall a package
```bash
nixp uninstall neovim
```

### Update all packages
```bash
nixp update
```

### Check system configuration
```bash
nixp doctor
```

## File Structure

nixp expects your Nix configuration files to contain certain attributes:

For Nix packages:
```nix
environment.systemPackages = with pkgs; [
  # your packages here
];
```

For Homebrew packages:
```nix
homebrew.brews = [
  # your formulae here
];

homebrew.casks = [
  # your casks here
];
```

## Git Integration

When `auto_commit` is enabled, nixp will:
1. Stage modified package files
2. Create a commit with your configured message
3. Push to remote if `auto_push` is enabled

## Command Execution

After package operations, nixp will execute the configured commands (install_command, uninstall_command, or update_command) in the directory containing your package files.

## Troubleshooting

Run the doctor command to check your system configuration:
```bash
nixp doctor
```

This will verify:
- Configuration file paths
- Required commands
- Git repository status
- Search functionality
- Package list parsing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
