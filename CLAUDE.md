# CLAUDE.md - MeshX Project Guide for AI Assistants

This document provides a comprehensive guide to the MeshX repository structure, development workflows, and conventions for AI assistants working on this codebase.

## Project Overview

**MeshX** is a Rust-based CLI tool for developing and publishing WebAssembly (Wasm) components. It is derived from the wasmCloud Shell (wash) project and maintained under the Apache-2.0 license.

- **Repository**: https://github.com/meshx-org/meshx
- **Language**: Rust (Edition 2024)
- **Type**: CLI application with library components
- **Main Binary**: `meshx`

## Repository Structure

```
meshx/
├── .github/
│   ├── actions/          # Custom GitHub actions (setup-rust, etc.)
│   └── workflows/        # CI/CD workflows
│       ├── meshx.yaml              # Main CI/CD workflow
│       └── docker-build-push.yaml  # Docker build and publish
├── .claude/
│   └── settings.local.json  # Claude Code permissions configuration
├── crates/
│   └── meshx/            # Main library crate
│       └── src/
│           ├── lib.rs    # Library entry point
│           ├── cli/      # CLI command implementations
│           │   ├── mod.rs
│           │   ├── completion.rs
│           │   └── update.rs
│           └── config.rs # Configuration management
├── src/
│   └── main.rs           # Binary entry point
├── Cargo.toml            # Workspace and package configuration
├── Cargo.lock            # Locked dependency versions
├── Dockerfile            # Multi-stage Docker build
├── rust-toolchain.toml   # Rust toolchain specification
├── rustfmt.toml          # Rustfmt configuration (nightly)
├── .rustfmt.toml         # Rustfmt edition specification
└── README.md             # Project documentation
```

## Tech Stack

### Core Dependencies
- **clap** (v4.5.40) - CLI argument parsing with derive macros
- **tokio** (v1.45.1) - Async runtime with full features
- **tracing** / **tracing-subscriber** - Structured logging
- **anyhow** (v1.0.98) - Error handling
- **serde** / **serde_json** - Serialization/deserialization
- **reqwest** (v0.12.20) - HTTP client with rustls-tls
- **dialoguer** (v0.11.0) - Interactive CLI prompts
- **figment** (v0.10.19) - Hierarchical configuration management
- **etcetera** (v0.10.0) - Cross-platform directory management (XDG compliance)
- **semver** (v1.0.26) - Semantic versioning

### Development Tools
- **Rust Toolchain**: Stable channel
- **Components**: clippy, rustfmt
- **Formatter**: Nightly rustfmt (edition 2024)
- **Linter**: clippy with `-D warnings` (deny all warnings)
- **cargo-machete**: Unused dependency detection

### External Tools Required for Development
- **Node.js**: v22 (for testing)
- **Go**: ^1.25.0 (for testing)
- **TinyGo**: v0.39.0 (for Wasm compilation)
- **wasm-tools**: v1.223.1 (WebAssembly tooling)
- **protoc**: v29.x (Protocol Buffers compiler)

## Code Style and Conventions

### Rust Edition
- **Edition**: 2024 (specified in both Cargo.toml and .rustfmt.toml)
- All code must use Rust 2024 edition features and idioms

### Formatting Rules (rustfmt.toml)
```toml
edition = "2024"
blank_lines_lower_bound = 0
blank_lines_upper_bound = 1
format_code_in_doc_comments = true
group_imports = "StdExternalCrate"
imports_granularity = "Module"
remove_nested_parens = true
reorder_imports = true
reorder_modules = true
unstable_features = true
wrap_comments = true
```

**Important**:
- Use **nightly** rustfmt: `cargo +nightly fmt`
- Imports are grouped: Std → External Crate → Crate
- Module-level import granularity
- Comments are wrapped for readability
- Maximum 1 blank line between code blocks

### Linting Standards
- **Zero tolerance** for clippy warnings: `cargo clippy --workspace -- -D warnings`
- All clippy warnings must be fixed before merging
- Use `#[allow(clippy::...)]` sparingly and only with good justification

### Architecture Patterns

#### CLI Command Pattern
All commands follow a trait-based architecture:

```rust
pub trait CliCommand {
    fn handle(&self, ctx: &CliContext) -> impl Future<Output = anyhow::Result<CommandOutput>>;
    fn enable_pre_hook(&self) -> Option<()>;
    fn enable_post_hook(&self) -> Option<()>;
}
```

**Key Points**:
- Commands use async/await with `tokio`
- All commands receive a `CliContext` with directory strategies
- Commands return `CommandOutput` with both text and JSON support
- Hook system available for pre/post execution logic

#### Output Formatting
- Support both **text** and **JSON** output formats
- Use `CommandOutput::ok()` for successful operations
- Use `CommandOutput::error()` for failures
- Structured output with `data` field for machine-readable information

#### Configuration Management
- **Hierarchical configuration** using Figment:
  1. Default values
  2. Global config (`~/.config/meshx/config.json`)
  3. Local project config (`.wash/config.json`)
  4. Environment variables (`WASH_` prefix)
  5. Command line arguments
- XDG Base Directory Specification compliance (Linux/macOS)
- Windows-specific directory handling via `etcetera`

#### Error Handling
- Use `anyhow::Result` for all fallible operations
- Provide context with `.context()` for better error messages
- Use `bail!` for early returns with errors
- Use `tracing` for structured error logging

### Logging and Tracing
- **Framework**: `tracing` with `tracing-subscriber`
- **Levels**: trace, debug, info, warn, error
- **Environment filter**: Configured via `RUST_LOG` or `--log-level` flag
- **Output formats**: ANSI (colored terminal), JSON
- Use `#[instrument]` attribute for function tracing
- Include relevant context in trace/debug calls

## Development Workflow

### Initial Setup
```bash
# Clone repository
git clone https://github.com/meshx-org/meshx.git
cd meshx

# Build project (downloads dependencies, compiles)
cargo build

# Run tests
cargo test --workspace
```

### Common Commands
```bash
# Format code (MUST use nightly)
cargo +nightly fmt

# Run clippy
cargo clippy --workspace -- -D warnings

# Check for unused dependencies
cargo machete

# Build release binary
cargo build --release

# Run binary
cargo run -- --help
./target/release/meshx --help
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test --workspace

# Run specific test
cargo test test_name

# Run tests for specific package
cargo test -p meshx
```

### Adding New Commands
1. Create a new module in `crates/meshx/src/cli/`
2. Implement the `CliCommand` trait
3. Add to the `MeshXCliCommand` enum in `src/main.rs`
4. Update the match statement in `handle()` and hook methods
5. Add tests for the new command
6. Update documentation

## CI/CD Workflow (.github/workflows/meshx.yaml)

### Triggers
- **Pull Requests**: to `main` branch
- **Pushes**: to `main` branch
- **Tags**: `meshx-v*` (e.g., `meshx-v0.1.0`)

### Jobs

#### 1. Check Job (Multi-platform)
Runs on: Ubuntu, macOS, Windows
- Checkout code
- Setup: Node.js, Go, TinyGo, wasm-tools, protoc, Rust
- Build: `cargo build`
- Test: `cargo test --workspace` with `RUST_BACKTRACE=1`

#### 2. Lint Job (Ubuntu only)
- Checkout code
- Setup: protoc, Rust (nightly for fmt)
- Install: cargo-machete
- Format check: `cargo +nightly fmt -- --check`
- Lint: `cargo clippy --workspace -- -D warnings`
- Unused deps: `cargo machete`

#### 3. Release Job (On git tags only)
- Multi-platform builds: Linux (musl), macOS, Windows
- Targets: x86_64, aarch64
- Linux uses `cargo-zigbuild` for cross-compilation
- Uploads artifacts for each platform
- Creates GitHub releases with binaries
- Generates build provenance attestations

#### 4. Docker Release (On git tags only)
- Builds and pushes Docker image
- Image: `ghcr.io/meshx-org/meshx`
- Tags: extracted from git tag (e.g., `meshx-v1.0.0` → `1.0.0`)

### Important Notes
- **Canary builds** are currently disabled (package permissions not configured)
- All jobs run in parallel except release jobs
- Concurrency: Cancels previous runs on new commits
- Timeout: 30 min (check), 15 min (lint), 60 min (release)

## Git Workflow

### Branch Strategy
- **Main branch**: `main`
- **Feature branches**: `feature/description` or `claude/session-id`
- **Tag format**: `meshx-v{version}` (e.g., `meshx-v0.0.1`)

### Commit Messages

**Format**: Use [Conventional Commits](https://www.conventionalcommits.org/) format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

#### Commit Types
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (formatting, white-space, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvement
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes to build system or dependencies (Cargo.toml, CI/CD)
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

#### Examples
```bash
# Feature addition
git commit -m "feat: add shell completion for bash and zsh"
git commit -m "feat(cli): add new update command with version check"

# Bug fix
git commit -m "fix: resolve panic when config file is missing"
git commit -m "fix(docker): correct entrypoint path in Dockerfile"

# Documentation
git commit -m "docs: update README with installation instructions"
git commit -m "docs: add CLAUDE.md for AI assistant guidance"

# Refactoring
git commit -m "refactor: simplify error handling in CLI context"

# Performance
git commit -m "perf: optimize dependency resolution caching"

# CI/CD changes
git commit -m "ci: add cargo-machete to lint workflow"
git commit -m "build: bump tokio version to 1.45.1"

# Breaking changes (add ! after type)
git commit -m "feat!: change config file format to JSON5"
```

#### Guidelines
- Use lowercase for type and description
- Use imperative mood ("add" not "added" or "adds")
- Keep first line under 72 characters
- Add body for complex changes explaining "why" rather than "what"
- Reference issues/PRs in footer: `Fixes #123` or `Closes #456`
- Use `!` after type or add `BREAKING CHANGE:` footer for breaking changes

### Pre-commit Checks
Before committing:
```bash
cargo +nightly fmt
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## Docker

### Building
```bash
docker build -t meshx .
```

### Dockerfile Overview
- **Base image**: `cgr.dev/chainguard/rust:latest-dev` (builder)
- **Runtime image**: `cgr.dev/chainguard/wolfi-base`
- **Multi-stage build**: Optimizes final image size
- **Dependencies cached**: Separate layer for faster rebuilds
- **Requires**: protoc, protobuf
- **Output**: Static binary at `/usr/local/bin/meshx`

## Release Process

### Version Bumping
1. Update version in `Cargo.toml`
2. Run `cargo build` to update `Cargo.lock`
3. Commit changes: `git commit -m "Bump version to X.Y.Z"`
4. Create tag: `git tag meshx-vX.Y.Z`
5. Push: `git push && git push --tags`

### GitHub Release
- Triggered automatically by pushing `meshx-v*` tags
- Builds binaries for all supported platforms
- Creates GitHub release with:
  - Auto-generated release notes
  - Attached binaries (Linux, macOS, Windows)
  - Build provenance attestations
- Publishes Docker image to GitHub Container Registry

## Configuration Files

### Claude Code Permissions (.claude/settings.local.json)
Pre-approved operations for Claude Code:
- `cargo metadata`, `cargo build`, `cargo test`
- `cargo +nightly fmt`, `cargo clippy`
- `git add`, `git restore`, `git commit`, `git push`
- `WebFetch` for github.com and raw.githubusercontent.com

These operations do not require user approval when working with Claude Code.

## Common Pitfalls and Best Practices

### DO:
- ✅ Always use `cargo +nightly fmt` for formatting
- ✅ Fix all clippy warnings before submitting PRs
- ✅ Add context to errors using `.context()`
- ✅ Use structured logging with `tracing`
- ✅ Write tests for new functionality
- ✅ Use workspace dependencies when possible
- ✅ Follow XDG directory conventions
- ✅ Support both text and JSON output
- ✅ Use async/await consistently with tokio

### DON'T:
- ❌ Don't use stable rustfmt (use nightly)
- ❌ Don't ignore clippy warnings
- ❌ Don't use unwrap() without good reason (prefer ?, context, or expect)
- ❌ Don't add unused dependencies
- ❌ Don't mix blocking and async code inappropriately
- ❌ Don't hardcode paths (use directory strategies)
- ❌ Don't forget to update Cargo.lock
- ❌ Don't push directly to main (use PRs)

## Useful Resources

- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **Tokio Docs**: https://tokio.rs/
- **Clap Documentation**: https://docs.rs/clap/
- **Tracing Guide**: https://docs.rs/tracing/
- **Rust 2024 Edition**: https://doc.rust-lang.org/edition-guide/rust-2024/

## Project Status

**Maintenance**: Actively developed
**License**: Apache-2.0
**Attribution**: Derived from wasmCloud Shell (wash)

## Quick Reference

### File References
- Main entry: `src/main.rs:1`
- Library entry: `crates/meshx/src/lib.rs:1`
- CLI trait: `crates/meshx/src/cli/mod.rs:28`
- Config: `crates/meshx/src/config.rs:1`
- CI workflow: `.github/workflows/meshx.yaml:1`

### Key Constants
- `CARGO_PKG_VERSION`: Current package version
- `CONFIG_FILE_NAME`: "config.json"
- `PROJECT_CONFIG_DIR`: ".wash"

---

**Last Updated**: 2025-11-15
**Document Version**: 1.0