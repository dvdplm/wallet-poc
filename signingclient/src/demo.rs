use reqwest::Client;
use tracing::info;

use signingcommon::{RegisterRequest, RegisterResponse, SignRequest, SignResponse};

const SERVER_URL: &str = "https://127.0.0.1:3443";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Build client that accepts self-signed certificates
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    info!("=== Signing Service Demo ===\n");
    info!("Note: Using HTTPS with self-signed certificate\n");

    // Check server
    if client
        .get(format!("{}/health", SERVER_URL))
        .send()
        .await
        .is_err()
    {
        anyhow::bail!("Server not running on {}", SERVER_URL);
    }
    info!("✓ Server is running\n");

    // Demo 1: Sign with alice (auto-registers)
    info!("[1] Sign with new user (auto-registers)");
    let sig1 = sign_message(&client, "alice", "Hello, blockchain!").await?;
    info!("Signature: {}\n", &sig1[..20.min(sig1.len())]);

    // Demo 2: Sign again with alice (existing user)
    info!("[2] Sign with existing user");
    let sig2 = sign_message(&client, "alice", "Sign me again").await?;
    info!("Signature: {}\n", &sig2[..20.min(sig2.len())]);

    // Demo 3: Different user
    info!("[3] Sign with different user (auto-registers)");
    let sig3 = sign_message(&client, "bob", "Hi Bob").await?;
    info!("Signature: {}\n", &sig3[..20.min(sig3.len())]);

    // Demo 4: Verify different messages = different signatures
    info!("[4] Verification");
    if sig1 != sig2 {
        info!("✓ Different messages produce different signatures\n");
    }

    info!("=== Demo Complete ===");
    Ok(())
}

async fn sign_message(client: &Client, user_id: &str, message: &str) -> anyhow::Result<String> {
    let response = client
        .post(format!("{}/sign", SERVER_URL))
        .json(&SignRequest {
            user_id: user_id.to_string(),
            message: message.to_string(),
        })
        .send()
        .await?;

    if response.status() == 404 {
        info!("  → Auto-registering user...");
        let response = client
            .post(format!("{}/register", SERVER_URL))
            .json(&RegisterRequest {
                seed: user_id.as_bytes().to_vec(),
            })
            .send()
            .await?;
        assert!(response.status().is_success(), "registration works");
        let user: RegisterResponse = response.json().await?;

        // Retry signing
        let response = client
            .post(format!("{}/sign", SERVER_URL))
            .json(&SignRequest {
                user_id: user.user_id,
                message: message.to_string(),
            })
            .send()
            .await?;

        let result: SignResponse = response.json().await?;
        Ok(result.signature)
    } else {
        let result: SignResponse = response.json().await?;
        Ok(result.signature)
    }
}
