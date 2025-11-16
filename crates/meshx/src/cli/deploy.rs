use crate::cli::{CliCommand, CliContext, CommandOutput};
use anyhow::Context as _;
use clap::Args;
use meshx_client::apis::configuration::Configuration;
use meshx_client::apis::workload_api;
use meshx_client::models::{
    WorkloadCreateRequestDto, WorkloadCreateRequestDtoData,
    WorkloadCreateRequestDtoDataComponentsInner, WorkloadCreateRequestDtoDataHostInterfacesInner,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tracing::{debug, info, warn};

/// YAML schema for the state file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StateFile {
    api_version: String,
    metadata: Metadata,
    workloads: Vec<WorkloadSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    name: String,
    #[serde(default)]
    annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkloadSpec {
    name: String,
    namespace: String,
    world: WorldSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldSpec {
    components: Vec<ComponentSpec>,
    host_interfaces: Vec<HostInterfaceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComponentSpec {
    name: String,
    image: String,
    pool_size: u32,
    max_invocations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HostInterfaceSpec {
    namespace: String,
    package: String,
    interfaces: Vec<String>,
    config: HashMap<String, String>,
}

#[derive(Debug, Clone, Args)]
pub struct DeployCommand {
    #[arg(default_value = ".")]
    pub working_directory: String,

    /// The path to the project directory
    #[clap(name = "app", short = 'a')]
    pub app: Option<String>,
}

/// Collect all YAML files from a directory
async fn collect_yaml_files(dir_path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut yaml_files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir_path)
        .await
        .context("Failed to read directory")?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .context("Failed to read directory entry")?
    {
        let path = entry.path();
        if path.is_file()
            && let Some(extension) = path.extension()
            && (extension == "yaml" || extension == "yml")
        {
            yaml_files.push(path);
        }
    }

    Ok(yaml_files)
}

/// Parse a single YAML file, returning None if it doesn't match the expected format
async fn parse_state_file(file_path: &PathBuf) -> Option<StateFile> {
    debug!("Attempting to parse: {}", file_path.display());

    let content = match tokio::fs::read_to_string(file_path).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let state: StateFile = match serde_yaml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            debug!(
                "File {} is not a valid state file: {}",
                file_path.display(),
                e
            );
            return None;
        }
    };

    // Validate API version
    if state.api_version != "meshx.co/v0" {
        debug!(
            "Skipping file {} with unsupported API version: {}",
            file_path.display(),
            state.api_version
        );
        return None;
    }

    debug!(
        "Successfully parsed {} with {} workload(s)",
        file_path.display(),
        state.workloads.len()
    );

    Some(state)
}

/// Get the app name from metadata annotations or fall back to CLI flag
fn get_app_name(metadata: &Metadata, cli_app: &Option<String>) -> Option<String> {
    // First, try to get from meshx.cloud/app annotation
    if let Some(app_name) = metadata.annotations.get("meshx.cloud/app") {
        return Some(app_name.clone());
    }

    // Fall back to CLI flag
    cli_app.clone()
}

impl CliCommand for DeployCommand {
    async fn handle(&self, _ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        info!(app = ?self.app, "starting deployment for application");

        // Determine the target path
        let target_path = if let Some(app_path) = &self.app {
            PathBuf::from(app_path)
        } else {
            PathBuf::from(&self.working_directory)
        };

        // Check if path exists
        if !target_path.exists() {
            let error_msg = format!(
                "Path not found: {}\nPlease ensure the path exists and is accessible.",
                target_path.display()
            );
            return Ok(CommandOutput::error(error_msg, None));
        }

        // Collect state files based on whether target is file or directory
        let state_files = if target_path.is_dir() {
            info!(
                "Scanning directory for YAML files: {}",
                target_path.display()
            );
            let yaml_files = match collect_yaml_files(&target_path).await {
                Ok(files) => files,
                Err(e) => {
                    return Ok(CommandOutput::error(
                        format!("Failed to read directory: {}", e),
                        None,
                    ));
                }
            };

            if yaml_files.is_empty() {
                return Ok(CommandOutput::error(
                    format!(
                        "No YAML files found in directory: {}",
                        target_path.display()
                    ),
                    None,
                ));
            }

            info!("Found {} YAML file(s)", yaml_files.len());

            let mut parsed_states = Vec::new();
            for file_path in yaml_files {
                if let Some(state) = parse_state_file(&file_path).await {
                    parsed_states.push(state);
                }
            }

            if parsed_states.is_empty() {
                return Ok(CommandOutput::error(
                    format!(
                        "No valid state files found in directory: {}. Ensure files have apiVersion: meshx.co/v0",
                        target_path.display()
                    ),
                    None,
                ));
            }

            parsed_states
        } else {
            // Single file
            info!("Reading state file from: {}", target_path.display());

            if let Some(state) = parse_state_file(&target_path).await {
                vec![state]
            } else {
                return Ok(CommandOutput::error(
                    format!(
                        "Failed to parse state file: {}. Ensure it has apiVersion: meshx.co/v0",
                        target_path.display()
                    ),
                    None,
                ));
            }
        };

        // Configure the API client
        let config = Configuration {
            base_path: "https://api.meshx.net".into(),
            user_agent: Some(format!(
                "meshx-cli/{version} ({os}; {arch})",
                version = env!("CARGO_PKG_VERSION"),
                os = std::env::consts::OS,
                arch = std::env::consts::ARCH,
            )),
            ..Default::default()
        };

        // Deploy each state file separately
        let total_manifests = state_files.len();
        let mut total_deployed_workloads = 0;
        for state in state_files {
            // Get the app name from annotations or CLI flag
            let app_name = get_app_name(&state.metadata, &self.app);

            info!(
                "Processing manifest '{}' with {} workload(s) for app: {:?}",
                state.metadata.name,
                state.workloads.len(),
                app_name
            );

            // Deploy each workload in this state file
            for workload_spec in &state.workloads {
                info!("Deploying workload: {}", workload_spec.name);

                // Convert components
                let components: Vec<WorkloadCreateRequestDtoDataComponentsInner> = workload_spec
                    .world
                    .components
                    .iter()
                    .map(|comp| WorkloadCreateRequestDtoDataComponentsInner {
                        name: comp.name.clone(),
                        image: comp.image.clone(),
                        pool_size: comp.pool_size as f64,
                        max_invocations: comp.max_invocations as f64,
                    })
                    .collect();

                // Convert host interfaces
                let host_interfaces: Vec<WorkloadCreateRequestDtoDataHostInterfacesInner> =
                    workload_spec
                        .world
                        .host_interfaces
                        .iter()
                        .map(|iface| WorkloadCreateRequestDtoDataHostInterfacesInner {
                            namespace: iface.namespace.clone(),
                            package: iface.package.clone(),
                            interfaces: iface.interfaces.clone(),
                            config: iface.config.clone(),
                        })
                        .collect();

                // Create the request
                let request = WorkloadCreateRequestDto {
                    data: Box::new(WorkloadCreateRequestDtoData {
                        name: workload_spec.name.clone(),
                        annotations: if state.metadata.annotations.is_empty() {
                            None
                        } else {
                            Some(state.metadata.annotations.clone())
                        },
                        service: None,
                        components,
                        host_interfaces,
                        volumes: None,
                    }),
                };

                // Send the request
                // TODO: Add app_name as query param when API supports it
                match workload_api::create_workload(&config, 0, request).await {
                    Ok(_) => {
                        info!(
                            "Successfully deployed workload: {} (namespace: {}, app: {:?})",
                            workload_spec.name, workload_spec.namespace, app_name
                        );
                        total_deployed_workloads += 1;
                    }
                    Err(e) => {
                        return Ok(CommandOutput::error(
                            format!("Failed to deploy workload '{}': {}", workload_spec.name, e),
                            None,
                        ));
                    }
                }
            }
        }

        Ok(CommandOutput::ok(
            format!(
                "Successfully deployed {} workload(s) from {} manifest(s)",
                total_deployed_workloads, total_manifests
            ),
            None,
        ))
    }
}
