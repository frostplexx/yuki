# Default NIX flags for experimental features
NIX_FLAGS = --extra-experimental-features nix-command --extra-experimental-features flakes

# Default target
.PHONY: all
all: build

# Build the project using nix build
.PHONY: build
build:
	nix $(NIX_FLAGS) build

# Run the project using nix run
.PHONY: run
run:
	nix $(NIX_FLAGS) run

# Enter development shell
.PHONY: dev
dev:
	nix $(NIX_FLAGS) develop

# Run checks (clippy, tests, etc.)
.PHONY: check
check:
	nix $(NIX_FLAGS) flake check

# Clean build artifacts
.PHONY: clean
clean:
	nix $(NIX_FLAGS) store gc
	rm -rf result

# Update flake dependencies
.PHONY: update
update:
	nix $(NIX_FLAGS) flake update

# Build release version
.PHONY: release
release:
	nix $(NIX_FLAGS) build -L

# Show flake info
.PHONY: info
info:
	nix $(NIX_FLAGS) flake show

# Install to user profile
.PHONY: install
install:
	nix $(NIX_FLAGS) profile install

# Development helpers
.PHONY: watch
watch:
	cargo watch -x check -x test

# Format code
.PHONY: fmt
fmt:
	cargo fmt

# Run clippy
.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

# Run tests
.PHONY: test
test:
	cargo test

# Help command
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all      - Default target (builds the project)"
	@echo "  build    - Build the project"
	@echo "  run      - Run the project"
	@echo "  dev      - Enter development shell"
	@echo "  check    - Run all checks"
	@echo "  clean    - Clean build artifacts"
	@echo "  update   - Update flake dependencies"
	@echo "  release  - Build release version"
	@echo "  info     - Show flake information"
	@echo "  install  - Install to user profile"
	@echo "  watch    - Watch for changes and run checks"
	@echo "  fmt      - Format code"
	@echo "  clippy   - Run clippy lints"
	@echo "  test     - Run tests"
	@echo "  help     - Show this help message"
