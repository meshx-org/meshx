// Copyright 2023 MeshX Contributors. All rights reserved.
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

    /// Get the current monotonic time.
    ///
    /// Wraps the
    /// [zx_clock_get_monotonic](https://fuchsia.dev/fuchsia-src/reference/syscalls/clock_get_monotonic.md)
    /// syscall.
    pub fn get_monotonic() -> Time {
        unsafe { Time(sys::fx_clock_get_monotonic()) }
    }

    /// Returns the number of nanoseconds since the epoch contained by this `Time`.
    pub const fn into_nanos(self) -> i64 {
        self.0
    }

    pub const fn from_nanos(nanos: i64) -> Self {
        Time(nanos)
    }
}

impl From<stdtime::Duration> for Duration {
    fn from(dur: stdtime::Duration) -> Self {
        Duration::from_seconds(dur.as_secs() as i64) + Duration::from_nanos(dur.subsec_nanos() as i64)
    }
}

impl ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, dur: Duration) -> Time {
        Time::from_nanos(dur.into_nanos().saturating_add(self.into_nanos()))
    }
}

impl ops::Add<Time> for Duration {
    type Output = Time;
    fn add(self, time: Time) -> Time {
        Time::from_nanos(self.into_nanos().saturating_add(time.into_nanos()))
    }
}

impl ops::Add for Duration {
    type Output = Duration;
    fn add(self, dur: Duration) -> Duration {
        Duration::from_nanos(self.into_nanos().saturating_add(dur.into_nanos()))
    }
}

impl ops::Sub for Duration {
    type Output = Duration;
    fn sub(self, dur: Duration) -> Duration {
        Duration::from_nanos(self.into_nanos().saturating_sub(dur.into_nanos()))
    }
}

impl ops::Sub<Duration> for Time {
    type Output = Time;
    fn sub(self, dur: Duration) -> Time {
        Time::from_nanos(self.into_nanos().saturating_sub(dur.into_nanos()))
    }
}

impl ops::Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, other: Time) -> Duration {
        Duration::from_nanos(self.into_nanos().saturating_sub(other.into_nanos()))
    }
}

impl ops::AddAssign for Duration {
    fn add_assign(&mut self, dur: Duration) {
        self.0 = self.0.saturating_add(dur.into_nanos());
    }
}

impl ops::SubAssign for Duration {
    fn sub_assign(&mut self, dur: Duration) {
        self.0 = self.0.saturating_sub(dur.into_nanos());
    }
}

impl ops::AddAssign<Duration> for Time {
    fn add_assign(&mut self, dur: Duration) {
        self.0 = self.0.saturating_add(dur.into_nanos());
    }
}

impl ops::SubAssign<Duration> for Time {
    fn sub_assign(&mut self, dur: Duration) {
        self.0 = self.0.saturating_sub(dur.into_nanos());
    }
}

impl<T> ops::Mul<T> for Duration
where
    T: Into<i64>,
{
    type Output = Self;
    fn mul(self, mul: T) -> Self {
        Duration::from_nanos(self.0.saturating_mul(mul.into()))
    }
}

impl<T> ops::Div<T> for Duration
where
    T: Into<i64>,
{
    type Output = Self;
    fn div(self, div: T) -> Self {
        Duration::from_nanos(self.0.saturating_div(div.into()))
    }
}

impl ops::Neg for Duration {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(self.0.saturating_neg())
    }
}

/// Read the number of high-precision timer ticks since boot. These ticks may be processor cycles,
/// high speed timer, profiling timer, etc. They are not guaranteed to continue advancing when the
/// system is asleep.
///
/// Wraps the
/// [fx_ticks_get](https://fuchsia.dev/fuchsia-src/reference/syscalls/ticks_get.md)
/// syscall.
pub fn ticks_get() -> i64 {
    unsafe { sys::fx_ticks_get() }
}