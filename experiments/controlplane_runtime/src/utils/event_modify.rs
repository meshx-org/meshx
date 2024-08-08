use pin_project::pin_project;

#[pin_project]
/// Stream returned by the [`modify`](super::WatchStreamExt::modify) method.
/// Modifies the [`Event`] item returned by the inner stream by calling
/// [`modify`](Event::modify()) on it.
pub struct EventModify<St, F> {
    #[pin]
    stream: St,
    f: F,
}
