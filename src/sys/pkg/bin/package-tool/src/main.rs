// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// TODO(https://fxbug.dev/42055130): Consider enabling globally.
#![deny(unused_crate_dependencies)]

mod cmd;

use {
    anyhow::Result,
    argh::FromArgs,
    package_tool::{cmd_package_build, PackageBuildCommand},
};

/// Package manipulation tool
#[derive(FromArgs)]
struct Command {
    #[argh(subcommand)]
    subcommands: SubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    Package(PackageCommand),
    // Repository(RepoCommand),
}

/// Package subcommands
#[derive(FromArgs)]
#[argh(subcommand, name = "package")]
struct PackageCommand {
    #[argh(subcommand)]
    subcommands: PackageSubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum PackageSubCommands {
    // Archive(PackageArchiveCommand),
    Build(PackageBuildCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd: Command = argh::from_env();

    match cmd.subcommands {
        SubCommands::Package(cmd) => match cmd.subcommands {
            PackageSubCommands::Build(cmd) => cmd_package_build(cmd).await,
        },
    }
}
