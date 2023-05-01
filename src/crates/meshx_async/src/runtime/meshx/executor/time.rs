// Copyright 2021 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::common::EHandle;
use crate::runtime::DurationExt;
use fiber_rust as fx;
use std::ops;

/// A time relative to the executor's clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Time(fx::Time);

pub use fx::Duration;

impl Time {
    /// Return the current time according to the global executor.
    ///
    /// This function requires that an executor has been set up.
    pub fn now() -> Self {
        EHandle::local().inner.now()
    }

    /// Compute a deadline for the time in the future that is the
    /// given `Duration` away. Similarly to `zx::Time::after`,
    /// saturates on overflow instead of wrapping around.
    ///
    /// This function requires that an executor has been set up.
    pub fn after(duration: fx::Duration) -> Self {
        Self::now() + duration
    }

    /// Convert from `zx::Time`. This only makes sense if the time is
    /// taken from the same source (for the real clock, this is
    /// `zx::ClockId::Monotonic`).
    pub fn from_fx(t: fx::Time) -> Self {
        Time(t)
    }

    /// Convert into `zx::Time`. For the real clock, this will be a
    /// monotonic time.
    pub fn into_fx(self) -> fx::Time {
        self.0
    }

    /// Convert from nanoseconds.
    pub fn from_nanos(nanos: i64) -> Self {
        Self::from_fx(fx::Time::from_nanos(nanos))
    }

    /// Convert to nanoseconds.
    pub fn into_nanos(self) -> i64 {
        self.0.into_nanos()
    }

    /// The maximum time.
    pub const INFINITE: Time = Time(fx::Time::INFINITE);

    /// The minimum time.
    pub const INFINITE_PAST: Time = Time(fx::Time::INFINITE_PAST);
}

impl From<fx::Time> for Time {
    fn from(t: fx::Time) -> Time {
        Time(t)
    }
}

impl From<Time> for fx::Time {
    fn from(t: Time) -> fx::Time {
        t.0
    }
}

impl ops::Add<fx::Duration> for Time {
    type Output = Time;
    fn add(self, d: fx::Duration) -> Time {
        Time(self.0 + d)
    }
}

impl ops::Add<Time> for fx::Duration {
    type Output = Time;
    fn add(self, t: Time) -> Time {
        Time(self + t.0)
    }
}

impl ops::Sub<fx::Duration> for Time {
    type Output = Time;
    fn sub(self, d: fx::Duration) -> Time {
        Time(self.0 - d)
    }
}

impl ops::Sub<Time> for Time {
    type Output = fx::Duration;
    fn sub(self, t: Time) -> fx::Duration {
        self.0 - t.0
    }
}

impl ops::AddAssign<fx::Duration> for Time {
    fn add_assign(&mut self, d: fx::Duration) {
        self.0.add_assign(d)
    }
}

impl ops::SubAssign<fx::Duration> for Time {
    fn sub_assign(&mut self, d: fx::Duration) {
        self.0.sub_assign(d)
    }
}

impl DurationExt for fx::Duration {
    fn after_now(self) -> Time {
        Time::after(self)
    }
}