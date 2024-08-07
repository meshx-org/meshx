mod event_flatten;
mod event_modify;

mod cancelable_join_handle;
pub use cancelable_join_handle::CancelableJoinHandle;

mod watch_ext;
pub use watch_ext::WatchStreamExt;

mod backoff_reset_timer;
pub use backoff_reset_timer::ResetTimerBackoff;

mod stream_backoff;
pub use stream_backoff::StreamBackoff;

pub(crate) mod delayed_init;

use futures::{
    stream::{self, Peekable},
    Future, Stream, StreamExt, TryStream, TryStreamExt,
};
use pin_project::pin_project;
use std::{
    fmt::Debug,
    pin::{pin, Pin},
    sync::{Arc, Mutex},
    task::Poll,
};
use stream::IntoStream;

/// Allows splitting a `Stream` into several streams that each emit a disjoint subset of the input stream's items,
/// like a streaming variant of pattern matching.
///
/// NOTE: The cases MUST be reunited into the same final stream (using `futures::stream::select` or similar),
/// since cases for rejected items will *not* register wakeup correctly, and may otherwise lose items and/or deadlock.
///
/// NOTE: The whole set of cases will deadlock if there is ever an item that no live case wants to consume.
#[pin_project]
pub(crate) struct SplitCase<S: Stream, Case> {
    // Future-unaware `Mutex` is OK because it's only taken inside single poll()s
    inner: Arc<Mutex<Peekable<S>>>,
    /// Tests whether an item from the stream should be consumed
    ///
    /// NOTE: This MUST be total over all `SplitCase`s, otherwise the input stream
    /// will get stuck deadlocked because no candidate tries to consume the item.
    should_consume_item: fn(&S::Item) -> bool,
    /// Narrows the type of the consumed type, using the same precondition as `should_consume_item`.
    ///
    /// NOTE: This MUST return `Some` if `should_consume_item` returns `true`, since we can't put
    /// an item back into the input stream once consumed.
    try_extract_item_case: fn(S::Item) -> Option<Case>,
}

impl<S, Case> Stream for SplitCase<S, Case>
where
    S: Stream + Unpin,
    S::Item: Debug,
{
    type Item = Case;

    #[allow(clippy::mut_mutex_lock)]
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        // this code triggers false positive in Clippy
        // https://github.com/rust-lang/rust-clippy/issues/9415
        // TODO: remove #[allow] once fix reaches nightly.
        let inner = this.inner.lock().unwrap();
        let mut inner = Pin::new(inner);
        let inner_peek = pin!(inner.as_mut().peek());

        match inner_peek.poll(cx) {
            Poll::Ready(Some(x_ref)) => {
                if (this.should_consume_item)(x_ref) {
                    let item = inner.as_mut().poll_next(cx);
                    match item {
                        Poll::Ready(Some(x)) => Poll::Ready(Some((this.try_extract_item_case)(x).expect(
                            "`try_extract_item_case` returned `None` despite `should_consume_item` returning `true`",
                        ))),
                        res => panic!(
                            "Peekable::poll_next() returned {res:?} when Peekable::peek() returned Ready(Some(_))"
                        ),
                    }
                } else {
                    // Handled by another SplitCase instead
                    Poll::Pending
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Splits a `TryStream` into separate `Ok` and `Error` streams.
///
/// Note: This will deadlock if one branch outlives the other
#[allow(clippy::type_complexity, clippy::arc_with_non_send_sync)]
fn trystream_split_result<S>(stream: S) -> (SplitCase<IntoStream<S>, S::Ok>, SplitCase<IntoStream<S>, S::Error>)
where
    S: TryStream + Unpin,
    S::Ok: Debug,
    S::Error: Debug,
{
    let stream = Arc::new(Mutex::new(stream.into_stream().peekable()));
    (
        SplitCase {
            inner: stream.clone(),
            should_consume_item: Result::is_ok,
            try_extract_item_case: Result::ok,
        },
        SplitCase {
            inner: stream,
            should_consume_item: Result::is_err,
            try_extract_item_case: Result::err,
        },
    )
}

/// Forwards Ok elements via a stream built from `make_via_stream`, while passing errors through unmodified
pub(crate) fn trystream_try_via<S1, S2>(
    input_stream: S1,
    make_via_stream: impl FnOnce(SplitCase<IntoStream<S1>, S1::Ok>) -> S2,
) -> impl Stream<Item = Result<S2::Ok, S1::Error>>
where
    S1: TryStream + Unpin,
    S2: TryStream<Error = S1::Error>,
    S1::Ok: Debug,
    S1::Error: Debug,
{
    let (oks, errs) = trystream_split_result(input_stream); // the select -> SplitCase
    let via = make_via_stream(oks); // the map_ok/err function
    stream::select(via.into_stream(), errs.map(Err)) // recombine
}

#[pin_project]
pub(crate) struct OnComplete<S, F> {
    #[pin]
    stream: stream::Fuse<S>,
    #[pin]
    on_complete: F,
}

impl<S: Stream, F: Future<Output = ()>> Stream for OnComplete<S, F> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            Poll::Ready(None) => match this.on_complete.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(()) => Poll::Ready(None),
            },
            x => x,
        }
    }
}
pub(crate) trait KubeRuntimeStreamExt: Stream + Sized {
    /// Runs the [`Future`] `on_complete` once the [`Stream`] finishes (by returning [`None`]).
    fn on_complete<F: Future<Output = ()>>(self, on_complete: F) -> OnComplete<Self, F> {
        OnComplete {
            stream: self.fuse(),
            on_complete,
        }
    }
}

impl<S: Stream> KubeRuntimeStreamExt for S {}
