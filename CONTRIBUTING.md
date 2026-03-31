# Contributing to Scoria

[Русский](CONTRIBUTING_RU.md)

Thank you for your interest in contributing to Scoria! This document outlines the process for contributing and the conventions we follow.

## Development Setup

```bash
git clone https://github.com/syn7xx/scoria.git
cd scoria

# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
make deps

# Build the project
make build
```

## Code Style

- **Format**: Run `make fmt` before committing. We use `cargo fmt` with default settings.
- **Linting**: Run `make check` to ensure no clippy warnings. CI will reject PRs with warnings.
- **Testing**: All new functionality should be covered by tests. Run tests with `cargo test`.

## Project Structure

```
src/
├── main.rs          # Entry point, CLI argument parsing
├── lib.rs           # Library root, re-exports
├── app/tray/        # System tray implementation
├── engine/          # Core logic: clipboard, config, hotkeys, vault, autostart, updates
├── i18n/            # Internationalization (English + Russian)
└── ui/              # Platform-specific UI (GTK on Linux, AppKit on macOS)
```

Key modules in `engine/`:
- `clipboard.rs` — reading clipboard (text + images)
- `vault.rs` — writing to Obsidian vault
- `config.rs` — configuration file handling
- `hotkey.rs` — global hotkey registration (X11)
- `autostart.rs` — login item management
- `update.rs` — self-update mechanism

## Submitting Changes

1. **Fork** the repository and create a feature branch from `main`
2. Make your changes following the code style conventions
3. Add tests for new functionality
4. Ensure `make check` and `cargo test` pass
5. Commit with a clear message describing **why** (not just what)
6. Open a Pull Request

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/). Format:

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`.

Example:
```
feat(clipboard): support WebP images

- Add WebP to supported image formats
- Add decoder using the `png` crate (WebP in PNG container)
```

## Reporting Bugs

When reporting a bug, please include:
- Scoria version (`scoria --version`)
- OS and desktop environment
- Steps to reproduce
- Relevant log output (run with `RUST_LOG=debug`)

## Feature Requests

Open an issue with:
- Clear description of the feature
- Use case (why do you need it?)
- Optional: rough implementation idea

## License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.
