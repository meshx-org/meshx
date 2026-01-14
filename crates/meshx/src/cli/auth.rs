use std::time::Duration;

use anyhow::{Context, bail};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, info};

use super::{CliCommand, CliContext, CommandOutput};

const AUTH_SERVER: &str = "http://localhost:4001";
const TOKEN_FILE: &str = "auth_token.json";

#[derive(Debug, Clone, Args)]
pub struct AuthCommand {
    #[clap(subcommand)]
    command: AuthSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
enum AuthSubcommand {
    /// Authenticate with MeshX Cloud using device flow
    Login {
        #[clap(
            short = 'k',
            long = "api-key",
            help = "API key for authentication server"
        )]
        api_key: String,
    },
    /// Log out from MeshX Cloud
    Logout,
    /// Display information about the currently authenticated user
    Whoami {
        #[clap(
            short = 'k',
            long = "api-key",
            help = "API key for authentication server"
        )]
        api_key: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: u64,
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredToken {
    access_token: String,
    token_type: String,
    expires_at: u64,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
}

impl CliCommand for AuthCommand {
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        match &self.command {
            AuthSubcommand::Login { api_key } => handle_login(ctx, api_key).await,
            AuthSubcommand::Logout => handle_logout(ctx).await,
            AuthSubcommand::Whoami { api_key } => handle_whoami(ctx, api_key).await,
        }
    }
}

async fn handle_login(ctx: &CliContext, api_key: &str) -> anyhow::Result<CommandOutput> {
    info!("Starting device authentication flow");

    // Step 1: Request device code
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/device/code", AUTH_SERVER))
        .header("x-api-key", api_key)
        .form(&[("client_id", "test"), ("scope", "openid profile email")])
        .send()
        .await
        .context("Failed to request device code")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Ok(CommandOutput::error(
            format!(
                "Authentication server returned error ({}): {}",
                status, error_text
            ),
            Some(serde_json::json!({
                "status": status.as_u16(),
                "error": error_text
            })),
        ));
    }

    let response_text = response
        .text()
        .await
        .context("Failed to read response body")?;
    debug!("Device code response: {}", response_text);

    let device_response: DeviceCodeResponse =
        serde_json::from_str(&response_text).context(format!(
            "Failed to parse device code response. Server returned: {}",
            response_text
        ))?;

    // Step 2: Display instructions to user
    let message = format!(
        "To authenticate, visit: {}\nAnd enter code: {}\n\nWaiting for authentication...",
        device_response.verification_uri, device_response.user_code
    );
    println!("{}", message);

    // Step 3: Poll for token
    let poll_interval = Duration::from_secs(device_response.interval);
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + device_response.expires_in;

    let token = loop {
        sleep(poll_interval).await;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time > expires_at {
            bail!("Device code expired. Please try again.");
        }

        let response = client
            .post(format!("{}/api/auth/device/token", AUTH_SERVER))
            .header("x-api-key", api_key)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", device_response.device_code.as_str()),
                ("client_id", "test"),
            ])
            .send()
            .await
            .context("Failed to poll for token")?;

        if response.status().is_success() {
            let token_response: TokenResponse = response
                .json()
                .await
                .context("Failed to parse token response")?;

            break token_response;
        } else if response.status() == 400 {
            let error: serde_json::Value = response.json().await?;
            if error["error"] == "authorization_pending" {
                debug!("Authorization still pending, continuing to poll");
                continue;
            } else if error["error"] == "slow_down" {
                debug!("Slowing down polling interval");
                sleep(Duration::from_secs(5)).await;
                continue;
            } else {
                bail!("Authentication failed: {}", error["error_description"]);
            }
        } else {
            bail!("Unexpected response status: {}", response.status());
        }
    };

    // Step 4: Store token
    let stored_token = StoredToken {
        access_token: token.access_token,
        token_type: token.token_type,
        expires_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + token.expires_in,
        refresh_token: token.refresh_token,
    };

    let token_path = ctx.data_dir().join(TOKEN_FILE);
    let token_json =
        serde_json::to_string_pretty(&stored_token).context("Failed to serialize token")?;

    tokio::fs::write(&token_path, token_json)
        .await
        .context("Failed to write token file")?;

    info!("Authentication successful!");

    Ok(CommandOutput::ok(
        "Successfully authenticated with MeshX Cloud",
        Some(serde_json::json!({
            "token_path": token_path.display().to_string(),
        })),
    ))
}

async fn handle_logout(ctx: &CliContext) -> anyhow::Result<CommandOutput> {
    let token_path = ctx.data_dir().join(TOKEN_FILE);

    if !token_path.exists() {
        return Ok(CommandOutput::error("Not currently logged in", None));
    }

    tokio::fs::remove_file(&token_path)
        .await
        .context("Failed to remove token file")?;

    info!("Logged out successfully");

    Ok(CommandOutput::ok(
        "Successfully logged out from MeshX Cloud",
        None,
    ))
}

async fn handle_whoami(ctx: &CliContext, _api_key: &str) -> anyhow::Result<CommandOutput> {
    let token_path = ctx.data_dir().join(TOKEN_FILE);

    if !token_path.exists() {
        return Ok(CommandOutput::error(
            "Not currently logged in. Use 'meshx auth login' to authenticate.",
            None,
        ));
    }

    let token_json = tokio::fs::read_to_string(&token_path)
        .await
        .context("Failed to read token file")?;

    let stored_token: StoredToken =
        serde_json::from_str(&token_json).context("Failed to parse token file")?;

    // Check if token is expired
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if current_time > stored_token.expires_at {
        return Ok(CommandOutput::error(
            "Authentication token has expired. Please run 'meshx auth login' again.",
            None,
        ));
    }

    // Fetch user info from the auth server
    let client = reqwest::Client::new();
    let user_info: UserInfo = client
        .get(format!("{}/api/auth/oauth2/userinfo", AUTH_SERVER))
        //.header("x-api-key", api_key)
        .bearer_auth(&stored_token.access_token)
        .send()
        .await
        .context("Failed to fetch user info")?
        .json()
        .await
        .context("Failed to parse user info")?;

    let message = format!(
        "Logged in as:\n  User ID: {}\n  Email: {}\n  Name: {}",
        user_info.sub,
        user_info.email.as_deref().unwrap_or("N/A"),
        user_info.name.as_deref().unwrap_or("N/A")
    );

    Ok(CommandOutput::ok(
        message,
        Some(serde_json::json!({
            "user_id": user_info.sub,
            "email": user_info.email,
            "name": user_info.name,
        })),
    ))
}
