use controlplane_core::ObjectList;
use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::error::ErrorResponse;
use crate::{api::Api, error::Error};

use controlplane_core::metadata::PartialObjectMeta;
use controlplane_core::params::{GetParams, ListParams, PatchParams, Patch, PostParams, WatchParams};
use controlplane_core::watch::WatchEvent;

/// PUSH/PUT/POST/GET abstractions
impl<K> Api<K>
where
    K: Clone + DeserializeOwned + Debug,
{
    /// Get a named resource
    ///
    /// ```no_run
    /// # use kube::Api;
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let p: Pod = pods.get("blog").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function assumes that the object is expected to always exist, and returns [`Error`] if it does not.
    /// Consider using [`Api::get_opt`] if you need to handle missing objects.
    pub async fn get(&self, name: &str) -> Result<K, Error> {
        self.get_with(name, &GetParams::default()).await
    }

    /// [Get](`Api::get`) a named resource with an explicit resourceVersion
    ///
    /// This function allows the caller to pass in a [`GetParams`](`super::GetParams`) type containing
    /// a `resourceVersion` to a [Get](`Api::get`) call.
    /// For example
    ///
    /// ```no_run
    /// # use kube::{Api, api::GetParams};
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let p: Pod = pods.get_with("blog", &GetParams::any()).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function assumes that the object is expected to always exist, and returns [`Error`] if it does not.
    /// Consider using [`Api::get_opt`] if you need to handle missing objects.
    pub async fn get_with(&self, name: &str, gp: &GetParams) -> Result<K, Error> {
        let mut req = self.request.get(name, gp).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("get");
        self.client.request::<K>(req).await
    }

    ///  Get only the metadata for a named resource as [`PartialObjectMeta`]
    ///
    /// ```no_run
    /// use kube::{Api, core::PartialObjectMeta};
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let p: PartialObjectMeta<Pod> = pods.get_metadata("blog").await?;
    /// # Ok(())
    /// # }
    /// ```
    /// Note that the type may be converted to `ObjectMeta` through the usual
    /// conversion traits.
    ///
    /// # Errors
    ///
    /// This function assumes that the object is expected to always exist, and returns [`Error`] if it does not.
    /// Consider using [`Api::get_metadata_opt`] if you need to handle missing objects.
    pub async fn get_metadata(&self, name: &str) -> Result<PartialObjectMeta<K>, Error> {
        self.get_metadata_with(name, &GetParams::default()).await
    }

    ///  [Get](`Api::get_metadata`) the metadata of an object using an explicit `resourceVersion`
    ///
    /// This function allows the caller to pass in a [`GetParams`](`super::GetParams`) type containing
    /// a `resourceVersion` to a [Get](`Api::get_metadata`) call.
    /// For example
    ///
    ///
    /// ```no_run
    /// use kube::{Api, api::GetParams, core::PartialObjectMeta};
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let p: PartialObjectMeta<Pod> = pods.get_metadata_with("blog", &GetParams::any()).await?;
    /// # Ok(())
    /// # }
    /// ```
    /// Note that the type may be converted to `ObjectMeta` through the usual
    /// conversion traits.
    ///
    /// # Errors
    ///
    /// This function assumes that the object is expected to always exist, and returns [`Error`] if it does not.
    /// Consider using [`Api::get_metadata_opt`] if you need to handle missing objects.
    pub async fn get_metadata_with(&self, name: &str, gp: &GetParams) -> Result<PartialObjectMeta<K>, Error> {
        let mut req = self.request.get_metadata(name, gp).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("get_metadata");
        self.client.request::<PartialObjectMeta<K>>(req).await
    }

    /// [Get](`Api::get`) a named resource if it exists, returns [`None`] if it doesn't exist
    ///
    /// ```no_run
    /// # use kube::Api;
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// if let Some(pod) = pods.get_opt("blog").await? {
    ///     // Pod was found
    /// } else {
    ///     // Pod was not found
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_opt(&self, name: &str) -> Result<Option<K>, Error> {
        match self.get(name).await {
            Ok(obj) => Ok(Some(obj)),
            Err(Error::Api(ErrorResponse { reason, .. })) if &reason == "NotFound" => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// [Get Metadata](`Api::get_metadata`) for a named resource if it exists, returns [`None`] if it doesn't exit
    ///
    /// ```no_run
    /// # use kube::Api;
    /// use k8s_openapi::api::core::v1::Pod;
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// if let Some(pod) = pods.get_metadata_opt("blog").await? {
    ///     // Pod was found
    /// } else {
    ///     // Pod was not found
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note that [`PartialObjectMeta`] embeds the raw `ObjectMeta`.
    pub async fn get_metadata_opt(&self, name: &str) -> Result<Option<PartialObjectMeta<K>>, Error> {
        match self.get_metadata(name).await {
            Ok(meta) => Ok(Some(meta)),
            Err(Error::Api(ErrorResponse { reason, .. })) if &reason == "NotFound" => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Get a list of resources
    ///
    /// You use this to get everything, or a subset matching fields/labels, say:
    ///
    /// ```no_run
    /// use kube::api::{Api, ListParams, ResourceExt};
    /// use k8s_openapi::api::core::v1::Pod;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let lp = ListParams::default().labels("app=blog"); // for this app only
    /// for p in pods.list(&lp).await? {
    ///     println!("Found Pod: {}", p.name_any());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, lp: &ListParams) -> Result<ObjectList<K>, Error> {
        let mut req = self.request.list(lp).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("list");
        self.client.request::<ObjectList<K>>(req).await
    }

    /// Create a resource
    ///
    /// This function requires a type that Serializes to `K`, which can be:
    /// 1. Raw string YAML
    /// - easy to port from existing files
    ///     - error prone (run-time errors on typos due to failed serialize attempts)
    ///     - very error prone (can write invalid YAML)
    /// 2. An instance of the struct itself
    ///     - easy to instantiate for CRDs (you define the struct)
    ///     - dense to instantiate for [`k8s_openapi`] types (due to many optionals)
    ///     - compile-time safety
    ///     - but still possible to write invalid native types (validation at apiserver)
    /// 3. [`serde_json::json!`] macro instantiated [`serde_json::Value`]
    ///     - Tradeoff between the two
    ///     - Easy partially filling of native [`k8s_openapi`] types (most fields optional)
    ///     - Partial safety against runtime errors (at least you must write valid JSON)
    ///
    /// Note that this method cannot write to the status object (when it exists) of a resource.
    /// To set status objects please see [`Api::replace_status`] or [`Api::patch_status`].
    pub async fn create(&self, pp: &PostParams, data: &K) -> Result<K, Error>
    where
        K: Serialize,
    {
        let bytes = serde_json::to_vec(&data).map_err(Error::SerdeError)?;
        let mut req = self.request.create(pp, bytes).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("create");
        self.client.request::<K>(req).await
    }

     /// Replace a resource entirely with a new one
    ///
    /// This is used just like [`Api::create`], but with one additional instruction:
    /// You must set `metadata.resourceVersion` in the provided data because k8s
    /// will not accept an update unless you actually knew what the last version was.
    ///
    /// Thus, to use this function, you need to do a `get` then a `replace` with its result.
    ///
    /// ```no_run
    /// use kube::api::{Api, PostParams, ResourceExt};
    /// use k8s_openapi::api::batch::v1::Job;
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let jobs: Api<Job> = Api::namespaced(client, "apps");
    /// let j = jobs.get("baz").await?;
    /// let j_new: Job = serde_json::from_value(serde_json::json!({
    ///     "apiVersion": "batch/v1",
    ///     "kind": "Job",
    ///     "metadata": {
    ///         "name": "baz",
    ///         "resourceVersion": j.resource_version(),
    ///     },
    ///     "spec": {
    ///         "template": {
    ///             "metadata": {
    ///                 "name": "empty-job-pod"
    ///             },
    ///             "spec": {
    ///                 "containers": [{
    ///                     "name": "empty",
    ///                     "image": "alpine:latest"
    ///                 }],
    ///                 "restartPolicy": "Never",
    ///             }
    ///         }
    ///     }
    /// }))?;
    /// jobs.replace("baz", &PostParams::default(), &j_new).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Consider mutating the result of `api.get` rather than recreating it.
    ///
    /// Note that this method cannot write to the status object (when it exists) of a resource.
    /// To set status objects please see [`Api::replace_status`] or [`Api::patch_status`].
    pub async fn replace(&self, name: &str, pp: &PostParams, data: &K) -> Result<K, Error>
    where
        K: Serialize,
    {
        let bytes = serde_json::to_vec(&data).map_err(Error::SerdeError)?;
        let mut req = self
            .request
            .replace(name, pp, bytes)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("replace");
        self.client.request::<K>(req).await
    }

    /// Patch a subset of a resource's properties
    ///
    /// Takes a [`Patch`] along with [`PatchParams`] for the call.
    ///
    /// ```no_run
    /// use kube::api::{Api, PatchParams, Patch, Resource};
    /// use k8s_openapi::api::core::v1::Pod;
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    ///
    /// let pods: Api<Pod> = Api::namespaced(client, "apps");
    /// let patch = serde_json::json!({
    ///     "apiVersion": "v1",
    ///     "kind": "Pod",
    ///     "metadata": {
    ///         "name": "blog"
    ///     },
    ///     "spec": {
    ///         "activeDeadlineSeconds": 5
    ///     }
    /// });
    /// let params = PatchParams::apply("myapp");
    /// let patch = Patch::Apply(&patch);
    /// let o_patched = pods.patch("blog", &params, &patch).await?;
    /// # Ok(())
    /// # }
    /// ```
    /// [`Patch`]: super::Patch
    /// [`PatchParams`]: super::PatchParams
    ///
    /// Note that this method cannot write to the status object (when it exists) of a resource.
    /// To set status objects please see [`Api::replace_status`] or [`Api::patch_status`].
    pub async fn patch<P: Serialize + Debug>(
        &self,
        name: &str,
        pp: &PatchParams,
        patch: &Patch<P>,
    ) -> Result<K, Error> {
        let mut req = self.request.patch(name, pp, patch).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("patch");
        self.client.request::<K>(req).await
    }

    /// Watch a list of resources
    ///
    /// This returns a future that awaits the initial response,
    /// then you can stream the remaining buffered `WatchEvent` objects.
    ///
    /// Note that a `watch` call can terminate for many reasons (even before the specified
    /// [`WatchParams::timeout`] is triggered), and will have to be re-issued
    /// with the last seen resource version when or if it closes.
    ///
    /// Consider using a managed [`watcher`] to deal with automatic re-watches and error cases.
    ///
    /// ```no_run
    /// use kube::api::{Api, WatchParams, ResourceExt, WatchEvent};
    /// use k8s_openapi::api::batch::v1::Job;
    /// use futures::{StreamExt, TryStreamExt};
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let jobs: Api<Job> = Api::namespaced(client, "apps");
    /// let lp = WatchParams::default()
    ///     .fields("metadata.name=my_job")
    ///     .timeout(20); // upper bound of how long we watch for
    /// let mut stream = jobs.watch(&lp, "0").await?.boxed();
    /// while let Some(status) = stream.try_next().await? {
    ///     match status {
    ///         WatchEvent::Added(s) => println!("Added {}", s.name_any()),
    ///         WatchEvent::Modified(s) => println!("Modified: {}", s.name_any()),
    ///         WatchEvent::Deleted(s) => println!("Deleted {}", s.name_any()),
    ///         WatchEvent::Bookmark(s) => {},
    ///         WatchEvent::Error(s) => println!("{}", s),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// [`WatchParams::timeout`]: super::WatchParams::timeout
    /// [`watcher`]: https://docs.rs/kube_runtime/*/kube_runtime/watcher/fn.watcher.html
    pub async fn watch(&self, wp: &WatchParams, version: &str) -> Result<impl Stream<Item = Result<WatchEvent<K>, Error>>, Error> {
        let mut req = self.request.watch(wp, version).map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("watch");
        self.client.request_events::<K>(req).await
    }

     /// Watch a list of metadata for a given resources
    ///
    /// This returns a future that awaits the initial response,
    /// then you can stream the remaining buffered `WatchEvent` objects.
    ///
    /// Note that a `watch_metadata` call can terminate for many reasons (even
    /// before the specified [`WatchParams::timeout`] is triggered), and will
    /// have to be re-issued with the last seen resource version when or if it
    /// closes.
    ///
    /// Consider using a managed [`metadata_watcher`] to deal with automatic re-watches and error cases.
    ///
    /// ```no_run
    /// use kube::api::{Api, WatchParams, ResourceExt, WatchEvent};
    /// use k8s_openapi::api::batch::v1::Job;
    /// use futures::{StreamExt, TryStreamExt};
    ///
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client: kube::Client = todo!();
    /// let jobs: Api<Job> = Api::namespaced(client, "apps");
    ///
    /// let lp = WatchParams::default()
    ///     .fields("metadata.name=my_job")
    ///     .timeout(20); // upper bound of how long we watch for
    /// let mut stream = jobs.watch(&lp, "0").await?.boxed();
    /// while let Some(status) = stream.try_next().await? {
    ///     match status {
    ///         WatchEvent::Added(s) => println!("Added {}", s.metadata.name.unwrap()),
    ///         WatchEvent::Modified(s) => println!("Modified: {}", s.metadata.name.unwrap()),
    ///         WatchEvent::Deleted(s) => println!("Deleted {}", s.metadata.name.unwrap()),
    ///         WatchEvent::Bookmark(s) => {},
    ///         WatchEvent::Error(s) => println!("{}", s),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// [`WatchParams::timeout`]: super::WatchParams::timeout
    /// [`metadata_watcher`]: https://docs.rs/kube_runtime/*/kube_runtime/watcher/fn.metadata_watcher.html
    pub async fn watch_metadata(
        &self,
        wp: &WatchParams,
        version: &str,
    ) -> Result<impl Stream<Item = Result<WatchEvent<PartialObjectMeta<K>>, Error>>, Error> {
        let mut req = self
            .request
            .watch_metadata(wp, version)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("watch_metadata");
        self.client.request_events::<PartialObjectMeta<K>>(req).await
    }
}
