use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use cm_types::ParseError;
use thiserror::Error;

pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_DYNAMIC_NAME_LENGTH: usize = 1024;
