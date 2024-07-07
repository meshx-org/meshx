use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::{
    api::{Api, Patch, PatchParams, PostParams},
    Error,
};

use controlplane_core::response::Status;
//pub use controlplane_core::subresource::{EvictParams, LogParams};

/// Arbitrary subresources
impl<K> Api<K>
where
    K: Clone + DeserializeOwned + Debug,
{
    /// Display one or many sub-resources.
    pub async fn get_subresource(&self, subresource_name: &str, name: &str) -> Result<K, Error> {
        let mut req = self
            .request
            .get_subresource(subresource_name, name)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("get_subresource");
        self.client.request::<K>(req).await
    }

    /// Create an instance of the subresource
    pub async fn create_subresource<T>(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PostParams,
        data: Vec<u8>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut req = self
            .request
            .create_subresource(subresource_name, name, pp, data)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("create_subresource");
        self.client.request::<T>(req).await
    }

    /// Patch an instance of the subresource
    pub async fn patch_subresource<P: serde::Serialize + Debug>(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PatchParams,
        patch: &Patch<P>,
    ) -> Result<K, Error> {
        let mut req = self
            .request
            .patch_subresource(subresource_name, name, pp, patch)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("patch_subresource");
        self.client.request::<K>(req).await
    }

    /// Replace an instance of the subresource
    pub async fn replace_subresource(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PostParams,
        data: Vec<u8>,
    ) -> Result<K, Error> {
        let mut req = self
            .request
            .replace_subresource(subresource_name, name, pp, data)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("replace_subresource");
        self.client.request::<K>(req).await
    }
}


// ----------------------------------------------------------------------------

// TODO: Replace examples with owned custom resources. Bad practice to write to owned objects
// These examples work, but the job controller will totally overwrite what we do.
/// Methods for [status subresource](https://kubernetes.io/docs/tasks/access-kubernetes-api/custom-resources/custom-resource-definitions/#status-subresource).
impl<K> Api<K>
where
    K: DeserializeOwned,
{
    /// Get the named resource with a status subresource
    ///
    /// This actually returns the whole K, with metadata, and spec.
    pub async fn get_status(&self, name: &str) -> Result<K, Error> {
        let mut req = self
            .request
            .get_subresource("status", name)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("get_status");
        self.client.request::<K>(req).await
    }

    /// Patch fields on the status object
    ///
    /// NB: Requires that the resource has a status subresource.
    ///
    /// ```no_run
    /// use kube::api::{Api, PatchParams, Patch};
    /// use k8s_openapi::api::batch::v1::Job;
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = kube::Client::try_default().await?;
    /// let jobs: Api<Job> = Api::namespaced(client, "apps");
    /// let mut j = jobs.get("baz").await?;
    /// let pp = PatchParams::default(); // json merge patch
    /// let data = serde_json::json!({
    ///     "status": {
    ///         "succeeded": 2
    ///     }
    /// });
    /// let o = jobs.patch_status("baz", &pp, &Patch::Merge(data)).await?;
    /// assert_eq!(o.status.unwrap().succeeded, Some(2));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn patch_status<P: serde::Serialize + Debug>(
        &self,
        name: &str,
        pp: &PatchParams,
        patch: &Patch<P>,
    ) -> Result<K, Error> {
        let mut req = self
            .request
            .patch_subresource("status", name, pp, patch)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("patch_status");
        self.client.request::<K>(req).await
    }

    /// Replace every field on the status object
    ///
    /// This works similarly to the [`Api::replace`] method, but `.spec` is ignored.
    /// You can leave out the `.spec` entirely from the serialized output.
    ///
    /// ```no_run
    /// use kube::api::{Api, PostParams};
    /// use k8s_openapi::api::batch::v1::{Job, JobStatus};
    /// # async fn wrapper() -> Result<(), Box<dyn std::error::Error>> {
    /// #   let client = kube::Client::try_default().await?;
    /// let jobs: Api<Job> = Api::namespaced(client, "apps");
    /// let mut o = jobs.get_status("baz").await?; // retrieve partial object
    /// o.status = Some(JobStatus::default()); // update the job part
    /// let pp = PostParams::default();
    /// let o = jobs.replace_status("baz", &pp, serde_json::to_vec(&o)?).await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub async fn replace_status(&self, name: &str, pp: &PostParams, data: Vec<u8>) -> Result<K, Error> {
        let mut req = self
            .request
            .replace_subresource("status", name, pp, data)
            .map_err(Error::BuildRequest)?;
        req.extensions_mut().insert("replace_status");
        self.client.request::<K>(req).await
    }
}
