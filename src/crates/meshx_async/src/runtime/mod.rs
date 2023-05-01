mod meshx;
use self::meshx as implementation;

pub use implementation::{
    executor::{Duration, LocalExecutor, Time},
    task::{unblock, Task},
    timer::Timer,
};

// MeshX specific exports
pub use self::meshx::{
    executor::{EHandle, PacketReceiver, ReceiverRegistration, WaitState},
    timer::Interval,
};

pub(crate) use self::meshx::executor::{need_signal, schedule_packet};

use futures::prelude::*;
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use std::pin::Pin;
use std::task::{Context, Poll};

/// An extension trait to provide `after_now` on `fx::Duration`.
pub trait DurationExt {
    /// Return a `Time` which is a `Duration` after the current time.
    /// `duration.after_now()` is equivalent to `Time::after(duration)`.
    ///
    /// This method requires that an executor has been set up.
    fn after_now(self) -> Time;
}

/// The time when a Timer should wakeup.
pub trait WakeupTime {
    /// Convert this time into a fuchsia_async::Time.
    /// This is allowed to be inaccurate, but the inaccuracy must make the wakeup time later,
    /// never earlier.
    fn into_time(self) -> Time;
}
