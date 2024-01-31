// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    argh::{ArgsInfo, FromArgs},
    camino::Utf8PathBuf,
};

/// Builds a package.
#[derive(ArgsInfo, FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "build")]
pub struct PackageBuildCommand {
    /// directory to save package artifacts
    #[argh(option, short = 'o', default = "Utf8PathBuf::from(\"./out\")")]
    pub out: Utf8PathBuf,

    /// package API level
    #[argh(option)]
    pub api_level: Option<u64>,

    /// package ABI revision
    #[argh(option)]
    pub abi_revision: Option<u64>,

    /// name of the package
    #[argh(option)]
    pub published_name: Option<String>,

    /// repository of the package
    #[argh(option)]
    pub repository: Option<String>,

    /// produce a depfile file
    #[argh(switch)]
    pub depfile: bool,

    /// produce a blobs.json file
    #[argh(switch)]
    pub blobs_json: bool,

    /// produce a blobs.manifest file
    #[argh(switch)]
    pub blobs_manifest: bool,

    /// path to the subpackages build manifest file
    #[argh(option)]
    pub subpackages_build_manifest_path: Option<Utf8PathBuf>,

    /// path to the package build manifest file
    #[argh(positional)]
    pub package_build_manifest_path: Utf8PathBuf,
}
