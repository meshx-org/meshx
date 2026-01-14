use crate::{
    cli::{CliCommand, CliContext, CommandOutput},
    config::{Config, load_config},
};
use anyhow::{Context as _, ensure};
use clap::Args;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use tokio::{fs::File, select, sync::mpsc};
use tracing::{debug, error, info};

use wash_runtime::{
    host::Host,
    plugin::{wasi_config::WasiConfig, wasi_http::HttpServer, wasi_logging::WasiLogging},
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
        info!(manifest = ?self.manifest_path, "starting development session for project");

        let _config = load_config(
            &ctx.config_path(),
            Some(self.manifest_path.as_path()),
            // Override the component path with the one provided in the command line
            Some(Config {}),
        )
        .context("failed to load config for development")?;

        let mut file = File::open(&self.manifest_path).await?;

        info!(manifest = ?self.manifest_path, "watching file for updates");

        let _running: HashMap<crate::types::WorkloadKey, crate::types::Workload> = HashMap::new();

        let mut host_builder = Host::builder();

        // Enable wasi config
        host_builder = host_builder.with_plugin(Arc::new(WasiConfig::default()))?;

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

        // TODO(#19): Only spawn the server if the component exports wasi:http
        // Configure HTTP server with optional TLS, enable HTTP Server
        let protocol = if let (Some(cert_path), Some(key_path)) = (&self.tls_cert, &self.tls_key) {
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

            if let Some(ca_path) = &self.tls_ca {
                ensure!(
                    ca_path.exists(),
                    "CA certificate file does not exist: {}",
                    ca_path.display()
                );
            }

            host_builder = host_builder.with_plugin(Arc::new(
                HttpServer::new_with_tls(
                    self.address.parse()?,
                    cert_path,
                    key_path,
                    self.tls_ca.as_deref(),
                )
                .await?,
            ))?;

            debug!("TLS configured - server will use HTTPS");
            "https"
        } else {
            debug!("No TLS configuration provided - server will use HTTP");
            host_builder =
                host_builder.with_plugin(Arc::new(HttpServer::new(self.address.parse()?)))?;
            "http"
        };

        // Add logging plugin
        host_builder = host_builder.with_plugin(Arc::new(WasiLogging))?;
        debug!("Logging plugin registered");

        // Build and start the host
        let _host = host_builder.build()?.start().await?;

        match load_state_file(&mut file).await {
            Ok(cfg) => {
                debug!("loaded file {:?}", cfg);
            }
            Err(e) => error!("failed to reload config: {e}"),
        }

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
