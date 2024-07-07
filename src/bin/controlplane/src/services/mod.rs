use std::sync::Arc;

use crate::apiservice::APIService;
use crate::error::Error;
use axum::async_trait;
use controlplane_api::apimachinery::apis::meta::v1::{APIResource, ListMeta, ObjectMeta};
use controlplane_core::discovery::ApiResource;
use controlplane_core::dynamic::DynamicObject;
use controlplane_core::error::ErrorResponse;
use controlplane_core::metadata::TypeMeta;
use controlplane_core::params::{ListParams, PostParams, WatchParams};
use controlplane_core::resource::Resource;
use controlplane_core::watch::WatchEvent;
use controlplane_core::GroupVersion;
use controlplane_core::GroupVersionKind;
use controlplane_core::Object;
use controlplane_core::ObjectList;
use controlplane_core::StoredObject;
use etcd_client::Client;
use etcd_client::EventType;
use etcd_client::GetOptions;
use etcd_client::WatchOptions;
use futures::Stream;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct StoredResource {
    client: Arc<RwLock<Client>>,
    api_resource: APIResource,
}

impl StoredResource {
    fn is(&self, name: String, namespaced: bool, verb: String) -> bool {
        self.api_resource.name.eq(&name) && !self.api_resource.namespaced && self.api_resource.verbs.contains(&verb)
    }

    async fn run(&mut self) -> Result<Object<Value, Value>, Error> {
        Err(Error::Unknown)
    }

    pub async fn create(&self, payload: Value, params: PostParams) -> Result<StoredObject<Value, Value>, Error> {
        let mut object: StoredObject<Value, Value> = serde_json::from_value(payload)?;

        let gvk = GroupVersionKind::gvk(
            self.api_resource.group.as_ref().ok_or(Error::Unknown).cloned()?,
            self.api_resource.version.as_ref().ok_or(Error::Unknown).cloned()?,
            self.api_resource.kind.clone(),
        );

        if object.types.is_none() {
            object.types = Some(TypeMeta {
                api_version: gvk.api_version(),
                kind: gvk.kind,
            })
        }

        let meta = object.metadata.clone();
        let types = object.types.clone();

        let name = meta.name.ok_or(Error::Unknown)?;

        // If the resource is namespaced and namespace
        // is missing return an error here
        if self.api_resource.namespaced && meta.namespace.is_none() {
            return Err(Error::NamespaceMissing);
        }

        let key = match meta.namespace {
            Some(ref ns) => {
                format!(
                    "/registry/{}/{}/{}/{}",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name,
                    ns,
                    name
                )
            }
            None => {
                format!(
                    "/registry/{}/{}/{}",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name,
                    name
                )
            }
        };

        let bytes = serde_json::to_vec(&object)?;
        self.client.write().await.kv_client().put(key, bytes, None).await?;

        return Ok(object);
    }
}

impl StoredResource {
    pub fn new(client: Arc<RwLock<Client>>, api_resource: APIResource) -> Self {
        Self { client, api_resource }
    }

    pub async fn watch(
        &self,
        namespace: Option<String>,
        params: WatchParams,
    ) -> Result<impl Stream<Item = WatchEvent<DynamicObject>>, Error> {
        let key = match namespace {
            Some(ref ns) => {
                format!(
                    "/registry/{}/{}/{}/",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name,
                    ns
                )
            }
            None => {
                format!(
                    "/registry/{}/{}/",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name
                )
            }
        };

        let (watcher, stream) = self
            .client
            .write()
            .await
            .watch(key, Some(WatchOptions::new().with_prefix().with_progress_notify()))
            .await?;

        Ok(stream.filter_map(|watch_response| async {
            let resp = watch_response.unwrap();

            if resp.created() {
                println!("watcher created: {}", resp.watch_id());
            }

            let event = &resp.events()[0];

            if let Some(kv) = event.kv() {
                if EventType::Put == event.event_type() {
                    // is create
                    if kv.create_revision() == kv.mod_revision() {
                        let object = serde_json::from_slice::<DynamicObject>(kv.value()).unwrap();
                        return Some(WatchEvent::Added(object));
                    } else {
                        let object = serde_json::from_slice::<DynamicObject>(kv.value()).unwrap();
                        return Some(WatchEvent::Modified(object));
                    }
                }

                if EventType::Delete == event.event_type() {
                    let object = serde_json::from_slice::<DynamicObject>(kv.value()).unwrap();
                    return Some(WatchEvent::Deleted(object));
                }
            }

            Some(WatchEvent::Error(ErrorResponse {
                status: todo!(),
                message: todo!(),
                reason: todo!(),
                code: todo!(),
            }))
        }))
    }

    pub async fn list<T>(
        &self,
        namespace: Option<String>,
        params: ListParams,
    ) -> Result<ObjectList<StoredObject<Value, Value>>, Error>
    where
        T: Resource<DynamicType = ApiResource> + Clone,
    {
        let key = match namespace {
            Some(ref ns) => {
                format!(
                    "/registry/{}/{}/{}/",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name,
                    ns
                )
            }
            None => {
                format!(
                    "/registry/{}/{}/",
                    self.api_resource.group.as_ref().ok_or(Error::Unknown)?,
                    self.api_resource.name
                )
            }
        };

        let mut resp = self
            .client
            .read()
            .await
            .kv_client()
            .get(key, Some(GetOptions::new().with_prefix()))
            .await
            .unwrap();

        let mut items = vec![];
        for k in resp.take_kvs() {
            k.mod_revision();

            let object = serde_json::from_slice::<StoredObject<Value, Value>>(k.value())?;
            items.push(object);
        }

        Ok(ObjectList {
            types: TypeMeta {
                api_version: "v1".to_owned(),
                kind: "List".to_owned(),
            },
            metadata: ListMeta::default(),
            items,
        })
    }
}

#[derive(Clone)]
pub struct Service {
    pub api_service: APIService,
    resources: Vec<StoredResource>,
    client: Client,
}

impl Service {
    pub fn new(client: Client, api_service: APIService) -> Self {
        Self {
            client,
            api_service,
            resources: vec![],
        }
    }

    pub fn add_resource(&mut self, resource: StoredResource) {
        self.resources.push(resource);
    }

    pub fn find_resource(&self, name: String, namespaced: bool, verb: String) -> Result<&StoredResource, Error> {
        self.resources
            .iter()
            .find(|res| res.is(name.clone(), namespaced, verb.clone()))
            .ok_or(Error::ResourceNotFound(name.clone()))
    }
}
