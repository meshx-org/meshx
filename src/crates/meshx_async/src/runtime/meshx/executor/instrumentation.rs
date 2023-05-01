// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! MeshX Executor Instrumentation.
//!
//! This module contains types used for instrumenting the fuchsia-async executor.
//! It exposes an event style API, intended to be invoked at certain points
//! during execution time, agnostic to the implementation of the executor.
//! The current implementation records counts and time slices with a focus
//! on minimal overhead.

use fiber_rust as fx;
use std::{
    cmp, mem,
    panic::Location,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use super::local::WakeupReason;

/// A low-overhead metrics collection type intended for use by an async executor.
#[derive(Default)]
pub struct Collector {
    tasks_created: AtomicUsize,
    tasks_completed: AtomicUsize,
    tasks_pending_max: AtomicUsize,
    polls: AtomicUsize,
    wakeups_io: AtomicUsize,
    wakeups_deadline: AtomicUsize,
    wakeups_notification: AtomicUsize,
    ticks_awake: AtomicU64,
    ticks_asleep: AtomicU64,
}

impl Collector {
    /// Create a new blank collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Called when a task is created (usually, this means spawned).
    pub fn task_created(&self, _id: usize, _source: Option<&Location<'_>>) {
        #[cfg(trace_level_logging)]
        tracing::trace!(
            tag = "fuchsia_async",
            id = _id,
            source = %_source.unwrap(),
            "Task spawned"
        );
        self.tasks_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Called when a task is complete.
    pub fn task_completed(&self, _id: usize, _source: Option<&Location<'_>>) {
        #[cfg(trace_level_logging)]
        tracing::trace!(
            tag = "fuchsia_async",
            id = _id,
            source = %_source.unwrap(),
            "Task completed"
        );
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Creates a local collector. Each run loop should have its own local collector.
    /// The local collector is initially awake, and has no recorded events.
    pub fn create_local_collector(&self) -> LocalCollector<'_> {
        LocalCollector {
            collector: &self,
            last_ticks: fx::ticks_get(),
            polls: 0,
            tasks_pending_max: 0, // Loading not necessary, handled by first update with same cost
        }
    }
}

/// A logical sub-collector type of a `Collector`, for a local run-loop.
pub struct LocalCollector<'a> {
    /// The main collector of this local collector
    collector: &'a Collector,
    /// Ticks since the awake state was last toggled
    last_ticks: i64,
    /// Number of polls since last `will_wait`
    polls: usize,
    /// Last observed `tasks_pending_max` from main Collector
    tasks_pending_max: usize,
}

impl<'a> LocalCollector<'a> {
    /// Called after a task was polled. If the task completed, `complete`
    /// should be true. `pending_tasks` is the observed size of the pending task
    /// queue (excluding the currently polled task).
    pub fn task_polled(&mut self, id: usize, source: Option<&Location<'_>>, complete: bool, tasks_pending: usize) {
        self.polls += 1;
        let new_local_max = cmp::max(self.tasks_pending_max, tasks_pending);
        if new_local_max > self.tasks_pending_max {
            let prev_upstream_max = self
                .collector
                .tasks_pending_max
                .fetch_max(new_local_max, Ordering::Relaxed);
            self.tasks_pending_max = cmp::max(new_local_max, prev_upstream_max);
        }
        if complete {
            self.collector.task_completed(id, source);
        }
    }

    /// Called before the loop waits. Must be followed by a `woke_up` call.
    pub fn will_wait(&mut self) {
        let delta = self.bump_ticks();
        self.collector.ticks_awake.fetch_add(delta, Ordering::Relaxed);
        self.collector
            .polls
            .fetch_add(mem::replace(&mut self.polls, 0), Ordering::Relaxed);
    }

    /// Called after the loop wakes up from waiting, containing the reason
    /// for the wakeup. Must follow a `will_wait` call.
    pub fn woke_up(&mut self, wakeup_reason: WakeupReason) {
        let delta = self.bump_ticks();
        let counter = match wakeup_reason {
            WakeupReason::Io => &self.collector.wakeups_io,
            WakeupReason::Deadline => &self.collector.wakeups_deadline,
            WakeupReason::Notification => &self.collector.wakeups_notification,
        };
        counter.fetch_add(1, Ordering::Relaxed);
        self.collector.ticks_asleep.fetch_add(delta, Ordering::Relaxed);
    }

    /// Helper which replaces `last_ticks` with the current ticks.
    /// Returns the ticks elapsed since `last_ticks`.
    fn bump_ticks(&mut self) -> u64 {
        let current_ticks = fx::ticks_get();
        let delta = current_ticks - self.last_ticks;
        assert!(delta >= 0, "time moved backwards in zx::ticks_get()");
        self.last_ticks = current_ticks;
        delta as u64
    }
}
