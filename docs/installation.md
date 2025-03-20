# Installation Guide

This guide will help you install Debshrew and its dependencies.

## Prerequisites

Before installing Debshrew, ensure you have the following prerequisites:

- **Rust**: Version 1.70 or later
- **Cargo**: The Rust package manager (included with Rust)
- **Git**: For cloning the repository
- **A running metashrew instance**: Debshrew requires a metashrew instance to connect to

### Installing Rust

If you don't have Rust installed, you can install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation.

### Installing Git

#### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install git
```

#### macOS

```bash
brew install git
```

#### Windows

Download and install Git from [git-scm.com](https://git-scm.com/download/win).

## Installing Debshrew

### From Source

1. Clone the repository:

```bash
git clone https://github.com/example/debshrew.git
cd debshrew
```

2. Build the project:

```bash
cargo build --release
```

3. The compiled binary will be available at `target/release/debshrew`.

### Using Cargo

You can also install Debshrew directly using Cargo:

```bash
cargo install --git https://github.com/example/debshrew.git
```

## Verifying the Installation

To verify that Debshrew was installed correctly, run:

```bash
debshrew --version
```

You should see the version number of Debshrew printed to the console.

## Setting Up a Development Environment

If you're planning to develop with Debshrew, you might want to set up a development environment:

1. Clone the repository:

```bash
git clone https://github.com/example/debshrew.git
cd debshrew
```

2. Install development dependencies:

```bash
# Install wasm-pack for building WASM modules
cargo install wasm-pack

# Install cargo-watch for automatic rebuilding
cargo install cargo-watch
```

3. Set up a local metashrew instance:

See the [Metashrew documentation](https://github.com/metashrew/metashrew) for instructions on setting up a local metashrew instance.

## Next Steps

Now that you have Debshrew installed, you can:

- [Configure Debshrew](configuration.md)
- [Create a transform module](wasm-transform-guide.md)
- [Run Debshrew](quickstart.md)