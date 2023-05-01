// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2021 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::runtime::meshx::executor::common::with_local_timer_heap;

use super::super::timer::TimerHeap;
use super::{
    common::{ExecutorTime, Inner, Notifier, EMPTY_WAKEUP_ID, MAIN_TASK_ID, TASK_READY_WAKEUP_ID},
    time::Time,
};

use fiber_rust as fx;
use futures::{task::ArcWake, FutureExt};
use pin_utils::pin_mut;
use std::{
    fmt,
    future::Future,
    marker::Unpin,
    sync::{Arc, Weak},
    task::{Context, Poll, Waker},
    usize,
};

/// The reason a waiting run-loop was woken up.
pub enum WakeupReason {
    /// An external io packet was received on the port.
    Io,
    /// Deadline from a user-space timer.
    Deadline,
    /// An executor-internal notification.
    Notification,
}

/// Indicates whether the executor can run, or is stuck waiting.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum WaitState {
    /// The executor can run immediately.
    Ready,
    /// The executor will wait for the given time or an external event.
    Waiting(Time),
}

/// A synthetic main task which represents the "main future" as passed by the user.
/// The main future can change during the lifetime of the executor, but the notification
/// mechanism is shared.
struct MainTask {
    executor: Weak<Inner>,
    notifier: Notifier,
}

impl ArcWake for MainTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        if arc_self.notifier.prepare_notify() {
            if let Some(executor) = Weak::upgrade(&arc_self.executor) {
                executor.notify_empty();
            }
        }
    }
}

impl MainTask {
    /// Poll the main future using the notification semantics of the main task.
    fn poll<F>(self: &Arc<Self>, main_future: &mut F, main_waker: &Waker) -> Poll<F::Output>
    where
        F: Future + Unpin,
    {
        self.notifier.reset();
        let main_cx = &mut Context::from_waker(&main_waker);
        main_future.poll_unpin(main_cx)
    }
}

/// A single-threaded port-based executor for Fuchsia OS.
///
/// Having a `LocalExecutor` in scope allows the creation and polling of zircon objects, such as
/// [`fuchsia_async::Channel`].
///
/// # Panics
///
/// `LocalExecutor` will panic on drop if any zircon objects attached to it are still alive. In
/// other words, zircon objects backed by a `LocalExecutor` must be dropped before it.
pub struct LocalExecutor {
    /// The inner executor state.
    inner: Arc<Inner>,
    // Synthetic main task, representing the main futures during the executor's lifetime.
    main_task: Arc<MainTask>,
    // Waker for the main task, cached for performance reasons.
    main_waker: Waker,
}

impl fmt::Debug for LocalExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalExecutor").field("port", &self.inner.port).finish()
    }
}

impl LocalExecutor {
    /// Create a new single-threaded executor running with actual time.
    pub fn new() -> Self {
        let inner = Arc::new(Inner::new(ExecutorTime::RealTime, /* is_local */ true));
        inner.clone().set_local(TimerHeap::default());
        let main_task = Arc::new(MainTask {
            executor: Arc::downgrade(&inner),
            notifier: Notifier::default(),
        });
        let main_waker = futures::task::waker(main_task.clone());
        
        Self {
            inner,
            main_task,
            main_waker,
        }
    }

    /// Run a single future to completion on a single thread, also polling other active tasks.
    pub fn run_singlethreaded<F>(&mut self, main_future: F) -> F::Output
    where
        F: Future,
    {
        self.inner
            .require_real_time()
            .expect("Error: called `run_singlethreaded` on an executor using fake time");
        let mut local_collector = self.inner.collector.create_local_collector();

        pin_mut!(main_future);
        let mut res = self.main_task.poll(&mut main_future, &self.main_waker);

        local_collector.task_polled(
            MAIN_TASK_ID,
            self.inner.source,
            /* complete */ false,
            /* pending_tasks */ self.inner.ready_tasks.len(),
        );

        loop {
            if let Poll::Ready(res) = res {
                return res;
            }

            let packet = with_local_timer_heap(|timer_heap| {
                let deadline = timer_heap.next_deadline().map(|t| t.time()).unwrap_or(Time::INFINITE);
                // into_zx: we are using real time, so the time is a monotonic time.
                local_collector.will_wait();
                match self.inner.port.wait(deadline.into_fx()) {
                    Ok(packet) => Some(packet),
                    Err(fx::Status::TIMED_OUT) => {
                        local_collector.woke_up(WakeupReason::Deadline);
                        let time_waker = timer_heap.pop().unwrap();
                        time_waker.wake();
                        None
                    }
                    Err(status) => {
                        panic!("Error calling port wait: {:?}", status);
                    }
                }
            });

            if let Some(packet) = packet {
                match packet.key() {
                    EMPTY_WAKEUP_ID => {
                        local_collector.woke_up(WakeupReason::Notification);
                        res = self.main_task.poll(&mut main_future, &self.main_waker);
                        local_collector.task_polled(
                            MAIN_TASK_ID,
                            self.inner.source,
                            /* complete */ false,
                            /* pending_tasks */ self.inner.ready_tasks.len(),
                        );
                    }
                    TASK_READY_WAKEUP_ID => {
                        local_collector.woke_up(WakeupReason::Notification);
                        self.inner.poll_ready_tasks();
                    }
                    receiver_key => {
                        local_collector.woke_up(WakeupReason::Io);
                        self.inner.deliver_packet(receiver_key as usize, packet);
                    }
                }
            }
        }
    }
}

impl Drop for LocalExecutor {
    fn drop(&mut self) {
        self.inner.mark_done();
        self.inner.on_parent_drop();
    }
}
