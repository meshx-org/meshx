// Copyright 2022 MeshX Contributors. All rights reserved.
// Copyright 2016 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon timer objects.

use crate::ok;
use crate::{AsHandleRef, Handle, HandleBased, HandleRef, Status};
use fiber_sys as sys;
use std::ops;
use std::time as stdtime;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Duration(sys::fx_duration_t);

impl Duration {
    pub const INFINITE: Duration = Duration(sys::fx_duration_t::MAX);
    pub const INFINITE_PAST: Duration = Duration(sys::fx_duration_t::MIN);

    /// Returns the number of nanoseconds contained by this `Duration`.
    pub const fn into_nanos(self) -> i64 {
        self.0
    }

    /// Returns the total number of whole microseconds contained by this `Duration`.
    pub const fn into_micros(self) -> i64 {
        self.0 / 1_000
    }

    /// Returns the total number of whole milliseconds contained by this `Duration`.
    pub const fn into_millis(self) -> i64 {
        self.into_micros() / 1_000
    }
    /// Returns the total number of whole seconds contained by this `Duration`.
    pub const fn into_seconds(self) -> i64 {
        self.into_millis() / 1_000
    }

    /// Returns the total number of whole minutes contained by this `Duration`.
    pub const fn into_minutes(self) -> i64 {
        self.into_seconds() / 60
    }

    /// Returns the total number of whole hours contained by this `Duration`.
    pub const fn into_hours(self) -> i64 {
        self.into_minutes() / 60
    }

    pub const fn from_nanos(nanos: i64) -> Self {
        Duration(nanos)
    }

    pub const fn from_micros(micros: i64) -> Self {
        Duration(micros.saturating_mul(1_000))
    }

    pub const fn from_millis(millis: i64) -> Self {
        Duration::from_micros(millis.saturating_mul(1_000))
    }

    pub const fn from_seconds(secs: i64) -> Self {
        Duration::from_millis(secs.saturating_mul(1_000))
    }

    pub const fn from_minutes(min: i64) -> Self {
        Duration::from_seconds(min.saturating_mul(60))
    }

    pub const fn from_hours(hours: i64) -> Self {
        Duration::from_minutes(hours.saturating_mul(60))
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Time(sys::fx_time_t);

impl Time {
    pub const INFINITE: Time = Time(sys::FX_TIME_INFINITE);
    pub const INFINITE_PAST: Time = Time(sys::FX_TIME_INFINITE_PAST);
    pub const ZERO: Time = Time(0);

    /// Returns the number of nanoseconds since the epoch contained by this `Time`.
    pub const fn into_nanos(self) -> i64 {
        self.0
    }

    pub const fn from_nanos(nanos: i64) -> Self {
        Time(nanos)
    }
}
