use std::{pin::Pin, task::Poll};

use futures::{Future, FutureExt};
use tokio::{runtime::Handle, task::JoinHandle};

/// A [`JoinHandle`] that cancels the [`Future`] when dropped, rather than detaching it
pub struct CancelableJoinHandle<T> {
    inner: JoinHandle<T>,
}

impl<T> CancelableJoinHandle<T>
where
    T: Send + 'static,
{
    pub fn spawn(future: impl Future<Output = T> + Send + 'static, runtime: &Handle) -> Self {
        CancelableJoinHandle {
            inner: runtime.spawn(future),
        }
    }
}

impl<T> Drop for CancelableJoinHandle<T> {
    fn drop(&mut self) {
        self.inner.abort()
    }
}

impl<T> Future for CancelableJoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx).map(
            // JoinError => underlying future was either aborted (which should only happen when the handle is dropped), or
            // panicked (which should be propagated)
            Result::unwrap,
        )
    }
}
