use super::object_ref::{Lookup, ObjectRef};
use crate::{
    utils::delayed_init::{self, DelayedInit},
    watcher,
};
use ahash::AHashMap;
use derivative::Derivative;
use parking_lot::RwLock;
use std::{fmt::Debug, hash::Hash, sync::Arc};
use thiserror::Error;

type Cache<K> = Arc<RwLock<AHashMap<ObjectRef<K>, Arc<K>>>>;

/// A writable Store handle
///
/// This is exclusive since it's not safe to share a single `Store` between multiple reflectors.
/// In particular, `Restarted` events will clobber the state of other connected reflectors.
#[derive(Debug)]
pub struct Writer<K: 'static + Lookup + Clone>
where
    K::DynamicType: Eq + Hash + Clone,
{
    store: Cache<K>,
    buffer: AHashMap<ObjectRef<K>, Arc<K>>,
    dyntype: K::DynamicType,
    ready_tx: Option<delayed_init::Initializer<()>>,
    ready_rx: Arc<DelayedInit<()>>,
    // dispatcher: Option<Dispatcher<K>>,
}

impl<K: 'static + Lookup + Clone> Writer<K>
where
    K::DynamicType: Eq + Hash + Clone,
{
    /// Creates a new Writer with the specified dynamic type.
    ///
    /// If the dynamic type is default-able (for example when writer is used with
    /// `k8s_openapi` types) you can use `Default` instead.
    pub fn new(dyntype: K::DynamicType) -> Self {
        let (ready_tx, ready_rx) = DelayedInit::new();
        Writer {
            store: Default::default(),
            buffer: Default::default(),
            dyntype,
            ready_tx: Some(ready_tx),
            ready_rx: Arc::new(ready_rx),
            // dispatcher: None,
        }
    }

    /// Return a read handle to the store
    ///
    /// Multiple read handles may be obtained, by either calling `as_reader` multiple times,
    /// or by calling `Store::clone()` afterwards.
    #[must_use]
    pub fn as_reader(&self) -> Store<K> {
        Store {
            store: self.store.clone(),
            ready_rx: self.ready_rx.clone(),
        }
    }

    /// Applies a single watcher event to the store
    pub fn apply_watcher_event(&mut self, event: &watcher::Event<K>) {
        match event {
            watcher::Event::Apply(obj) => {
                let key = obj.to_object_ref(self.dyntype.clone());
                let obj = Arc::new(obj.clone());
                self.store.write().insert(key, obj);
            }
            watcher::Event::Delete(obj) => {
                let key = obj.to_object_ref(self.dyntype.clone());
                self.store.write().remove(&key);
            }
            watcher::Event::Init => {
                self.buffer = AHashMap::new();
            }
            watcher::Event::InitPage(new_objs) => {
                let new_objs = new_objs
                    .iter()
                    .map(|obj| (obj.to_object_ref(self.dyntype.clone()), Arc::new(obj.clone())));
                self.buffer.extend(new_objs);
            }
            watcher::Event::InitApply(obj) => {
                let key = obj.to_object_ref(self.dyntype.clone());
                let obj = Arc::new(obj.clone());
                self.buffer.insert(key, obj);
            }
            watcher::Event::InitDone => {
                let mut store = self.store.write();

                // Swap the buffer into the store
                std::mem::swap(&mut *store, &mut self.buffer);

                // Clear the buffer
                // This is preferred over self.buffer.clear(), as clear() will keep the allocated memory for reuse.
                // This way, the old buffer is dropped.
                self.buffer = AHashMap::new();

                // Mark as ready after the Restart, "releasing" any calls to Store::wait_until_ready()
                if let Some(ready_tx) = self.ready_tx.take() {
                    ready_tx.init(())
                }
            }
        }
    }

    /// Broadcast an event to any downstream listeners subscribed on the store
    pub(crate) async fn dispatch_event(&mut self, event: &watcher::Event<K>) {
        /*if let Some(ref mut dispatcher) = self.dispatcher {
            match event {
                watcher::Event::Apply(obj) => {
                    let obj_ref = obj.to_object_ref(self.dyntype.clone());
                    // TODO (matei): should this take a timeout to log when backpressure has
                    // been applied for too long, e.g. 10s
                    dispatcher.broadcast(obj_ref).await;
                }

                watcher::Event::InitDone => {
                    let obj_refs: Vec<_> = {
                        let store = self.store.read();
                        store.keys().cloned().collect()
                    };

                    for obj_ref in obj_refs {
                        dispatcher.broadcast(obj_ref).await;
                    }
                }

                _ => {}
            }
        }*/
    }
}

/// A readable cache of Kubernetes objects of kind `K`
///
/// Cloning will produce a new reference to the same backing store.
///
/// Cannot be constructed directly since one writer handle is required,
/// use `Writer::as_reader()` instead.
#[derive(Derivative)]
#[derivative(Debug(bound = "K: Debug, K::DynamicType: Debug"), Clone)]
pub struct Store<K: 'static + Lookup>
where
    K::DynamicType: Hash + Eq,
{
    store: Cache<K>,
    ready_rx: Arc<DelayedInit<()>>,
}

#[derive(Debug, Error)]
#[error("writer was dropped before store became ready")]
pub struct WriterDropped(delayed_init::InitDropped);

impl<K: 'static + Clone + Lookup> Store<K>
where
    K::DynamicType: Eq + Hash + Clone,
{
    /// Wait for the store to be populated by Kubernetes.
    ///
    /// Note that polling this will _not_ await the source of the stream that populates the [`Writer`].
    /// The [`reflector`](crate::reflector()) stream must be awaited separately.
    ///
    /// # Errors
    /// Returns an error if the [`Writer`] was dropped before any value was written.
    pub async fn wait_until_ready(&self) -> Result<(), WriterDropped> {
        self.ready_rx.get().await.map_err(WriterDropped)
    }

    /// Retrieve a `clone()` of the entry referred to by `key`, if it is in the cache.
    ///
    /// `key.namespace` is ignored for cluster-scoped resources.
    ///
    /// Note that this is a cache and may be stale. Deleted objects may still exist in the cache
    /// despite having been deleted in the cluster, and new objects may not yet exist in the cache.
    /// If any of these are a problem for you then you should abort your reconciler and retry later.
    /// If you use `kube_rt::controller` then you can do this by returning an error and specifying a
    /// reasonable `error_policy`.
    #[must_use]
    pub fn get(&self, key: &ObjectRef<K>) -> Option<Arc<K>> {
        let store = self.store.read();
        store
            .get(key)
            // Try to erase the namespace and try again, in case the object is cluster-scoped
            .or_else(|| {
                store.get(&{
                    let mut cluster_key = key.clone();
                    cluster_key.namespace = None;
                    cluster_key
                })
            })
            // Clone to let go of the entry lock ASAP
            .cloned()
    }
}
