use crate::{
    cli::{
        CliCommand, CliContext, CommandOutput,
        doctor::{check_project_specific_tools, detect_project_context},
    },
    config::{Config, load_config},
};
use anyhow::{Context as _, ensure};
use clap::Args;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use tokio::{fs::File, select, sync::mpsc};
use tracing::{debug, error, info, warn};

use wash_runtime::{
    host::{
        Host,
        http::{DevRouter, HttpServer},
    },
    plugin::{wasi_config::DynamicConfig, wasi_logging::TracingLogger},
};

async fn load_state_file(file: &mut File) -> anyhow::Result<crate::types::ConfigFile> {
    file.seek(SeekFrom::Start(0)).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let cfg: crate::types::ConfigFile = serde_yaml::from_str(&content)?;
    Ok(cfg)
}

#[derive(Debug, Clone, Args)]
pub struct DevCommand {
    /// The path to the project directory
    #[clap(name = "manifest-file", default_value = "./state.yaml")]
    pub manifest_path: PathBuf,

    /// The address on which the HTTP server will listen
    #[clap(long = "address", default_value = "0.0.0.0:8000")]
    pub address: String,

    // TODO: filesystem root?
    /// The root directory for the blobstore to use for `wasi:blobstore/blobstore`. Defaults to a subfolder in the meshx data directory.
    #[clap(long = "blobstore-root")]
    pub blobstore_root: Option<PathBuf>,

    /// Path to TLS certificate file (PEM format) for HTTPS support
    #[clap(long = "tls-cert", requires = "tls_key")]
    pub tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format) for HTTPS support
    #[clap(long = "tls-key", requires = "tls_cert")]
    pub tls_key: Option<PathBuf>,

    /// Path to CA certificate bundle (PEM format) for client certificate verification (optional)
    #[clap(long = "tls-ca")]
    pub tls_ca: Option<PathBuf>,
}

impl CliCommand for DevCommand {
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        let project_dir = ctx.project_dir();
        info!(path = ?project_dir, "starting development session for project");

        let config = load_config(&ctx.user_config_path(), Some(project_dir), None::<Config>)
            .context("failed to load config for development")?;

        // Check for required tools (e.g., yel, WIT)
        let project_context = detect_project_context(project_dir)
            .await
            .context("failed to detect project context")?;
        let (issues, recommendations) = check_project_specific_tools(&project_context)
            .await
            .context("failed to check project specific tools")?;
        if !issues.is_empty() {
            for issue in issues {
                warn!(issue = issue, "project tool issue");
            }
        } else {
            debug!("no issues found with project tools");
        }
        if !recommendations.is_empty() {
            for recommendation in recommendations {
                warn!(
                    recommendation = recommendation,
                    "project tool recommendation"
                );
            }
        } else {
            debug!("no recommendations found for project tools");
        }

        let dev_config = config.dev();
        let http_addr = dev_config
            .address
            .clone()
            .unwrap_or_else(|| "0.0.0.0:8000".to_string());

        let mut file = File::open(&self.manifest_path).await?;

        info!(manifest = ?self.manifest_path, "watching file for updates");

        let _running: HashMap<crate::types::WorkloadKey, crate::types::Workload> = HashMap::new();

        let mut host_builder = Host::builder();

        // Enable wasi config
        host_builder = host_builder.with_plugin(Arc::new(DynamicConfig::default()))?;

        let volume_root = self
            .blobstore_root
            .clone()
            .unwrap_or_else(|| ctx.data_dir().join("dev_blobstore"));

        // Ensure the blobstore root directory exists
        if !volume_root.exists() {
            tokio::fs::create_dir_all(&volume_root)
                .await
                .context("failed to create blobstore root directory")?;
        }
        debug!(path = ?volume_root.display(), "using blobstore root directory");

        let http_handler = DevRouter::default();
        // TODO(#19): Only spawn the server if the component exports wasi:http
        // Configure HTTP server with optional TLS, enable HTTP Server
        let protocol = if let (Some(cert_path), Some(key_path)) =
            (&dev_config.tls_cert_path, &dev_config.tls_key_path)
        {
            ensure!(
                cert_path.exists(),
                "TLS certificate file does not exist: {}",
                cert_path.display()
            );
            ensure!(
                key_path.exists(),
                "TLS private key file does not exist: {}",
                key_path.display()
            );

            if let Some(ca_path) = &dev_config.tls_ca_path {
                ensure!(
                    ca_path.exists(),
                    "CA certificate file does not exist: {}",
                    ca_path.display()
                );
            }

            let http_server = HttpServer::new_with_tls(
                http_handler,
                http_addr.parse()?,
                cert_path,
                key_path,
                dev_config.tls_ca_path.as_deref(),
            )
            .await?;

            host_builder = host_builder.with_http_handler(Arc::new(http_server));

            debug!("TLS configured - server will use HTTPS");
            "https"
        } else {
            debug!("No TLS configuration provided - server will use HTTP");
            let http_server = HttpServer::new(http_handler, http_addr.parse()?);
            host_builder = host_builder.with_http_handler(Arc::new(http_server));
            "http"
        };

        // Add logging plugin
        host_builder = host_builder.with_plugin(Arc::new(TracingLogger::default()))?;
        debug!("Logging plugin registered");

        // Build and start the host
        let _host = host_builder.build()?.start().await?;

        match load_state_file(&mut file).await {
            Ok(cfg) => {
                debug!("loaded file {:?}", cfg);
            }
            Err(e) => error!("failed to reload config: {e}"),
        };

        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        // Spawn a task to handle Ctrl + C signal
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .context("failed to wait for ctrl_c signal")?;
            stop_tx
                .send(())
                .await
                .context("failed to send stop signal after receiving Ctrl + c")?;
            Result::<_, anyhow::Error>::Ok(())
        });

        info!("development session started successfully");
        info!(address = %format!("{}://{}", protocol, self.address), "listening for HTTP requests");
        info!("watching for file changes (press Ctrl+c to stop)...");

        select! {
            // Process a stop
            _ = stop_rx.recv() => {
                info!("Stopping development session ...");
            },
        }

        Ok(CommandOutput::ok(
            "Development command executed successfully".to_string(),
            None,
        ))
    }
}
