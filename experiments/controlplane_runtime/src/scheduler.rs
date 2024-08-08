//! Delays and deduplicates [`Stream`](futures::stream::Stream) items

use futures::{stream::Fuse, Stream, StreamExt};
use hashbrown::{hash_map::Entry, HashMap};
use pin_project::pin_project;
use std::{
    collections::HashSet,
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::Instant;
use tokio_util::time::delay_queue::{self, DelayQueue};

/// A request to re-emit `message` at a given `Instant` (`run_at`).
#[derive(Debug)]
pub struct ScheduleRequest<T> {
    pub message: T,
    pub run_at: Instant,
}

/// Internal metadata for a scheduled message.
struct ScheduledEntry {
    run_at: Instant,
    queue_key: delay_queue::Key,
}

#[pin_project(project = SchedulerProj)]
pub struct Scheduler<T, R> {
    /// Queue of already-scheduled messages.
    ///
    /// To ensure that the metadata is kept up-to-date, use `schedule_message` and
    /// `poll_pop_queue_message` rather than manipulating this directly.
    ///
    /// NOTE: `scheduled` should be considered to hold the "canonical" representation of the message.
    /// Always pull the message out of `scheduled` once it has been retrieved from `queue`.
    queue: DelayQueue<T>,

    /// Metadata for all currently scheduled messages. Used to detect duplicate messages.
    ///
    /// `scheduled` is considered to hold the "canonical" representation of the message.
    scheduled: HashMap<T, ScheduledEntry>,

    /// Messages that are scheduled to have happened, but have been held using `hold_unless`.
    pending: HashSet<T>,

    /// Incoming queue of scheduling requests.
    #[pin]
    requests: Fuse<R>,

    /// Debounce time to allow for deduplication of requests. It is added to the request's
    /// initial expiration time. If another request with the same message arrives before
    /// the request expires, its added to the new request's expiration time. This allows
    /// for a request to be emitted, if the scheduler is "uninterrupted" for the configured
    /// debounce period. Its primary purpose to deduplicate requests that expire instantly.
    debounce: Duration,
}

impl<T, R: Stream> Scheduler<T, R> {
    fn new(requests: R, debounce: Duration) -> Self {
        Self {
            queue: DelayQueue::new(),
            scheduled: HashMap::new(),
            pending: HashSet::new(),
            requests: requests.fuse(),
            debounce,
        }
    }
}

impl<'a, T: Hash + Eq + Clone, R> SchedulerProj<'a, T, R> {
    /// Attempt to schedule a message into the queue.
    ///
    /// If the message is already in the queue then the earlier `request.run_at` takes precedence.
    fn schedule_message(&mut self, request: ScheduleRequest<T>) {
        if self.pending.contains(&request.message) {
            // Message is already pending, so we can't even expedite it
            return;
        }

        let next_time = request.run_at.checked_add(*self.debounce).unwrap_or_else(far_future);

        match self.scheduled.entry(request.message) {
            // If new request is supposed to be earlier than the current entry's scheduled
            // time (for eg: the new request is user triggered and the current entry is the
            // reconciler's usual retry), then give priority to the new request.
            Entry::Occupied(mut old_entry) if old_entry.get().run_at >= request.run_at => {
                // Old entry will run after the new request, so replace it..
                let entry = old_entry.get_mut();
                self.queue.reset_at(&entry.queue_key, next_time);
                entry.run_at = next_time;
                old_entry.replace_key();
            }
            Entry::Occupied(_old_entry) => {
                // Old entry will run before the new request, so ignore the new request..
            }
            Entry::Vacant(entry) => {
                // No old entry, we're free to go!
                let message = entry.key().clone();
                entry.insert(ScheduledEntry {
                    run_at: next_time,
                    queue_key: self.queue.insert_at(message, next_time),
                });
            }
        }
    }

    /// Attempt to retrieve a message from the queue.
    fn poll_pop_queue_message(&mut self, cx: &mut Context<'_>, can_take_message: impl Fn(&T) -> bool) -> Poll<T> {
        if let Some(msg) = self.pending.iter().find(|msg| can_take_message(*msg)).cloned() {
            return Poll::Ready(self.pending.take(&msg).unwrap());
        }

        loop {
            match self.queue.poll_expired(cx) {
                Poll::Ready(Some(msg)) => {
                    let msg = msg.into_inner();
                    let (msg, _) = self
                        .scheduled
                        .remove_entry(&msg)
                        .expect("Expired message was popped from the Scheduler queue, but was not in the metadata map");
                    if can_take_message(&msg) {
                        break Poll::Ready(msg);
                    }
                    self.pending.insert(msg);
                }
                Poll::Ready(None) | Poll::Pending => break Poll::Pending,
            }
        }
    }

    /// Attempt to retrieve a message from queue and mark it as pending.
    pub fn pop_queue_message_into_pending(&mut self, cx: &mut Context<'_>) {
        while let Poll::Ready(Some(msg)) = self.queue.poll_expired(cx) {
            let msg = msg.into_inner();
            self.scheduled
                .remove_entry(&msg)
                .expect("Expired message was popped from the Scheduler queue, but was not in the metadata map");
            self.pending.insert(msg);
        }
    }
}

/// See [`Scheduler::hold`]
pub struct Hold<'a, T, R> {
    scheduler: Pin<&'a mut Scheduler<T, R>>,
}

impl<'a, T, R> Stream for Hold<'a, T, R>
where
    T: Eq + Hash + Clone,
    R: Stream<Item = ScheduleRequest<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut scheduler = this.scheduler.as_mut().project();

        loop {
            match scheduler.requests.as_mut().poll_next(cx) {
                Poll::Ready(Some(request)) => scheduler.schedule_message(request),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => break,
            }
        }

        scheduler.pop_queue_message_into_pending(cx);
        Poll::Pending
    }
}

/// See [`Scheduler::hold_unless`]
pub struct HoldUnless<'a, T, R, C> {
    scheduler: Pin<&'a mut Scheduler<T, R>>,
    can_take_message: C,
}

impl<'a, T, R, C> Stream for HoldUnless<'a, T, R, C>
where
    T: Eq + Hash + Clone,
    R: Stream<Item = ScheduleRequest<T>>,
    C: Fn(&T) -> bool + Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let can_take_message = &this.can_take_message;
        let mut scheduler = this.scheduler.as_mut().project();

        loop {
            match scheduler.requests.as_mut().poll_next(cx) {
                Poll::Ready(Some(request)) => scheduler.schedule_message(request),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => break,
            }
        }

        match scheduler.poll_pop_queue_message(cx, can_take_message) {
            Poll::Ready(expired) => Poll::Ready(Some(expired)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, R> Scheduler<T, R>
where
    T: Eq + Hash + Clone,
    R: Stream<Item = ScheduleRequest<T>>,
{
    /// A filtered view of the [`Scheduler`], which will keep items "pending" if
    /// `can_take_message` returns `false`, allowing them to be handled as soon as
    /// they are ready.
    ///
    /// The returned [`HoldUnless`] is designed to be short-lived: it has no allocations, and
    /// no messages will be lost, even if it is reconstructed on each call to [`poll_next`](Self::poll_next).
    /// In fact, this is often desirable, to avoid long-lived borrows in `can_take_message`'s closure.
    ///
    /// NOTE: `can_take_message` should be considered to be fairly performance-sensitive, since
    /// it will generally be executed for each pending message, for each [`poll_next`](Self::poll_next).
    pub fn hold_unless<C: Fn(&T) -> bool>(self: Pin<&mut Self>, can_take_message: C) -> HoldUnless<T, R, C> {
        HoldUnless {
            scheduler: self,
            can_take_message,
        }
    }

    /// A restricted view of the [`Scheduler`], which will keep all items "pending".
    /// Its equivalent to doing `self.hold_unless(|_| false)` and is useful when the
    /// consumer is not ready to consume the expired messages that the [`Scheduler`] emits.
    #[must_use]
    pub fn hold(self: Pin<&mut Self>) -> Hold<T, R> {
        Hold { scheduler: self }
    }

    /// Checks whether `msg` is currently a pending message (held by `hold_unless`)
    #[cfg(test)]
    pub fn contains_pending(&self, msg: &T) -> bool {
        self.pending.contains(msg)
    }
}

impl<T, R> Stream for Scheduler<T, R>
where
    T: Eq + Hash + Clone,
    R: Stream<Item = ScheduleRequest<T>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.hold_unless(|_| true)).poll_next(cx)
    }
}

/// Stream transformer that delays and deduplicates [`Stream`] items.
///
/// The debounce period lets the scheduler deduplicate requests that ask to be
/// emitted instantly, by making sure we wait for the configured period of time
/// to receive an uninterrupted request before actually emitting it.
///
/// For more info, see [`scheduler()`].
#[allow(clippy::module_name_repetitions)]
pub fn debounced_scheduler<T: Eq + Hash + Clone, S: Stream<Item = ScheduleRequest<T>>>(
    requests: S,
    debounce: Duration,
) -> Scheduler<T, S> {
    Scheduler::new(requests, debounce)
}

// internal fallback for overflows in schedule times
pub(crate) fn far_future() -> Instant {
    // private method from tokio for convenience - remove if upstream becomes pub
    // https://github.com/tokio-rs/tokio/blob/6fcd9c02176bf3cd570bc7de88edaa3b95ea480a/tokio/src/time/instant.rs#L57-L63
    Instant::now() + Duration::from_secs(86400 * 365 * 30)
}
