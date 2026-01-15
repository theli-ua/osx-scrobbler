# Contributing to OSX Scrobbler

Thank you for your interest in contributing to OSX Scrobbler! This document provides guidelines for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue on GitHub with:
- A clear, descriptive title
- Steps to reproduce the problem
- Expected behavior vs. actual behavior
- Your macOS version and app version
- Relevant log output (from `~/Library/Logs/osx-scrobbler.log`)

### Suggesting Features

Feature suggestions are welcome! Please create an issue with:
- A clear description of the feature
- Why it would be useful
- How it might work

### Pull Requests

1. **Fork the repository** and create a new branch from `master`
2. **Make your changes** following the coding standards below
3. **Add tests** if applicable (especially for new features)
4. **Run tests** with `cargo test`
5. **Run clippy** with `cargo clippy` and fix any warnings
6. **Update documentation** (README.md, CHANGELOG.md) as needed
7. **Commit your changes** with clear, descriptive commit messages
8. **Push to your fork** and submit a pull request

## Development Setup

### Prerequisites

- macOS 10.15 or later
- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- Git

### Building from Source

```bash
git clone https://github.com/yourusername/osx-scrobbler.git
cd osx-scrobbler
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Locally

```bash
cargo run
```

For debug logging:
```bash
RUST_LOG=debug cargo run --console
```

## Coding Standards

### Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Keep lines under 100 characters when reasonable

### Code Quality

- Avoid `unwrap()` except in tests or with clear justification
- Use `expect()` with descriptive messages when panicking is acceptable
- Prefer `?` operator for error propagation
- Add doc comments for public APIs
- Write tests for new functionality

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- First line should be under 72 characters
- Reference issues and PRs when relevant
- Include "Co-Authored-By" if pair programming

Example:
```
Add support for multiple ListenBrainz instances

- Allow users to configure multiple ListenBrainz endpoints
- Add name field to distinguish between instances
- Update config validation

Fixes #42
```

## Project Structure

```
src/
├── main.rs              # Application entry point and event loop
├── config.rs            # Configuration loading and validation
├── media_monitor.rs     # Media player monitoring and scrobble logic
├── scrobbler.rs         # Last.fm and ListenBrainz integrations
├── text_cleanup.rs      # Text cleanup with regex patterns
└── ui/
    ├── mod.rs
    └── tray.rs          # System tray menu implementation
```

## Testing

- Add unit tests for pure functions (see `text_cleanup.rs` for examples)
- Test error cases, not just happy paths
- Use descriptive test names that explain what is being tested

## Documentation

- Update README.md for user-facing changes
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format
- Add doc comments for public APIs
- Include examples in doc comments when helpful

## Questions?

If you have questions about contributing, feel free to:
- Open an issue for discussion
- Ask in your pull request

Thank you for contributing!
