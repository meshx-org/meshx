// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A crate containing common Component Manager types used in Component Manifests
//! (`.cml` files and binary `.cm` files). These types come with `serde` serialization
//! and deserialization implementations that perform the required validation.
use lazy_static::lazy_static;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub const MAX_NAME_LENGTH: usize = 255;
pub const MAX_LONG_NAME_LENGTH: usize = 1024;
pub const MAX_URL_LENGTH: usize = 4096;

lazy_static! {
    /// A default base URL from which to parse relative component URL
    /// components.
    static ref DEFAULT_BASE_URL: url::Url = url::Url::parse("relative:///").unwrap();
}

/// A name that can refer to a component, collection, or other entity in the
/// Component Manifest. Its length is bounded to `MAX_NAME_LENGTH`.
pub type Name = BoundedName<MAX_NAME_LENGTH>;

/// A `Name` with a higher string capacity of `MAX_LONG_NAME_LENGTH`.
pub type LongName = BoundedName<MAX_LONG_NAME_LENGTH>;

/// A `BoundedName` is a `Name` that can have a max length of `N` bytes.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedName<const N: usize>(String);

impl<const N: usize> BoundedName<N> {
    /// Creates a `BoundedName` from a `String`, returning an `Err` if the string
    /// fails validation. The string must be non-empty, no more than `N`
    /// characters in length, and consist of one or more of the
    /// following characters: `A-Z`, `a-z`, `0-9`, `_`, `.`, `-`. It may not start
    /// with `.` or `-`.
    pub fn new(name: impl AsRef<str> + Into<String>) -> Result<Self, ParseError> {
        {
            let name = name.as_ref();
            if name.is_empty() {
                return Err(ParseError::Empty);
            }
            if name.len() > N {
                return Err(ParseError::TooLong);
            }
            let mut char_iter = name.chars();
            let first_char = char_iter.next().unwrap();
            if !first_char.is_ascii_alphanumeric() && first_char != '_' {
                return Err(ParseError::InvalidValue);
            }
            let valid_fn = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.';
            if !char_iter.all(valid_fn) {
                return Err(ParseError::InvalidValue);
            }
        }
        Ok(Self(name.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const N: usize> fmt::Display for BoundedName<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <String as fmt::Display>::fmt(&self.0, f)
    }
}

impl<const N: usize> FromStr for BoundedName<N> {
    type Err = ParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::new(name)
    }
}

/// The error representing a failure to parse a type from string.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// The string did not match a valid value.
    #[error("invalid value")]
    InvalidValue,
    /// The string did not match a valid absolute or relative component URL
    #[error("invalid URL: {details}")]
    InvalidComponentUrl { details: String },
    /// The string was empty.
    #[error("empty")]
    Empty,
    /// The string was too long.
    #[error("too long")]
    TooLong,
    /// A required leading slash was missing.
    #[error("no leading slash")]
    NoLeadingSlash,
    /// The path segment is invalid.
    #[error("invalid path segment")]
    InvalidSegment,
}

// A component URL. The URL is validated, but represented as a string to avoid
/// normalization and retain the original representation.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url(String);

impl Url {
    /// Creates a `Url` from a `String`, returning an `Err` if the string fails
    /// validation. The string must be non-empty, no more than 4096 characters
    /// in length, and be a valid URL. See the [`url`](../../url/index.html) crate.
    pub fn new(url: impl AsRef<str> + Into<String>) -> Result<Self, ParseError> {
        Self::validate(url.as_ref())?;
        Ok(Self(url.into()))
    }

    /// Verifies the given string is a valid absolute or relative component URL.
    pub fn validate(url_str: &str) -> Result<(), ParseError> {
        if url_str.is_empty() {
            return Err(ParseError::Empty);
        }
        if url_str.len() > MAX_URL_LENGTH {
            return Err(ParseError::TooLong);
        }
        match url::Url::parse(url_str).map(|url| (url, false)).or_else(|err| {
            if err == url::ParseError::RelativeUrlWithoutBase {
                DEFAULT_BASE_URL.join(url_str).map(|url| (url, true))
            } else {
                Err(err)
            }
        }) {
            Ok((url, is_relative)) => {
                let mut path = url.path();
                if path.starts_with('/') {
                    path = &path[1..];
                }

                if is_relative && url.fragment().is_none() {
                    // TODO(https://fxbug.dev/42070831): Fragments should be optional
                    // for relative path URLs.
                    //
                    // Historically, a component URL string without a scheme
                    // was considered invalid, unless it was only a fragment.
                    // Subpackages allow a relative path URL, and by current
                    // definition they require a fragment. By declaring a
                    // relative path without a fragment "invalid", we can avoid
                    // breaking tests that expect a path-only string to be
                    // invalid. Sadly this appears to be a behavior of the
                    // public API.
                    return Err(ParseError::InvalidComponentUrl {
                        details: "Relative URL has no resource fragment.".to_string(),
                    });
                }

                if url.host_str().unwrap_or("").is_empty() && path.is_empty() && url.fragment().is_none() {
                    return Err(ParseError::InvalidComponentUrl {
                        details: "URL is missing either `host`, `path`, and/or `resource`.".to_string(),
                    });
                }
            }
            Err(err) => {
                return Err(ParseError::InvalidComponentUrl {
                    details: format!("Malformed URL: {err:?}."),
                });
            }
        }
        // Use the unparsed URL string so that the original format is preserved.
        Ok(())
    }
}

/// Same as [Path] except the path does not begin with `/`.
#[derive(Eq, Ord, PartialOrd, PartialEq, Hash, Clone, Default)]
pub struct RelativePath {
    segments: Vec<Name>,
}

impl RelativePath {
    pub fn dot() -> Self {
        Self { segments: vec![] }
    }

    pub fn is_dot(&self) -> bool {
        self.segments.is_empty()
    }
}

impl fmt::Debug for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dot() {
            write!(f, ".")
        } else {
            write!(
                f,
                "{}",
                self.segments.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("/")
            )
        }
    }
}

/// A path type used throughout Component Framework, along with its variants [NamespacePath] and
/// [RelativePath]. Examples of use:
///
/// - [NamespacePath]: Namespace paths
/// - [Path]: Outgoing paths and namespace paths that can't be "/"
/// - [RelativePath]: Dictionary paths
///
/// [Path] obeys the following constraints:
///
/// - Is a [fuchsia.io.Path](https://fuchsia.dev/reference/fidl/fuchsia.io#Directory.Open).
/// - Begins with `/`.
/// - Is not `.`.
/// - Contains at least one path segment (just `/` is disallowed).
/// - Each path segment is a [Name]. (This is strictly more constrained than a fuchsia.io
///   path segment.)
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Path(RelativePath);

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.0)
    }
}


/// A named capability.
///
/// Unlike a `CapabilityPath`, a `CapabilityName` doesn't encode any form
/// of hierarchy.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityName(pub String);

impl CapabilityName {
    pub fn str(&self) -> &str {
        &self.0
    }
}

impl From<CapabilityName> for String {
    fn from(name: CapabilityName) -> String {
        name.0
    }
}

impl From<&str> for CapabilityName {
    fn from(name: &str) -> CapabilityName {
        CapabilityName(name.to_string())
    }
}

impl From<String> for CapabilityName {
    fn from(name: String) -> CapabilityName {
        CapabilityName(name)
    }
}

impl From<Name> for CapabilityName {
    fn from(name: Name) -> CapabilityName {
        name.as_str().into()
    }
}

impl<'a> PartialEq<&'a str> for CapabilityName {
    fn eq(&self, other: &&'a str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for CapabilityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A named capability type.
///
/// `CapabilityTypeName` provides a user friendly type encoding for a capability.
//#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityTypeName {
    Directory,
    Event,
    EventStreamDeprecated,
    EventStream,
    Protocol,
    Resolver,
    Runner,
    Service,
    Storage,
}

impl fmt::Display for CapabilityTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_name = match &self {
            CapabilityTypeName::Directory => "directory",
            CapabilityTypeName::Event => "event",
            CapabilityTypeName::EventStreamDeprecated => "event_stream_deprecated",
            CapabilityTypeName::EventStream => "event_stream",
            CapabilityTypeName::Protocol => "protocol",
            CapabilityTypeName::Resolver => "resolver",
            CapabilityTypeName::Runner => "runner",
            CapabilityTypeName::Service => "service",
            CapabilityTypeName::Storage => "storage",
        };
        write!(f, "{}", display_name)
    }
}
