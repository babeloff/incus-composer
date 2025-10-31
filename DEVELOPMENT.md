# Development Guide

## Prerequisites

This project uses [pixi](https://pixi.sh/) for dependency management instead of rustup. Pixi manages both the Rust toolchain and other development dependencies through conda-forge.

### Installing Pixi

#### Linux & macOS
```bash
curl -fsSL https://pixi.sh/install.sh | bash
```

#### Windows
```powershell
iwr -useb https://pixi.sh/install.ps1 | iex
```

Alternatively, you can install pixi via conda:
```bash
conda install -c conda-forge pixi
```

## Quick Start

Once pixi is installed, you can use the project without any additional setup:

```bash
# Build the project
pixi run build

# Run tests
pixi run test

# Run the application
pixi run run

# Check code (without building)
pixi run check

# Format code
pixi run fmt

# Run clippy linter
pixi run clippy
```

Pixi will automatically install Rust and all dependencies the first time you run any command.

## Available Commands

All commands are defined in `pixi.toml` under the `[tasks]` section:

- `pixi run build` - Compile the project
- `pixi run test` - Run all tests
- `pixi run run` - Run the application
- `pixi run check` - Quick syntax check
- `pixi run fmt` - Format code with rustfmt
- `pixi run clippy` - Run the clippy linter with strict warnings

## Direct Cargo Access

You can also use cargo directly through pixi:

```bash
pixi run cargo build --release
pixi run cargo test -- --nocapture
pixi run cargo doc --open
```

## Environment Activation

Pixi creates an isolated environment. You can activate it for direct shell access:

```bash
pixi shell
```

This gives you access to `cargo`, `rustc`, and other tools directly.

## Adding Dependencies

To add Rust dependencies, edit `Cargo.toml`:

```toml
[dependencies]
my-crate = "1.0"
```

To add system-level dependencies through pixi, edit `pixi.toml` or use:

```bash
pixi add <package-name>
```

## Project Structure

```
incus-composer/
├── src/                  # Rust source code
│   ├── main.rs          # Application entry point
│   └── schema.rs        # Schema definitions
├── examples/            # Example configuration files
├── Cargo.toml           # Rust package manifest
├── Cargo.lock           # Locked dependency versions
├── pixi.toml            # Pixi project configuration
├── pixi.lock            # Locked pixi environment (not in git)
└── .pixi/               # Pixi cache directory (not in git)
```

## Why Pixi?

Pixi offers several advantages over rustup:

1. **Reproducible Environments**: Exact versions of all dependencies are locked
2. **Cross-Platform**: Works consistently across Linux, macOS, and Windows
3. **Multi-Language**: Can manage non-Rust dependencies (Python, C libraries, etc.)
4. **Fast**: Leverages conda's efficient binary package distribution
5. **Isolated**: Each project has its own environment

## Troubleshooting

### Pixi command not found

After installation, you may need to restart your shell or source the configuration:

```bash
source ~/.bashrc  # or ~/.zshrc
```

### Build fails with missing dependencies

Try cleaning the environment:

```bash
pixi clean
pixi run build
```

### Version conflicts

Ensure you're using a compatible Rust version. The project specifies the Rust version in `pixi.toml`.

## CI/CD

For continuous integration, install pixi in your CI environment:

```yaml
# GitHub Actions example
- name: Install pixi
  run: curl -fsSL https://pixi.sh/install.sh | bash
  
- name: Build and test
  run: |
    pixi run build
    pixi run test
```

## Contributing

When contributing, please:

1. Run `pixi run fmt` to format your code
2. Run `pixi run clippy` to check for common mistakes
3. Run `pixi run test` to ensure all tests pass
4. Do not commit `pixi.lock` or `.pixi/` directory (they're gitignored)
5. Do commit changes to `pixi.toml` if you add dependencies

## Resources

- [Pixi Documentation](https://pixi.sh/latest/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Incus Documentation](https://linuxcontainers.org/incus/docs/main/)
