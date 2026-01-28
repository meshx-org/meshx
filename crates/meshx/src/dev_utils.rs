use serde::Deserialize;
use wash_runtime::host::HostApi;
use wash_runtime::oci;
use wash_runtime::types;
use wash_runtime::types::Component;
use wash_runtime::types::LocalResources;
use wash_runtime::types::WorkloadStartResponse;

pub(crate) type WorkloadKey = (String, String); // (namespace, name)

pub(crate) fn workload_key(workload: &crate::types::Workload) -> WorkloadKey {
    (workload.namespace.clone(), workload.name.clone())
}

pub(crate)fn workloads_equal(a: &crate::types::Workload, b: &crate::types::Workload) -> bool {
    a.name == b.name
        && a.namespace == b.namespace
        && a.annotations == b.annotations
        && a.service == b.service
        && a.components == b.components
        && a.host_interfaces == b.host_interfaces
        && a.volumes == b.volumes
}

async fn fetch_wasm_image(image_ref: &crate::types::ImageRef) -> anyhow::Result<(Vec<u8>, String)> {
    match image_ref {
        crate::types::ImageRef::Oci(value) => {
            let bytes = oci::pull_component(value, oci::OciConfig::default()).await?;
            Ok(bytes)
        }
        crate::types::ImageRef::Blob(value) => {
            // TODO: Implement KV blob storage fetch
            anyhow::bail!("blob:// KV blob storage is not yet implemented: {}", value)
        }
        crate::types::ImageRef::File(value) => {
            let bytes = tokio::fs::read(value).await?;
            Ok((bytes, "".to_string()))
        }
    }
}

pub(crate) async fn workload_start(
    host: &impl HostApi,
    workload: crate::types::Workload,
) -> anyhow::Result<WorkloadStartResponse> {
    let crate::types::Workload {
        namespace,
        name,
        annotations,
        service,
        components: workload_components,
        host_interfaces: workload_host_interfaces,
        volumes,
    } = workload;

    let (components, host_interfaces) = if let Some(workload_components) = workload_components {
        let mut pulled_components = Vec::with_capacity(workload_components.len());
        for component in &workload_components {
            // TODO(lxf): Pull Secrets
            let Ok(bytes) = fetch_wasm_image(&component.image).await else {
                return Ok(types::WorkloadStartResponse {
                    workload_status: types::WorkloadStatus {
                        workload_id: "".into(),
                        workload_state: types::WorkloadState::Error,
                        message: format!("failed to pull component image: {:?}", component.image),
                    },
                });
            };
            pulled_components.push(Component {
                bytes: bytes.0.into(),
                local_resources: LocalResources::default(), /*component
                                                            .local_resources
                                                            .clone()
                                                            .map(Into::into)
                                                            .unwrap_or_default() */
                pool_size: component.pool_size,
                max_invocations: component.max_invocations,
                name: name.clone(),
            })
        }
        (
            pulled_components,
            workload_host_interfaces
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    } else {
        (vec![], vec![])
    };

    let service = if let Some(service) = service {
        let Ok(bytes) = fetch_wasm_image(&service.image).await else {
            return Ok(types::WorkloadStartResponse {
                workload_status: types::WorkloadStatus {
                    workload_id: "".into(),
                    workload_state: types::WorkloadState::Error,
                    message: format!("failed to pull service image: {:?}", service.image),
                },
            });
        };
        Some(types::Service {
            bytes: bytes.0.into(),
            local_resources: types::LocalResources::default(), /*service
                                                               .local_resources
                                                               .clone()
                                                               .map(Into::into)
                                                               .unwrap_or_default() */
            max_restarts: service.max_restarts,
        })
    } else {
        None
    };

    let volumes = vec![]; //volumes.into_iter().map(Into::into).collect();

    let request = types::WorkloadStartRequest {
        workload_id: uuid::Uuid::new_v4().to_string(),
        workload: types::Workload {
            namespace,
            name,
            annotations: annotations.unwrap_or_default(),
            service,
            components,
            host_interfaces,
            volumes,
        },
    };

    Ok(host.workload_start(request).await?.into())
}
