<br>
<br>
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/meshx_cli_logo_dark.svg">
    <img width="500" alt="MeshX CLI logo" src="docs/meshx_cli_logo_light.svg">
  </picture>
</p>

<br>

<p align="center">
  Build wasm components then deploy and manage them on MeshX Cloud
</p>

<br>

<div align="center">

[![License](https://img.shields.io/github/license/meshx-org/meshx)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/meshx-org/meshx)](https://github.com/meshx-org/meshx)
[![Maintenance Status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/meshx-org/meshx)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Cargo](https://img.shields.io/badge/cargo-latest-blue.svg)](https://crates.io)
</div>

<p align="center">
  <strong>⚠️ Highly WIP - Broken builds on main are common</strong>
</p>

<br>
<hr />
<br>

The `meshx` CLI is a terminal wrapper around MeshX Cloud, designed to simplify interacting with MeshX Cloud's services from your terminal. It streamlines Wasm component development, deployment, and management by providing direct access to MeshX Cloud features in a familiar command-line interface.

## Features

To be updated...

## Installation

### From Source

```bash
git clone https://github.com/meshx-org/meshx.git
cd meshx
cargo install --path .
```

## Commands

| Command            | Description                                                    |
| ------------------ | -------------------------------------------------------------- |
| `meshx auth`       | Authenticate with MeshX ID                                     |
| `meshx completion` | Generate shell completions                                     |
| `meshx dev`        | Start a development server for a Wasm component                |
| `meshx deploy`     | Deploy a manifest to MeshX Cloud                               |
| `meshx new`        | Create a new project from a template or git repository         |
| `meshx update`     | Update MeshX to the latest version                             |

_Run `meshx --help` or `meshx <command> --help` for detailed usage information._

## Regenerate Cloud OpenAPI

```bash
openapi-generator-cli generate -g rust -o crates/meshx-client -i openapi.yaml --additional-properties=packageName=meshx_client,packageVersion=<version-here>
```

## Attribution

This project is a derived work of the [wasmCloud Shell (wash)](https://github.com/wasmcloud/wash) project, originally created and maintained by the wasmCloud Maintainers. We are grateful for their foundational work that made this project possible.

## License

Apache-2.0

This project maintains the same Apache 2.0 license as the original wash project. See the [LICENSE](LICENSE) file for details.
