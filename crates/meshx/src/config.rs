//! Contains the [Config] struct and related functions for managing
//! meshx configuration, including loading, saving, and merging configurations
//! with explicit defaults.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::cli::CONFIG_FILE_NAME;

pub const PROJECT_CONFIG_DIR: &str = ".meshx";

/// Main meshx configuration structure with hierarchical merging support and explicit defaults
///
/// The "global" [Config] is stored under the user's XDG_CONFIG_HOME directory
/// (typically `~/.config/meshx/config.json`), while the "local" project configuration
/// is stored in the project's `.meshx/config.json` file. This allows for both reasonable
/// global defaults and project-specific overrides.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    // TODO(#15): Support dev config which can be overridden in local project config
    // e.g. for runtime config, http ports, etc
}

impl Config {
    /// Create a new [Config] instance with default values and the list of MeshX templates
    pub fn default_with_templates() -> Self {
        Self {}
    }
}

/// Load configuration with hierarchical merging
/// Order of precedence (lowest to highest):
/// 1. Default values
/// 2. Global config (~/.meshx/config.json)
/// 3. Local project config (.meshx/config.json)
/// 4. Environment variables (MESHX_ prefix)
/// 5. Command line arguments
///
/// # Arguments
/// - `global_config_path`:
pub fn load_config<T>(
    global_config_path: &Path,
    project_dir: Option<&Path>,
    cli_args: Option<T>,
) -> Result<Config>
where
    T: Serialize + Into<Config>,
{
    let mut figment = Figment::new();

    // Start with defaults
    figment = figment.merge(figment::providers::Serialized::defaults(Config::default()));

    // Global config file
    if global_config_path.exists() {
        figment = figment.merge(Json::file(global_config_path));
    }

    // Local project config
    if let Some(project_dir) = project_dir {
        let local_config_path = project_dir.join(PROJECT_CONFIG_DIR).join(CONFIG_FILE_NAME);
        if local_config_path.exists() {
            figment = figment.merge(Json::file(local_config_path));
        }
    }

    // Environment variables with MESHX_ prefix
    figment = figment.merge(Env::prefixed("MESHX_"));

    // TODO(#16): There's more testing to be done here to ensure that CLI args can override existing
    // config without replacing present values with empty values.
    if let Some(args) = cli_args {
        // Convert CLI args to configuration format
        let cli_config: Config = args.into();
        figment = figment.merge(figment::providers::Serialized::defaults(cli_config));
    }

    figment
        .extract()
        .context("Failed to load meshx configuration")
}

/// Save configuration to specified path
pub async fn save_config(config: &Config, path: &Path) -> Result<()> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.with_context(|| {
            format!(
                "Failed to create config directory: {parent}",
                parent = parent.display()
            )
        })?;
    }

    let json = serde_json::to_string_pretty(config).context("Failed to serialize configuration")?;

    tokio::fs::write(path, json)
        .await
        .with_context(|| format!("failed to write config file: {}", path.display()))?;

    Ok(())
}

/// Generate project-specific configuration after successful build
pub async fn generate_project_config<T>(project_dir: &Path, build_args: T) -> Result<()>
where
    T: Serialize,
{
    let config_dir = project_dir.join(".meshx");
    let config_path = config_dir.join("config.json");

    // Don't overwrite existing config
    if config_path.exists() {
        return Ok(());
    }

    let config = Config::default();

    // Create a figment from the build args and extract relevant config
    let _figment = Figment::new().merge(figment::providers::Serialized::defaults(build_args));

    save_config(&config, &config_path).await?;

    info!(
        "Generated project configuration at {}",
        config_path.display()
    );
    Ok(())
}

/// Get the local project configuration file path
pub fn local_config_path(project_dir: &Path) -> PathBuf {
    project_dir.join(".meshx").join(CONFIG_FILE_NAME)
}

/// Generate a default configuration file with all explicit defaults
/// This is useful for `meshx config init` command
pub async fn generate_default_config(
    path: &Path,
    force: bool,
    include_templates: bool,
) -> Result<()> {
    // Don't overwrite existing config unless force is specified
    if path.exists() && !force {
        bail!(
            "Configuration file already exists at {}. Use --force to overwrite",
            path.display()
        );
    }

    let default_config = if include_templates {
        Config::default_with_templates()
    } else {
        Config::default()
    };
    save_config(&default_config, path).await?;

    info!(config_path = %path.display(), "Generated default configuration");
    Ok(())
}
