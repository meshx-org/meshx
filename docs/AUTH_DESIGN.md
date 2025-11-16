# CLI Authentication System Design

## Overview

This document outlines the authentication system design for the MeshX CLI, enabling users to securely authenticate with an OAuth-protected backend and deploy applications.

## Authentication Flow

### 1. OAuth Device Code Flow (Recommended for CLI)

The **Device Authorization Grant** (RFC 8628) is the recommended OAuth flow for CLI applications as it:
- Works without a local web server
- Provides a good user experience
- Is secure and doesn't require embedding client secrets

```
┌─────────────┐                                ┌──────────────┐
│             │                                │              │
│  MeshX CLI  │                                │   Backend    │
│             │                                │  (OAuth AS)  │
└──────┬──────┘                                └──────┬───────┘
       │                                              │
       │ 1. Request Device Code                       │
       │─────────────────────────────────────────────>│
       │                                              │
       │ 2. Device Code + User Code + Verification URI│
       │<─────────────────────────────────────────────│
       │                                              │
       │ 3. Display User Code & URL to User          │
       │                                              │
       │ 4. Poll for Token (every 5s)                │
       │─────────────────────────────────────────────>│
       │                                              │
       │ 5. Authorization Pending                     │
       │<─────────────────────────────────────────────│
       │                                              │
       │    (User completes auth in browser)          │
       │                                              │
       │ 6. Poll for Token                            │
       │─────────────────────────────────────────────>│
       │                                              │
       │ 7. Access Token + Refresh Token              │
       │<─────────────────────────────────────────────│
       │                                              │
       │ 8. Store tokens securely                     │
       │                                              │
```

### 2. Alternative: OAuth Authorization Code Flow with PKCE

For environments where opening a browser is reliable:

```
┌─────────────┐                                ┌──────────────┐
│             │                                │              │
│  MeshX CLI  │                                │   Backend    │
│             │                                │  (OAuth AS)  │
└──────┬──────┘                                └──────┬───────┘
       │                                              │
       │ 1. Generate PKCE Code Verifier & Challenge   │
       │                                              │
       │ 2. Open Browser with Auth URL + Challenge    │
       │─────────────────────────────────────────────>│
       │                                              │
       │    (User authenticates in browser)           │
       │                                              │
       │ 3. Redirect to localhost:PORT with code      │
       │<─────────────────────────────────────────────│
       │                                              │
       │ 4. Exchange code + verifier for tokens       │
       │─────────────────────────────────────────────>│
       │                                              │
       │ 5. Access Token + Refresh Token              │
       │<─────────────────────────────────────────────│
       │                                              │
       │ 6. Store tokens securely                     │
       │                                              │
```

## Secure Token Storage

### Storage Strategy by Platform

#### Linux
**Primary**: Use `Secret Service API` (via libsecret/D-Bus)
- KWallet (KDE)
- GNOME Keyring
- KeePassXC

**Fallback**: Encrypted file with OS user permissions

#### macOS
**Primary**: Use **Keychain Services**
- Native macOS keychain integration
- Automatic encryption and access control

#### Windows
**Primary**: Use **Windows Credential Manager** (via DPAPI)
- Data Protection API for encryption
- User-scoped credential storage

### Implementation with keyring-rs

```toml
[dependencies]
keyring = "2.3"  # Cross-platform keyring access
```

```rust
use keyring::Entry;

pub struct TokenStore {
    service: &'static str,
    username: String,
}

impl TokenStore {
    pub fn new(username: String) -> Self {
        Self {
            service: "meshx-cli",
            username,
        }
    }

    pub fn store_access_token(&self, token: &str) -> anyhow::Result<()> {
        let entry = Entry::new(self.service, &format!("{}.access_token", self.username))?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn store_refresh_token(&self, token: &str) -> anyhow::Result<()> {
        let entry = Entry::new(self.service, &format!("{}.refresh_token", self.username))?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn get_access_token(&self) -> anyhow::Result<String> {
        let entry = Entry::new(self.service, &format!("{}.access_token", self.username))?;
        Ok(entry.get_password()?)
    }

    pub fn get_refresh_token(&self) -> anyhow::Result<String> {
        let entry = Entry::new(self.service, &format!("{}.refresh_token", self.username))?;
        Ok(entry.get_password()?)
    }

    pub fn delete_tokens(&self) -> anyhow::Result<()> {
        let access_entry = Entry::new(self.service, &format!("{}.access_token", self.username))?;
        let refresh_entry = Entry::new(self.service, &format!("{}.refresh_token", self.username))?;

        let _ = access_entry.delete_password();
        let _ = refresh_entry.delete_password();

        Ok(())
    }
}
```

### Fallback: Encrypted File Storage

For systems without keyring support:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use argon2::Argon2;

pub struct EncryptedTokenStore {
    file_path: PathBuf,
    key: Key<Aes256Gcm>,
}

impl EncryptedTokenStore {
    pub fn new(config_dir: PathBuf) -> anyhow::Result<Self> {
        // Derive encryption key from machine-specific data
        let machine_id = get_machine_id()?;
        let mut key_bytes = [0u8; 32];

        Argon2::default().hash_password_into(
            machine_id.as_bytes(),
            b"meshx-cli-salt-v1",
            &mut key_bytes,
        )?;

        Ok(Self {
            file_path: config_dir.join(".tokens.enc"),
            key: Key::from_slice(&key_bytes).clone(),
        })
    }

    pub fn store_tokens(&self, tokens: &Tokens) -> anyhow::Result<()> {
        let json = serde_json::to_vec(tokens)?;
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(b"unique_nonce"); // Generate random nonce
        let ciphertext = cipher.encrypt(nonce, json.as_ref())?;

        std::fs::write(&self.file_path, ciphertext)?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.file_path)?.permissions();
            perms.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&self.file_path, perms)?;
        }

        Ok(())
    }
}
```

## Token Management

### Token Lifecycle

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: SystemTime,
    pub token_type: String,
}

impl Tokens {
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    pub fn needs_refresh(&self) -> bool {
        // Refresh 5 minutes before expiry
        let now = SystemTime::now();
        let threshold = self.expires_at - Duration::from_secs(300);
        now >= threshold
    }
}
```

### Automatic Token Refresh

```rust
pub struct AuthClient {
    client: reqwest::Client,
    token_store: TokenStore,
    oauth_config: OAuthConfig,
}

impl AuthClient {
    pub async fn get_valid_token(&self) -> anyhow::Result<String> {
        let mut tokens = self.load_tokens()?;

        if tokens.is_expired() {
            // Token expired, try to refresh
            if let Some(refresh_token) = &tokens.refresh_token {
                tokens = self.refresh_access_token(refresh_token).await?;
                self.token_store.store_tokens(&tokens)?;
            } else {
                // No refresh token, need to re-authenticate
                bail!("Session expired. Please run 'meshx login' to re-authenticate.");
            }
        } else if tokens.needs_refresh() {
            // Proactively refresh before expiry
            if let Some(refresh_token) = &tokens.refresh_token {
                if let Ok(new_tokens) = self.refresh_access_token(refresh_token).await {
                    tokens = new_tokens;
                    self.token_store.store_tokens(&tokens)?;
                }
            }
        }

        Ok(tokens.access_token)
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> anyhow::Result<Tokens> {
        let response = self.client
            .post(&self.oauth_config.token_endpoint)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.oauth_config.client_id),
            ])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(Tokens {
            access_token: response.access_token,
            refresh_token: response.refresh_token.or(Some(refresh_token.to_string())),
            expires_at: SystemTime::now() + Duration::from_secs(response.expires_in),
            token_type: response.token_type,
        })
    }
}
```

## CLI Commands

### Command Structure

```rust
// crates/meshx/src/cli/auth/mod.rs

pub mod login;
pub mod logout;
pub mod status;
pub mod whoami;

use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum AuthCommand {
    /// Authenticate with MeshX backend
    Login(login::LoginCommand),

    /// Sign out and remove credentials
    Logout(logout::LogoutCommand),

    /// Check authentication status
    Status(status::StatusCommand),

    /// Display current user information
    Whoami(whoami::WhoamiCommand),
}
```

### Login Command

```rust
// crates/meshx/src/cli/auth/login.rs

use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct LoginCommand {
    /// OAuth provider (defaults to configured provider)
    #[clap(long)]
    pub provider: Option<String>,

    /// Use browser-based login (PKCE flow)
    #[clap(long)]
    pub browser: bool,

    /// Backend API endpoint
    #[clap(long, env = "MESHX_API_URL")]
    pub api_url: Option<String>,
}

impl CliCommand for LoginCommand {
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        let oauth_config = load_oauth_config(ctx, self.api_url.as_deref())?;

        let tokens = if self.browser {
            // Use PKCE flow with browser
            device_flow_with_pkce(&oauth_config).await?
        } else {
            // Use device code flow
            device_code_flow(&oauth_config).await?
        };

        // Store tokens securely
        let token_store = TokenStore::new(ctx)?;
        token_store.store_tokens(&tokens)?;

        // Fetch user info
        let user_info = fetch_user_info(&oauth_config, &tokens.access_token).await?;

        info!(
            user = user_info.username,
            email = user_info.email,
            "Successfully authenticated"
        );

        Ok(CommandOutput::ok(
            format!("✓ Logged in as {} ({})", user_info.username, user_info.email),
            Some(serde_json::json!({
                "username": user_info.username,
                "email": user_info.email,
                "expires_at": tokens.expires_at,
            })),
        ))
    }
}
```

### Device Code Flow Implementation

```rust
async fn device_code_flow(config: &OAuthConfig) -> anyhow::Result<Tokens> {
    let client = reqwest::Client::new();

    // 1. Request device code
    let device_response: DeviceCodeResponse = client
        .post(&config.device_authorization_endpoint)
        .form(&[("client_id", &config.client_id)])
        .send()
        .await?
        .json()
        .await?;

    // 2. Display instructions to user
    println!("\n{}", "=".repeat(60));
    println!("  MeshX Authentication");
    println!("{}", "=".repeat(60));
    println!();
    println!("  Visit: {}", device_response.verification_uri_complete);
    println!();
    println!("  And enter code: {}", device_response.user_code);
    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Optionally open browser automatically
    if dialoguer::Confirm::new()
        .with_prompt("Open browser automatically?")
        .default(true)
        .interact()?
    {
        let _ = open::that(&device_response.verification_uri_complete);
    }

    // 3. Poll for token
    let interval = Duration::from_secs(device_response.interval.unwrap_or(5));
    let expires_at = SystemTime::now() + Duration::from_secs(device_response.expires_in);

    println!("Waiting for authentication...");

    loop {
        if SystemTime::now() >= expires_at {
            bail!("Authentication timeout. Please try again.");
        }

        tokio::time::sleep(interval).await;

        let token_response = client
            .post(&config.token_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_response.device_code),
                ("client_id", &config.client_id),
            ])
            .send()
            .await?;

        if token_response.status().is_success() {
            let token_data: TokenResponse = token_response.json().await?;

            return Ok(Tokens {
                access_token: token_data.access_token,
                refresh_token: token_data.refresh_token,
                expires_at: SystemTime::now() + Duration::from_secs(token_data.expires_in),
                token_type: token_data.token_type,
            });
        }

        let error: OAuthError = token_response.json().await?;
        match error.error.as_str() {
            "authorization_pending" => continue,
            "slow_down" => {
                tokio::time::sleep(interval).await;
                continue;
            }
            "access_denied" => bail!("Authentication denied by user"),
            "expired_token" => bail!("Authentication timeout. Please try again."),
            _ => bail!("Authentication failed: {}", error.error_description.unwrap_or_default()),
        }
    }
}
```

## Deployment with Authentication

### Authenticated API Client

```rust
pub struct MeshXClient {
    client: reqwest::Client,
    auth_client: AuthClient,
    base_url: String,
}

impl MeshXClient {
    pub async fn deploy_app(&self, manifest: &AppManifest) -> anyhow::Result<Deployment> {
        let token = self.auth_client.get_valid_token().await?;

        let response = self.client
            .post(&format!("{}/v1/deployments", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(manifest)
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status() == StatusCode::UNAUTHORIZED {
                bail!("Authentication failed. Please run 'meshx login' to re-authenticate.");
            }
            bail!("Deployment failed: {}", response.status());
        }

        Ok(response.json().await?)
    }
}
```

### Deploy Command

```rust
#[derive(Debug, Clone, Args)]
pub struct DeployCommand {
    /// Path to application manifest
    #[clap(default_value = "./app.yaml")]
    pub manifest: PathBuf,

    /// Environment to deploy to
    #[clap(short, long, default_value = "production")]
    pub env: String,
}

impl CliCommand for DeployCommand {
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        // Check authentication first
        let auth_client = AuthClient::new(ctx)?;

        if !auth_client.is_authenticated()? {
            bail!("Not authenticated. Please run 'meshx login' first.");
        }

        // Load manifest
        let manifest = AppManifest::from_file(&self.manifest)?;

        // Deploy
        let client = MeshXClient::new(ctx, auth_client)?;

        println!("Deploying {} to {}...", manifest.name, self.env);

        let deployment = client.deploy_app(&manifest).await
            .context("Failed to deploy application")?;

        info!(
            app = manifest.name,
            deployment_id = deployment.id,
            "Deployment successful"
        );

        Ok(CommandOutput::ok(
            format!("✓ Deployed {} (deployment: {})", manifest.name, deployment.id),
            Some(serde_json::json!(deployment)),
        ))
    }
}
```

## Configuration

### OAuth Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub device_authorization_endpoint: String,
    pub userinfo_endpoint: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn from_backend(api_url: &str) -> anyhow::Result<Self> {
        // Fetch OAuth discovery document
        let discovery_url = format!("{}/.well-known/openid-configuration", api_url);

        // This would be an async call in practice
        let discovery: OpenIDDiscovery = reqwest::blocking::get(&discovery_url)?.json()?;

        Ok(Self {
            client_id: "meshx-cli".to_string(), // Pre-registered client ID
            authorization_endpoint: discovery.authorization_endpoint,
            token_endpoint: discovery.token_endpoint,
            device_authorization_endpoint: discovery.device_authorization_endpoint,
            userinfo_endpoint: discovery.userinfo_endpoint,
            scopes: vec!["openid".to_string(), "profile".to_string(), "deployments".to_string()],
        })
    }
}
```

### User Configuration Storage

```toml
# ~/.config/meshx/config.json
{
  "api_url": "https://api.meshx.io",
  "current_user": "user@example.com",
  "default_environment": "production"
}
```

## Security Considerations

### 1. Token Storage
- **Never** store tokens in plain text
- Use OS-provided secure storage when available
- Encrypt tokens when using file-based storage
- Set restrictive file permissions (0600 on Unix)

### 2. Token Transmission
- **Always** use HTTPS for API communication
- Include tokens in `Authorization` header, never in URL
- Validate TLS certificates (no self-signed certs in production)

### 3. Token Lifecycle
- Implement automatic token refresh
- Clear tokens on logout
- Handle token revocation gracefully
- Implement reasonable token expiry times

### 4. Error Handling
- Don't log tokens or sensitive data
- Provide clear error messages without leaking implementation details
- Handle network errors gracefully

### 5. Client Registration
- Use a pre-registered public client ID for the CLI
- Don't embed client secrets (not possible for public clients)
- Use PKCE for authorization code flows

## Dependencies Required

```toml
[dependencies]
# Existing dependencies...

# Authentication & OAuth
keyring = "2.3"                    # Secure credential storage
oauth2 = "4.4"                     # OAuth 2.0 client
openidconnect = "3.5"              # OpenID Connect support

# Cryptography (for fallback storage)
aes-gcm = "0.10"                   # AES-GCM encryption
argon2 = "0.5"                     # Key derivation
rand = "0.8"                       # Random number generation

# Utilities
open = "5.0"                       # Open URLs in browser
```

## Example Usage

### Login
```bash
# Device code flow (default)
$ meshx login
Visit: https://auth.meshx.io/device
And enter code: ABCD-1234

Waiting for authentication...
✓ Logged in as john@example.com (john.doe@example.com)

# Browser-based flow
$ meshx login --browser
Opening browser for authentication...
✓ Logged in as john@example.com (john.doe@example.com)
```

### Deploy
```bash
$ meshx deploy
Deploying my-app to production...
✓ Deployed my-app (deployment: dep_abc123)
```

### Check Status
```bash
$ meshx auth status
✓ Authenticated as john@example.com
Token expires: 2025-11-17 12:00:00 UTC
```

### Logout
```bash
$ meshx logout
✓ Logged out successfully
```

## Future Enhancements

1. **Multi-account Support**: Support multiple authenticated accounts
2. **SSO Integration**: Support enterprise SSO providers
3. **MFA Support**: Handle multi-factor authentication flows
4. **API Key Alternative**: Support API keys for CI/CD environments
5. **Token Rotation**: Implement automatic token rotation policies
6. **Audit Logging**: Log authentication events for security monitoring

## References

- [RFC 8628 - OAuth 2.0 Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [RFC 7636 - Proof Key for Code Exchange (PKCE)](https://datatracker.ietf.org/doc/html/rfc7636)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [OAuth 2.0 for Native Apps](https://datatracker.ietf.org/doc/html/rfc8252)
