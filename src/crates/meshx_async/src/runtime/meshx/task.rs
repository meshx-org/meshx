// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use futures::future::RemoteHandle;
use futures::prelude::*;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A handle to a future that is owned and polled by the executor.
///
/// Once a task is created, the executor will poll it until completion,
/// even if the task handle itself is not polled.
///
/// When a task is dropped its future will no longer be polled by the
/// executor. See [`Task::cancel`] for cancellation semantics.
///
/// Polling (or attempting to extract the value from) a task after the
/// executor is dropped may trigger a panic.
#[must_use]
#[derive(Debug)]
pub struct Task<T> {
    remote_handle: RemoteHandle<T>,
}

impl Task<()> {
    /// Detach this task so that it can run independently in the background.
    ///
    /// *Note*: this is usually not what you want. This API severs the control flow from the
    /// caller, making it impossible to return values (including errors). If your goal is to run
    /// multiple futures concurrently, consider if futures combinators such as
    ///
    /// * [`futures::future::join`]
    /// * [`futures::future::select`]
    /// * [`futures::select`]
    ///
    /// their error-aware variants
    ///
    /// * [`futures::future::try_join`]
    /// * [`futures::future::try_select`]
    ///
    /// or their stream counterparts
    ///
    /// * [`futures::stream::StreamExt::for_each`]
    /// * [`futures::stream::StreamExt::for_each_concurrent`]
    /// * [`futures::stream::TryStreamExt::try_for_each`]
    /// * [`futures::stream::TryStreamExt::try_for_each_concurrent`]
    ///
    /// can meet your needs.
    pub fn detach(self) {
        self.remote_handle.forget();
    }
}

impl<T: Send> Task<T> {
    /// Spawn a new task on the current executor.
    ///
    /// The task may be executed on any thread(s) owned by the current executor.
    /// See [`Task::local`] for an equivalent that ensures locality.
    ///
    /// The passed future will live until either (a) the future completes,
    /// (b) the returned [`Task`] is dropped while the executor is running, or
    /// (c) the executor is destroyed; whichever comes first.
    ///
    /// # Panics
    ///
    /// `spawn` may panic if not called in the context of an executor (e.g.
    /// within a call to `run` or `run_singlethreaded`).
    #[cfg_attr(trace_level_logging, track_caller)]
    pub fn spawn(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
        // Fuse is a combinator that will drop the underlying future as soon as it has been
        // completed to ensure resources are reclaimed as soon as possible. That gives callers that
        // await on the Task the guarantee that the future has been dropped.
        //
        // Note that it is safe to pass in a future that has already been fused. Double fusing
        // a future does not change the expected behavior.
        let future = future.fuse();
        let (future, remote_handle) = future.remote_handle();
        super::executor::spawn(future);
        Task { remote_handle }
    }

    /// Spawn a new task on the specified executor.
    ///
    /// The task may be executed on any thread(s) owned by the current executor.
    /// See [`Task::local`] for an equivalent that ensures locality.
    ///
    /// The passed future will live until either (a) the future completes,
    /// (b) the returned [`Task`] is dropped while the executor is running, or
    /// (c) the executor is destroyed; whichever comes first.
    #[cfg_attr(trace_level_logging, track_caller)]
    pub fn spawn_on(
        executor: &super::executor::EHandle,
        future: impl Future<Output = T> + Send + 'static,
    ) -> Task<T> {
        // Fuse is a combinator that will drop the underlying future as soon as it has been
        // completed to ensure resources are reclaimed as soon as possible. That gives callers that
        // await on the Task the guarantee that the future has been dropped.
        //
        // Note that it is safe to pass in a future that has already been fused. Double fusing
        // a future does not change the expected behavior.
        let future = future.fuse();
        let (future, remote_handle) = future.remote_handle();
        super::executor::spawn_on(executor, future);
        Task { remote_handle }
    }
}

impl<T> Task<T> {
    /// Spawn a new task on the thread local executor.
    ///
    /// The passed future will live until either (a) the future completes,
    /// (b) the returned [`Task`] is dropped while the executor is running, or
    /// (c) the executor is destroyed; whichever comes first.
    ///
    /// NOTE: This is not supported with a [`SendExecutor`] and will cause a
    /// runtime panic. Use [`Task::spawn`] instead.
    ///
    /// # Panics
    ///
    /// `local` may panic if not called in the context of a local executor (e.g.
    /// within a call to `run` or `run_singlethreaded`).
    #[cfg_attr(trace_level_logging, track_caller)]
    pub fn local(future: impl Future<Output = T> + 'static) -> Task<T> {
        // Fuse is a combinator that will drop the underlying future as soon as it has been
        // completed to ensure resources are reclaimed as soon as possible. That gives callers that
        // await on the Task the guarantee that the future has been dropped.
        //
        // Note that it is safe to pass in a future that has already been fused. Double fusing
        // a future does not change the expected behavior.
        let future = future.fuse();
        let (future, remote_handle) = future.remote_handle();
        super::executor::spawn_local(future);
        Task { remote_handle }
    }
}

impl<T: 'static> Task<T> {
    /// Initiate cancellation of this task.
    ///
    /// Returns the tasks output if it was available prior to cancelation.
    ///
    /// NOTE: If `None` is returned, the underlying future may continue executing for a
    /// short period before getting dropped. If so, do not assume any resources held
    /// by the task's future are released. If `Some(..)` is returned, such resources
    /// are guaranteed to be released.
    pub async fn cancel(self) -> Option<T> {
        self.remote_handle.now_or_never()
    }
}

impl<T: 'static> Future for Task<T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.remote_handle.poll_unpin(cx)
    }
}

/// Offload a blocking function call onto a different thread.
///
/// This function can be called from an asynchronous function without blocking
/// it, returning a future that can be `.await`ed normally. The provided
/// function should contain at least one blocking operation, such as:
///
/// - A synchronous syscall that does not yet have an async counterpart.
/// - A compute operation which risks blocking the executor for an unacceptable
///   amount of time.
///
/// If neither of these conditions are satisfied, just call the function normally,
/// as synchronous functions themselves are allowed within an async context,
/// as long as they are not blocking.
///
/// If you have an async function that may block, refactor the function such that
/// the blocking operations are offloaded onto the function passed to [`unblock`].
///
/// NOTE:
///
/// - The input function should not interact with the executor. Attempting to do so
///   can cause runtime errors. This includes spawning, creating new executors,
///   passing futures between the input function and the calling context, and
///   in some cases constructing async-aware types (such as IO-, IPC- and timer objects).
/// - Synchronous functions cannot be cancelled and may keep running after
///   the returned future is dropped. As a result, resources held by the function
///   should be assumed to be held until the returned future completes.
/// - This function assumes panic=abort semantics, so if the input function panics,
///   the process aborts. Behavior for panic=unwind is not defined.
// TODO(fxbug.dev/78332): Consider using a backing thread pool to alleviate the cost of
// spawning new threads if this proves to be a bottleneck.
pub fn unblock<T: 'static + Send>(
    f: impl 'static + Send + FnOnce() -> T,
) -> impl 'static + Send + Future<Output = T> {
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.map(|r| r.unwrap())
}
