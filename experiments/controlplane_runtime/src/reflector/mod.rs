//! Caches objects in memory

pub mod object_ref;
pub mod store;

use async_stream::stream;
use futures::{Stream, StreamExt};
use std::hash::Hash;
use crate::watcher;

pub use self::object_ref::{Extra as ObjectRefExtra, Lookup, ObjectRef};

/// Cache objects from a [`watcher()`] stream into a local [`Store`]
///
/// Observes the raw `Stream` of [`watcher::Event`] objects, and modifies the cache.
/// It passes the raw [`watcher()`] stream through unmodified.
///
/// ## Usage
/// Create a [`Store`] through e.g. [`store::store()`]. The `writer` part is not-clonable,
/// and must be moved into the reflector. The `reader` part is the [`Store`] interface
/// that you can send to other parts of your program as state.
///
/// The cache contains the last-seen state of objects,
/// which may lag slightly behind the actual state.
///
/// ## Example
///
/// Infinite watch of [`Node`](k8s_openapi::api::core::v1::Node) resources with a certain label.
///
/// The `reader` part being passed around to a webserver is omitted.
/// For examples see [version-rs](https://github.com/kube-rs/version-rs) for integration with [axum](https://github.com/tokio-rs/axum),
/// or [controller-rs](https://github.com/kube-rs/controller-rs) for the similar controller integration with [actix-web](https://actix.rs/).
///
/// ```no_run
/// use std::future::ready;
/// use k8s_openapi::api::core::v1::Node;
/// use kube::runtime::{reflector, watcher, WatchStreamExt, watcher::Config};
/// use futures::StreamExt;
/// # use kube::api::Api;
/// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
/// # let client: kube::Client = todo!();
///
/// let nodes: Api<Node> = Api::all(client);
/// let node_filter = Config::default().labels("kubernetes.io/arch=amd64");
/// let (reader, writer) = reflector::store();
///
/// // Create the infinite reflector stream
/// let rf = reflector(writer, watcher(nodes, node_filter));
///
/// // !!! pass reader to your webserver/manager as state !!!
///
/// // Poll the stream (needed to keep the store up-to-date)
/// let infinite_watch = rf.applied_objects().for_each(|o| { ready(()) });
/// infinite_watch.await;
/// # Ok(())
/// # }
/// ```
///
///
/// ## Memory Usage
///
/// A reflector often constitutes one of the biggest components of a controller's memory use.
/// Given a ~2000 pods cluster, a reflector saving everything (including injected sidecars, managed fields)
/// can quickly consume a couple of hundred megabytes or more, depending on how much of this you are storing.
///
/// While generally acceptable, there are techniques you can leverage to reduce the memory usage
/// depending on your use case.
///
/// 1. Reflect a [`PartialObjectMeta<K>`](kube_client::core::PartialObjectMeta) stream rather than a stream of `K`
///
/// You can send in a [`metadata_watcher()`](crate::watcher::metadata_watcher()) for a type rather than a [`watcher()`],
/// and this can drop your memory usage by more than a factor of two,
/// depending on the size of `K`. 60% reduction seen for `Pod`. Usage is otherwise identical.
///
/// 2. Use `modify` the raw [`watcher::Event`] object stream to clear unneeded properties
///
/// For instance, managed fields typically constitutes around half the size of `ObjectMeta` and can often be dropped:
///
/// ```no_run
/// # use futures::TryStreamExt;
/// # use kube::{ResourceExt, Api, runtime::watcher};
/// # let api: Api<k8s_openapi::api::core::v1::Node> = todo!();
/// let stream = watcher(api, Default::default()).map_ok(|ev| {
///     ev.modify(|pod| {
///         pod.managed_fields_mut().clear();
///         pod.annotations_mut().clear();
///         pod.status = None;
///     })
/// });
/// ```
/// The `stream` can then be passed to `reflector` causing smaller objects to be written to its store.
/// Note that you **cannot drop everything**; you minimally need the spec properties your app relies on.
/// Additionally, only `labels`, `annotations` and `managed_fields` are safe to drop from `ObjectMeta`.
///
/// For more information check out: <https://kube.rs/controllers/optimization/> for graphs and techniques.
///
/// ## Stream sharing
///
/// `reflector()` as an interface may optionally create a stream that can be
/// shared with other components to help with resource usage.
///
/// To share a stream, the `Writer<K>` consumed by `reflector()` must be
/// created through an interface that allows a store to be subscribed on, such
/// as [`store_shared()`]. When the store supports being subscribed on, it will
/// broadcast an event to all active listeners after caching any object
/// contained in the event.
///
/// Creating subscribers requires an
/// [`unstable`](https://github.com/kube-rs/kube/blob/main/kube-runtime/Cargo.toml#L17-L21)
/// feature
pub fn reflector<K, W>(mut writer: store::Writer<K>, stream: W) -> impl Stream<Item = W::Item>
where
    K: Lookup + Clone,
    K::DynamicType: Eq + Hash + Clone,
    W: Stream<Item = Result<watcher::Event<K>, watcher::Error>>,
{
    let mut stream = Box::pin(stream);
    stream! {
        while let Some(event) = stream.next().await {
            match event {
                Ok(ev) => {
                    writer.apply_watcher_event(&ev);
                    writer.dispatch_event(&ev).await;
                    yield Ok(ev);
                },
                Err(ev) => yield Err(ev)
            }
        }
    }
}
