// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    tempfile_ext::NamedTempFileExt as _,
    super::{args::PackageBuildCommand, BLOBS_JSON_NAME, PACKAGE_MANIFEST_NAME},
    anyhow::{bail, Context as _, Result},
    meshx_pkg::{PackageBuildManifest, PackageBuilder, SubpackagesBuildManifest},
    std::{
        collections::BTreeSet,
        fs::{create_dir_all, File},
        io::{BufReader, BufWriter, Write},
    },
    tempfile::NamedTempFile,
    version_history::AbiRevision,
};

const META_FAR_NAME: &str = "meta.far";
const META_FAR_DEPFILE_NAME: &str = "meta.far.d";
const BLOBS_MANIFEST_NAME: &str = "blobs.manifest";

pub async fn cmd_package_build(cmd: PackageBuildCommand) -> Result<()> {
    let package_build_manifest = File::open(&cmd.package_build_manifest_path)
        .with_context(|| format!("opening {}", cmd.package_build_manifest_path))?;

    let package_build_manifest = PackageBuildManifest::from_pmd_mini(BufReader::new(package_build_manifest))
        .with_context(|| format!("reading {}", cmd.package_build_manifest_path))?;

    println!("{:?}", package_build_manifest);

    let mut builder = PackageBuilder::from_package_build_manifest(&package_build_manifest)
        .with_context(|| format!("creating package manifest from {}", cmd.package_build_manifest_path))?;

    if let Some(abi_revision) = get_abi_revision(&cmd)? {
        builder.abi_revision(abi_revision);
    }

    if let Some(published_name) = &cmd.published_name {
        builder.published_name(published_name);
    }

    if let Some(repository) = &cmd.repository {
        builder.repository(repository);
    }

    let subpackages_build_manifest = if let Some(subpackages_build_manifest_path) = &cmd.subpackages_build_manifest_path
    {
        let f = File::open(subpackages_build_manifest_path)?;
        Some(SubpackagesBuildManifest::deserialize(BufReader::new(f))?)
    } else {
        None
    };

    if let Some(subpackages_build_manifest) = &subpackages_build_manifest {
        for (url, hash, package_manifest_path) in subpackages_build_manifest.to_subpackages()? {
            builder
                .add_subpackage(&url, hash, package_manifest_path.into())
                .with_context(|| format!("adding subpackage {url} : {hash}"))?;
        }
    }

    if !cmd.out.exists() {
        create_dir_all(&cmd.out).with_context(|| format!("creating {}", cmd.out))?;
    }

    let package_manifest_path = cmd.out.join(PACKAGE_MANIFEST_NAME);
    builder.manifest_path(package_manifest_path);

    // Build the package.
    let gendir = tempfile::TempDir::new_in(&cmd.out)?;
    let meta_tar_path = cmd.out.join(META_FAR_NAME);
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

fn get_abi_revision(cmd: &PackageBuildCommand) -> Result<Option<u64>> {
    match (cmd.api_level, cmd.abi_revision) {
        (Some(_), Some(_)) => {
            bail!("--api-level and --abi-revision cannot be specified at the same time")
        }
        (Some(api_level), None) => {
            for version in version_history::version_history().unwrap() {
                if api_level == version.api_level {
                    return Ok(Some(version.abi_revision.into()));
                }
            }

            bail!("Unknown API level {}", api_level)
        }
        (None, Some(abi_revision)) => {
            let abi_revision = AbiRevision::new(abi_revision);
            for version in version_history::version_history().unwrap() {
                if version.abi_revision == abi_revision {
                    return Ok(Some(abi_revision.into()));
                }
            }

            bail!("Unknown ABI revision {}", abi_revision)
        }
        (None, None) => Ok(None),
    }
}
