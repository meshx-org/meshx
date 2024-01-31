// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    super::tempfile_ext::NamedTempFileExt as _,
    super::{args::PackageBuildCommand, BLOBS_JSON_NAME, PACKAGE_MANIFEST_NAME},
    anyhow::{bail, Context as _, Result},
    //fuchsia_pkg::{
    //    PackageBuildManifest, PackageBuilder, SubpackagesBuildManifest,
    //    SubpackagesBuildManifestEntry, SubpackagesBuildManifestEntryKind,
    //},
    meshx_pkg::{PackageBuildManifest, PackageBuilder},
    std::{
        collections::BTreeSet,
        fs::{create_dir_all, File},
        io::{BufReader, BufWriter, Write},
    },
    tempfile::NamedTempFile,
};

const META_TAR_NAME: &str = "meta.tar";
const META_TAR_DEPFILE_NAME: &str = "meta.tar.d";
const BLOBS_MANIFEST_NAME: &str = "blobs.manifest";

pub async fn cmd_package_build(cmd: PackageBuildCommand) -> Result<()> {
    let package_build_manifest = File::open(&cmd.package_build_manifest_path)
        .with_context(|| format!("opening {}", cmd.package_build_manifest_path))?;

    let package_build_manifest = PackageBuildManifest::from_pm_fini(BufReader::new(package_build_manifest))
        .with_context(|| format!("reading {}", cmd.package_build_manifest_path))?;

    let mut builder = PackageBuilder::from_package_build_manifest(&package_build_manifest)
        .with_context(|| format!("creating package manifest from {}", cmd.package_build_manifest_path))?;

    // Build the package.
    let gendir = tempfile::TempDir::new_in(&cmd.out)?;
    let meta_tar_path = cmd.out.join(META_TAR_NAME);
    let package_manifest = builder
        .build(gendir.path(), &meta_tar_path)
        .with_context(|| format!("creating package manifest {meta_tar_path}"))?;

    // FIXME(https://fxbug.dev/42052115): Some tools still depend on the legacy `blobs.manifest` file. We
    // should migrate them over to using `package_manifest.json` so we can stop producing this file.
    if cmd.blobs_manifest {
        let blobs_manifest_path = cmd.out.join(BLOBS_MANIFEST_NAME);

        let mut tmp = NamedTempFile::new_in(&cmd.out).with_context(|| format!("creating {blobs_manifest_path}"))?;

        {
            let mut file = BufWriter::new(&mut tmp);

            for entry in package_manifest.blobs() {
                writeln!(file, "{}={}", entry.merkle, entry.source_path)?;
            }
        }

        tmp.persist_if_changed(&blobs_manifest_path)
            .with_context(|| format!("creating {blobs_manifest_path}"))?;
    }

    Ok(())
}
