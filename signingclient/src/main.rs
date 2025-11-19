use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use signingcommon::{ErrorResponse, RegisterRequest, SignRequest, SignResponse};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "sign")]
#[command(about = "Sign a message using the remote signing service")]
struct Args {
    /// The message to sign
    #[arg(short, long)]
    message: String,

    /// The user ID for the signing service
    #[arg(short, long)]
    user_id: Option<String>,

    /// The server URL
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    let user_id = args
        .user_id
        .or_else(|| std::env::var("WALLET_USER_ID").ok())
        .ok_or_else(|| anyhow::anyhow!("User ID required (-u flag or WALLET_USER_ID env var)"))?;

    let client = Client::new();

    // Try to sign
    let response = client
        .post(format!("{}/sign", args.server))
        .json(&SignRequest {
            user_id: user_id.clone(),
            message: args.message.clone(),
        })
        .send()
        .await?;

    if response.status() == 404 {
        // User doesn't exist, register first
        info!("User not found, registering...");
        let reg_response = client
            .post(format!("{}/register", args.server))
            .json(&RegisterRequest {
                seed: user_id.as_bytes().to_vec(),
            })
            .send()
            .await?;

        if !reg_response.status().is_success() {
            error!("Registration failed");
            anyhow::bail!("Registration failed");
        }

        info!("User registered, signing message...");

        // Try signing again
        let sign_response = client
            .post(format!("{}/sign", args.server))
            .json(&SignRequest {
                user_id,
                message: args.message,
            })
            .send()
            .await?;

        if sign_response.status().is_success() {
            let result: SignResponse = sign_response.json().await?;
            println!("{}", result.signature);
            info!("Message signed successfully");
        } else {
            let err: ErrorResponse = sign_response.json().await?;
            error!("Signing failed: {}", err.error);
            anyhow::bail!("Signing failed");
        }
    } else if response.status().is_success() {
        let result: SignResponse = response.json().await?;
        println!("{}", result.signature);
        info!("Message signed successfully");
    } else {
        let err: ErrorResponse = response.json().await?;
        error!("Signing failed: {}", err.error);
        anyhow::bail!("Signing failed");
    }

    Ok(())
}
