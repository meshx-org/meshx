use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_DYNAMIC_NAME_LENGTH: usize = 1024;

/// A name that can refer to a component, collection, or other entity in the
/// Component Manifest.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

/// The error representing a failure to parse a type from string.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// The string did not match a valid value.
    #[error("invalid value")]
    InvalidValue,
    /// The string was too long or too short.
    #[error("invalid length")]
    InvalidLength,
    /// A name was expected and the string was a path.
    #[error("not a name")]
    NotAName,
    // A path was expected and the string was a name.
    #[error("not a path")]
    NotAPath,
}

impl Name {
    /// Creates a `Name` from a `String`, returning an `Err` if the string
    /// fails validation. The string must be non-empty, no more than 100
    /// characters in length, and consist of one or more of the
    /// following characters: `a-z`, `0-9`, `_`, `.`, `-`.
    pub fn new(name: String) -> Result<Self, ParseError> {
        Self::validate(Cow::Owned(name.clone()), MAX_NAME_LENGTH)?;
        Ok(Self(name))
    }

    pub fn validate(name: Cow<'_, str>, max_name_len: usize) -> Result<(), ParseError> {
        if name.is_empty() || name.len() > max_name_len {
            return Err(ParseError::InvalidLength);
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
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq<&str> for Name {
    fn eq(&self, o: &&str) -> bool {
        self.0 == *o
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <String as fmt::Display>::fmt(&self.0, f)
    }
}

impl FromStr for Name {
    type Err = ParseError;
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::validate(Cow::Borrowed(name), MAX_NAME_LENGTH)?;
        Ok(Self(name.to_owned()))
    }
}

impl From<Name> for String {
    fn from(name: Name) -> String {
        name.0
    }
}

/*
impl<'de> de::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Name;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(
                    "a non-empty string no more than 100 characters in length, \
                    consisting of [A-Za-z0-9_.-] and starting with [A-Za-z0-9_]",
                )
            }
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                s.parse().map_err(|err| match err {
                    ParseError::InvalidValue => E::invalid_value(
                        de::Unexpected::Str(s),
                        &"a name that consists of [A-Za-z0-9_.-] and starts with [A-Za-z0-9_]",
                    ),
                    ParseError::InvalidLength => E::invalid_length(
                        s.len(),
                        &"a non-empty name no more than 100 characters in length",
                    ),
                    e => {
                        panic!("unexpected parse error: {:?}", e);
                    }
                })
            }
        }
        deserializer.deserialize_string(Visitor)
    }
}
*/

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
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(rename_all = "snake_case"))]
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
