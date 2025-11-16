use crate::{
    cli::{CliCommand, CliContext, CommandOutput},
    config::{Config, load_config},
};
use anyhow::{Context as _, ensure};
use clap::Args;
use meshx_client::apis::configuration::Configuration;
use meshx_client::apis::workload_api;
use meshx_client::models::{WorkloadCreateRequestDto, WorkloadCreateRequestDtoData};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use tokio::{fs::File, select, sync::mpsc};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Args)]
pub struct DeployCommand {
    #[arg(default_value = ".")]
    pub working_directory: String,

    /// The path to the project directory
    #[clap(name = "app", short = 'a')]
    pub app: Option<String>,
}

impl CliCommand for DeployCommand {
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        info!(app = ?self.app, "starting deployment for application");

        let mut config = Configuration::default();
        config.base_path = "http://localhost:9094".into();
        config.user_agent = Some(format!(
            "meshx-cli/{version} ({os}; {arch})",
            version = env!("CARGO_PKG_VERSION"),
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        ));

        let result = workload_api::create_workload(
            &config,
            WorkloadCreateRequestDto {
                data: Box::from(WorkloadCreateRequestDtoData {
                    name: "test-workload".into(),
                    annotations: None,
                    service: None,
                    components: vec![],
                    host_interfaces: vec![],
                    volumes: None,
                }),
            },
        )
        .await?;

        info!("{:?}", result);

        Ok(CommandOutput::ok(
            "Deployment command executed successfully".to_string(),
            None,
        ))
    }
}
