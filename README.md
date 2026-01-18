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

- **Project Creation**: Generate new WebAssembly component projects from templates
- **Multi-Language Build System**: Compile components for multiple languages (Rust, Go, TypeScript)
- **Development Loop**: Built-in hot-reload development server (`meshx dev`)
- **OCI Registry Integration**: Push and pull components to/from OCI-compatible registries
- **Environment Health Checks**: Built-in diagnostics and system verification
- **Configuration Management**: Hierarchical configuration with global and project-level settings
- **Self-Updates**: Keep wash up-to-date with the latest features and fixes

## Installation

### From Source

```bash
git clone https://github.com/meshx-org/meshx.git
cd meshx
cargo install --path .
```

## Quickstart

Note: This quickstart requires the [Rust toolchain](https://www.rust-lang.org/tools/install) and the `wasm32-wasip2` target for Rust: `rustup target add wasm32-wasip2`

1. **Check your environment:**

   ```bash
   meshx doctor
   ```

2. **Create a new component:**

   ```bash
   meshx new https://github.com/wasmCloud/wash.git --subfolder examples/http-hello-world
   ```

3. **Build your component:**

   ```bash
   meshx -C ./http-hello-world build
   ```

4. **Start a development loop**

   ```bash
   meshx -C ./http-hello-world dev
   ```

5. **Keep meshx updated:**

   ```bash
   meshx update
   ```

## Commands

| Command            | Description                                                    |
| ------------------ | -------------------------------------------------------------- |
| `meshx auth`       | Authenticate with MeshX ID                                     |
| `meshx config`     | View and manage meshx configuration                             |
| `meshx completion` | Generate shell completions                                     |
| `meshx dev`        | Start a development server for a Wasm component                |
| `meshx deploy`     | Deploy a manifest to MeshX Cloud                               |
| `meshx doctor`     | Check the health of your meshx installation and environment      |
| `meshx new`        | Create a new project from a template or git repository         |
| `meshx update`     | Update MeshX to the latest version                             |

_Run `meshx --help` or `meshx <command> --help` for detailed usage information._

### Shell Completion

#### Zsh

For zsh completion, please run:

```shell
mkdir -p ~/.zsh/completion
meshx completion zsh > ~/.zsh/completion/_meshx
```

and put the following in `~/.zshrc`:

```shell
fpath=(~/.zsh/completion $fpath)
```

Note if you're not running a distribution like oh-my-zsh you may first have to enable autocompletion (and put in `~/.zshrc` to make it persistent):

```shell
autoload -Uz compinit && compinit
```

#### Bash

To enable bash completion, run the following, or put it in `~/.bashrc` or `~/.profile`:

```shell
. <(meshx completion bash)
```

#### Fish

The below commands can be used for fish auto completion:

```shell
mkdir -p ~/.config/fish/completions
meshx completion fish > ~/.config/fish/completions/meshx.fish
```

#### Powershell

The below command can be referred for setting it up. Please note that the path might be different depending on your
system settings.

```shell
meshx completion powershell > $env:UserProfile\\Documents\\WindowsPowerShell\\Scripts\\meshx.ps1
```

## Regenerate Cloud OpenAPI

```bash
openapi-generator-cli generate -g rust -o crates/meshx-client -i openapi.yaml --additional-properties=packageName=meshx_client,packageVersion=<version-here>
```

## Documentation

- [WebAssembly Component Model](https://component-model.bytecodealliance.org/) - Learn about the component model
- [WASI Preview 2](https://github.com/WebAssembly/WASI/tree/main/preview2) - WebAssembly System Interface
- [wasmCloud Documentation](https://wasmcloud.com/docs) - Learn about wasmCloud
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to this project

## Attribution

This project is a derived work of the [wasmCloud Shell (wash)](https://github.com/wasmcloud/wash) project, originally created and maintained by the wasmCloud Maintainers. We are grateful for their foundational work that made this project possible.

## License

Apache-2.0

This project maintains the same Apache 2.0 license as the original wash project. See the [LICENSE](LICENSE) file for details.
