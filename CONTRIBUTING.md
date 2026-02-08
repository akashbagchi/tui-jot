# Contributing to tui-jot

Thanks for your interest in contributing.

## Getting Started

```bash
git clone https://github.com/akashbagchi/tui-jot.git
cd tui-jot
cargo run
```

The app will create a default config at `~/.config/tui-jot/config.toml` and use `~/notes` as the vault directory on first launch.

## Development

```bash
cargo check       # Fast compile check
cargo clippy       # Lint
cargo fmt          # Format code
cargo build        # Debug build
cargo run          # Build and run
```

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy` before committing
5. Write clear commit messages
6. Open a pull request against `main`

## Reporting Issues

When filing a bug report, please include:

- Steps to reproduce
- Expected vs actual behavior
- Terminal emulator and OS
- tui-jot version (`cargo pkgid` or the release tag)

## Code Style

- Run `cargo fmt` before committing
- Address any `cargo clippy` warnings your changes introduce
- Follow existing patterns in the codebase
- Keep changes focused — one feature or fix per PR
