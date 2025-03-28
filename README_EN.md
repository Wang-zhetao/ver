# Ver - High-Performance Node.js Version Manager

`ver` is a fast, cross-platform Node.js version manager written in Rust that helps you easily manage multiple Node.js versions.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- 🚀 **High Performance** - Written in Rust for blazing-fast speed
- 🔄 **Easy Switching** - Seamlessly switch between Node.js versions
- 🌐 **Cross-Platform** - Works on macOS, Linux, and Windows
- 🏗️ **Multi-Architecture** - Supports x64, arm64, and more
- 🏷️ **Version Aliases** - Create memorable aliases for versions
- 📁 **Project-Specific Versions** - Set different Node.js versions for different projects
- 📦 **One-Click Migration** - Migrate from other version managers (nvm, n)
- 🔍 **Smart Environment Management** - Automatically handles environment variables and path settings

## Installation

### Via Homebrew (macOS)

```bash
brew install ver
```

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/ver.git
cd ver

# Build and install
cargo install --path .
```

### Pre-built Binaries

Download pre-built binaries for your platform from the [Releases](https://github.com/yourusername/ver/releases) page.

## Usage

### Basic Commands

```bash
# View help
ver --help

# List available Node.js versions
ver list
ver list --lts  # Show only LTS versions

# Install a specific version
ver install 18.17.0

# Install the latest version or latest LTS version
ver install latest
ver install lts

# Switch to a version
ver use 18.17.0

# View current version
ver current

# List installed versions
ver installed

# Remove a specific version
ver remove 18.17.0
```

### Version Aliases

```bash
# Create an alias
ver alias myproject 18.17.0

# Switch using an alias
ver use myproject

# List all aliases
ver aliases
```

### Project-Specific Versions

```bash
# Set a specific version for the current project
ver local 16.13.0
```

This creates a `.node-version` file in the current directory.

### Execute Commands

Run commands with a specific version without switching the global version:

```bash
ver exec 14.17.0 npm install
```

### Migration

Migrate installed versions from other version managers:

```bash
ver migrate nvm
ver migrate n
```

### Maintenance

```bash
# Clean cache and temporary files
ver clean

# Update ver itself
ver selfupdate
```

## Supported Platforms

- **Operating Systems**: macOS, Linux, Windows
- **Architectures**: x64, arm64, x86, arm

## Development

### Requirements

- Rust 1.70 or higher

### Building

```bash
cargo build --release
```

## Contributing

Pull requests and issues are welcome. Before submitting a PR, please ensure:

1. Your code passes `cargo fmt` and `cargo clippy` checks
2. You've added necessary tests
3. Documentation is updated accordingly

## License

MIT License 
