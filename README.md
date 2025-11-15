# MeshX

A Rust workspace for MeshX projects.

## Workspace Structure

This repository is organized as a Rust workspace with the following structure:

```
meshx/
├── Cargo.toml          # Workspace configuration
├── crates/             # Individual crates/projects
│   └── (your crates here)
└── README.md
```

## Getting Started

### Creating a New Crate

To add a new crate to the workspace:

```bash
cd crates
cargo new your-crate-name
```

Or for a library:

```bash
cd crates
cargo new --lib your-crate-name
```

The workspace is configured to automatically include all crates in the `crates/` directory.

### Building

Build all crates in the workspace:

```bash
cargo build
```

Build a specific crate:

```bash
cargo build -p your-crate-name
```

### Testing

Run tests for all crates:

```bash
cargo test
```

Run tests for a specific crate:

```bash
cargo test -p your-crate-name
```

### Shared Dependencies

Common dependencies can be defined in the `[workspace.dependencies]` section of the root `Cargo.toml`. Individual crates can then reference these shared dependencies without specifying versions:

```toml
# In crates/your-crate/Cargo.toml
[dependencies]
tokio = { workspace = true }
```

## Attribution

This project is a derived work of the [wasmCloud Shell (wash)](https://github.com/wasmcloud/wash) project, originally created and maintained by the wasmCloud Maintainers. We are grateful for their foundational work that made this project possible.

## License

Apache-2.0

This project maintains the same Apache 2.0 license as the original wash project. See the [LICENSE](LICENSE) file for details.
