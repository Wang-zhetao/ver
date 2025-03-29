# Ver-dev - High-Performance Version Manager

`ver-dev` is a fast, cross-platform version manager written in Rust that helps you easily manage multiple Node.js, Rust, Python, and Go versions.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- 🚀 **High Performance**: Written in Rust for speed and low resource usage
- 🔄 **Multi-language Support**: Manage Node.js, Rust, Python, and Go versions
- 🌈 **Colored Output**: Intuitive colored terminal output to distinguish different languages
- 🔄 **Version Switching**: Quickly switch between different versions
- 📦 **Simple Installation**: No extra dependencies, one-command installation
- 🔌 **Cross-Platform**: Support for macOS, Linux, and Windows
- 🏷️ **Version Aliases**: Create aliases for commonly used versions
- 📂 **Project-Specific Versions**: Set different versions for different projects
- 🔄 **Migration Tools**: Migrate from other version managers (nvm, rustup, pyenv, gvm)
- 🚀 **High Performance** - Written in Rust for blazing-fast speed
- 🔄 **Easy Switching** - Seamlessly switch between versions
- 🌐 **Cross-Platform** - Works on macOS, Linux, and Windows
- 🏗️ **Multi-Architecture** - Supports x64, arm64, and more
- 🏷️ **Version Aliases** - Create memorable aliases for versions
- 📁 **Project-Specific Versions** - Set different versions for different projects
- 📦 **One-Click Migration** - Migrate from other version managers (nvm, rustup)
- 🔍 **Smart Environment Management** - Automatically handles environment variables and path settings
- 🦀 **Multi-Language Support** - Manages both Node.js and Rust versions
- 🎨 **Colored Terminal Output** - Visually distinguish Node.js (green) and Rust (yellow) versions

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

### Development Version Installation

If you want to use the latest development version, you can create a symbolic link:

```bash
# Build the development version
cargo b

# Create a symbolic link to the ~/.cargo/bin directory
ln -sf "$(pwd)/target/debug/ver" ~/.cargo/bin/ver-dev

# Now you can use the ver-dev command
ver-dev -h
```

### Pre-built Binaries

Download pre-built binaries for your platform from the [Releases](https://github.com/yourusername/ver/releases) page.

## Usage

### Node.js Version Management

```bash
# View help
ver-dev --help

# List available Node.js versions
ver-dev list
ver-dev list --lts  # Show only LTS versions

# Install a specific version
ver-dev install 18.17.0

# Install the latest version or latest LTS version
ver-dev install latest
ver-dev install lts

# Switch to a version
ver-dev use 18.17.0

# View current version
ver-dev current

# List installed versions
ver-dev installed

# Remove a specific version
ver-dev remove 18.17.0
```

### Rust Version Management

```bash
# List available Rust versions
ver-dev rust list
ver-dev rust list --stable  # Show only stable versions

# Install a specific Rust version
ver-dev rust install 1.85.0

# Install the latest version
ver-dev rust install latest
ver-dev rust install stable  # Install latest stable version

# Switch to a Rust version
ver-dev rust use 1.85.0

# View current Rust version
ver-dev rust current

# List installed Rust versions
ver-dev rust installed

# Remove a specific Rust version
ver-dev rust remove 1.85.0
```

### Python Version Management

```bash
# List available Python versions
ver-dev python list
ver-dev python list --stable  # Show only stable versions

# Install a specific Python version
ver-dev python install 3.12.0

# Install the latest version
ver-dev python install latest
ver-dev python install stable  # Install latest stable version

# Switch to a Python version
ver-dev python use 3.12.0

# View current Python version
ver-dev python current

# List installed Python versions
ver-dev python installed

# Remove a specific Python version
ver-dev python remove 3.12.0

# Create a Python alias
ver-dev python alias myproject 3.12.0

# Switch using an alias
ver-dev python use myproject

# List all Python aliases
ver-dev python aliases

# Set a specific Python version for the current project
ver-dev python local 3.12.0

# Migrate from pyenv
ver-dev python migrate pyenv
```

### Go Version Management

```bash
# List available Go versions
ver-dev go list
ver-dev go list --stable  # Show only stable versions

# Install a specific Go version
ver-dev go install 1.22.0

# Install the latest version
ver-dev go install latest
ver-dev go install stable  # Install latest stable version

# Switch to a Go version
ver-dev go use 1.22.0

# View current Go version
ver-dev go current

# List installed Go versions
ver-dev go installed

# Remove a specific Go version
ver-dev go remove 1.22.0

# Create a Go alias
ver-dev go alias myproject 1.22.0

# Switch using an alias
ver-dev go use myproject

# List all Go aliases
ver-dev go aliases

# Set a specific Go version for the current project
ver-dev go local 1.22.0

# Migrate from gvm
ver-dev go migrate gvm
```

### Version Aliases

```bash
# Create a Node.js alias
ver-dev alias myproject 18.17.0

# Create a Rust alias
ver-dev rust alias myproject 1.85.0

# Switch using an alias
ver-dev use myproject
ver-dev rust use myproject

# List all aliases
ver-dev aliases
ver-dev rust aliases
```

### Project-Specific Versions

```bash
# Set a specific Node.js version for the current project
ver-dev local 16.13.0

# Set a specific Rust version for the current project
ver-dev rust local 1.85.0
```

This creates a `.node-version` or `.rust-version` file in the current directory.

### Execute Commands

Run commands with a specific version without switching the global version:

```bash
# Run commands with a specific Node.js version
ver-dev exec 14.17.0 npm install

# Run commands with a specific Rust version
ver-dev rust exec 1.85.0 cargo b
```

### Migration

Migrate installed versions from other version managers:

```bash
# Migrate Node.js versions from nvm
ver-dev migrate nvm

# Migrate Rust versions from rustup
ver-dev rust migrate rustup
```

### Maintenance

```bash
# Clean cache and temporary files
ver-dev clean

# Update ver itself
ver-dev selfupdate
```

## Colored Terminal Output

To improve readability and user experience, ver-dev uses colored terminal output to distinguish between different languages and version information:

- **Node.js**: Green
- **Rust**: Yellow
- **Python**: Blue
- **Go**: Red

Current versions and important information are displayed in bold, making it easier to identify key information.

## Supported Platforms

- **Operating Systems**: macOS, Linux, Windows
- **Architectures**: x64, arm64, x86, arm

## Development

### Requirements

- Rust 1.70 or higher

### Building

```bash
# Build with Cargo (development version)
cargo b

# Build release version
cargo b --release

# Run tests
cargo t

# Check code formatting
cargo fmt

# Static code analysis
cargo c
```

## Contributing

Pull requests and issues are welcome. Before submitting a PR, please ensure:

1. Your code passes `cargo fmt` and `cargo clippy` checks
2. You've added necessary tests
3. Documentation is updated accordingly

## License

MIT License 
