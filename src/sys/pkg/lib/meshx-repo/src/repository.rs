// Copyright 2021 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    crate::{range::Range, resource::Resource},
    anyhow::Result,
    camino::{Utf8Path, Utf8PathBuf},
    futures::{future::BoxFuture, stream::BoxStream},
    meshx_merkle::Hash,
    serde::{Deserialize, Serialize},
    std::{collections::BTreeSet, fmt::Debug, io, sync::Arc, time::SystemTime},
    tuf::{
        pouf::Pouf1, repository::RepositoryProvider as TufRepositoryProvider,
        repository::RepositoryStorage as TufRepositoryStorage,
    },
    url::ParseError,
};

mod file_system;
mod pm;

//#[cfg(test)]
//pub(crate) mod repo_tests;

pub use {
    file_system::{CopyMode, FileSystemRepository, FileSystemRepositoryBuilder},
    pm::PmRepository,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("invalid path '{0}'")]
    InvalidPath(Utf8PathBuf),
    #[error("I/O error")]
    Io(#[source] io::Error),
    #[error("URL Parsing Error")]
    URLParseError(#[source] ParseError),
    #[error(transparent)]
    Tuf(#[from] tuf::Error),
    #[error(transparent)]
    Far(#[from] meshx_archive::Error),
    #[error(transparent)]
    Meta(#[from] meshx_pkg::MetaContentsError),
    #[error(transparent)]
    Http(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ToStr(#[from] hyper::header::ToStrError),
    #[error(transparent)]
    MirrorConfig(#[from] meshx_pkg_midl_ext::MirrorConfigError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("range not satisfiable")]
    RangeNotSatisfiable,
    #[error(transparent)]
    Hash(#[from] meshx_hash::ParseHashError),
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::URLParseError(err)
    }
}

pub trait RepoProvider: TufRepositoryProvider<Pouf1> + Debug + Send + Sync {
    #[cfg(not(target_os = "fuchsia"))]
    /// Get a [RepositorySpec] for this [Repository]
    fn spec(&self) -> RepositorySpec;

    /// Get the repository aliases.
    fn aliases(&self) -> &BTreeSet<String>;

    /// Fetch a metadata [Resource] from this repository.
    fn fetch_metadata_range<'a>(&'a self, path: &str, range: Range) -> BoxFuture<'a, Result<Resource, Error>>;

    /// Fetch a blob [Resource] from this repository.
    fn fetch_blob_range<'a>(&'a self, path: &str, range: Range) -> BoxFuture<'a, Result<Resource, Error>>;

    /// Whether or not the backend supports watching for file changes.
    fn supports_watch(&self) -> bool {
        false
    }

    /// Returns a stream which sends a unit value every time the given path is modified.
    fn watch(&self) -> anyhow::Result<BoxStream<'static, ()>> {
        Err(anyhow::anyhow!("Watching not supported for this repo type"))
    }

    /// Get the length of a blob in this repository.
    fn blob_len<'a>(&'a self, path: &str) -> BoxFuture<'a, anyhow::Result<u64>>;

    /// Get the modification time of a blob in this repository if available.
    fn blob_modification_time<'a>(&'a self, path: &str) -> BoxFuture<'a, anyhow::Result<Option<SystemTime>>>;
}

pub trait RepoStorage: TufRepositoryStorage<Pouf1> + Send + Sync {
    /// Store a blob in this repository.
    fn store_blob<'a>(&'a self, hash: &Hash, len: u64, path: &Utf8Path) -> BoxFuture<'a, Result<()>>;
}

pub trait RepoStorageProvider: RepoStorage + RepoProvider {}
impl<T: RepoStorage + RepoProvider> RepoStorageProvider for T {}

macro_rules! impl_provider {
    (
        <$($desc:tt)+
    ) => {
        impl <$($desc)+ {
            #[cfg(not(target_os = "fuchsia"))]
            fn spec(&self) -> RepositorySpec {
                (**self).spec()
            }

            fn aliases(&self) -> &BTreeSet<String> {
                (**self).aliases()
            }

            fn fetch_metadata_range<'a>(
                &'a self,
                path: &str,
                range: Range,
            ) -> BoxFuture<'a, Result<Resource, Error>> {
                (**self).fetch_metadata_range(path, range)
            }

            fn fetch_blob_range<'a>(
                &'a self,
                path: &str,
                range: Range,
            ) -> BoxFuture<'a, Result<Resource, Error>> {
                (**self).fetch_blob_range(path, range)
            }

            /// Whether or not the backend supports watching for file changes.
            fn supports_watch(&self) -> bool {
                (**self).supports_watch()
            }

            /// Returns a stream which sends a unit value every time the given path is modified.
            fn watch(&self) -> anyhow::Result<BoxStream<'static, ()>> {
                (**self).watch()
            }

            /// Get the length of a blob in this repository.
            fn blob_len<'a>(&'a self, path: &str) -> BoxFuture<'a, anyhow::Result<u64>> {
                (**self).blob_len(path)
            }

            /// Get the modification time of a blob in this repository if available.
            fn blob_modification_time<'a>(
                &'a self,
                path: &str,
            ) -> BoxFuture<'a, anyhow::Result<Option<SystemTime>>> {
                (**self).blob_modification_time(path)
            }
        }
    };
}

impl_provider!(<T: RepoProvider + ?Sized> RepoProvider for &T);
impl_provider!(<T: RepoProvider + ?Sized> RepoProvider for &mut T);
impl_provider!(<T: RepoProvider + ?Sized> RepoProvider for Box<T>);
impl_provider!(<T: RepoProvider + ?Sized> RepoProvider for Arc<T>);

macro_rules! impl_storage {
    (
        <$($desc:tt)+
    ) => {
        impl <$($desc)+ {
            fn store_blob<'a>(&'a self, hash: &Hash, len: u64, path: &Utf8Path) -> BoxFuture<'a, Result<()>> {
                (**self).store_blob(hash, len, path)
            }
        }
    };
}

impl_storage!(<T: RepoStorage + ?Sized> RepoStorage for &T);
impl_storage!(<T: RepoStorage + ?Sized> RepoStorage for &mut T);
impl_storage!(<T: RepoStorage + ?Sized> RepoStorage for Box<T>);
impl_storage!(<T: RepoStorage + ?Sized> RepoStorage for Arc<T>);

/// RepositorySpec describes all the different supported repositories.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RepositorySpec {
    FileSystem {
        metadata_repo_path: Utf8PathBuf,
        blob_repo_path: Utf8PathBuf,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        aliases: BTreeSet<String>,
    },
    Pm {
        path: Utf8PathBuf,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        aliases: BTreeSet<String>,
    },
    Http {
        metadata_repo_url: String,
        blob_repo_url: String,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        aliases: BTreeSet<String>,
    },
    Gcs {
        metadata_repo_url: String,
        blob_repo_url: String,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        aliases: BTreeSet<String>,
    },
}
