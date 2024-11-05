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

### Using Nix Flakes (recommended)
Add nixp to your system configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixp.url = "github:frostplexx/nixp";
  };

  outputs = { self, nixpkgs, nixp, ... }: {
    # For NixOS systems
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      # ...
      environment.systemPackages = [ nixp.packages.${system}.default ];
    };

    # For Darwin systems
    darwinConfigurations.yourmac = darwin.lib.darwinSystem {
      # ...
      environment.systemPackages = [ nixp.packages.${system}.default ];
    };
  };
}
```

Or install directly using `nix profile`:
```bash
nix profile install github:yourusername/nixp
```

### Using Cargo
```bash
cargo install nixp
```

## Prerequisites
- Nix package manager with flakes enabled
- Git
- Make
- Homebrew (for macOS only)

### Enabling Flakes
Add the following to your `~/.config/nix/nix.conf` or `/etc/nix/nix.conf`:
```conf
experimental-features = nix-command flakes
```

## Development

### Building from source
```bash
# Clone the repository
git clone https://github.com/yourusername/nixp.git
cd nixp

# Enter development shell
nix develop

# Build the project
nix build

# Run directly
nix run

# Install to your environment
nix profile install .
```

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

## Command Execution
After package operations, nixp will execute the configured commands (install_command, uninstall_command, or update_command) in the directory containing your package files. Command output is displayed in real-time.

## Troubleshooting

### Run the doctor command
Check your system configuration:
```bash
nixp doctor
```

This will verify:
- Configuration file paths
- Required commands
- Git repository status
- Search functionality
- Package list parsing

### Common Issues
1. **Nix Flakes not enabled**:
   Ensure you have enabled flakes in your Nix configuration.

2. **Permission Issues**:
   Make sure your dotfiles directory is writable and git is properly configured.

3. **Command Execution Fails**:
   Verify that Make is installed and your Makefile exists in the dotfiles directory.

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## License
MIT
