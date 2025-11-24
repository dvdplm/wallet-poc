use anyhow::Result;
use clap::{Parser, Subcommand};
use signingcommon::{
    ErrorResponse, ForgetRequest, ForgetResponse, RegisterRequest, RegisterResponse, SignRequest,
    SignResponse,
};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "sign")]
#[command(about = "Sign messages using the remote signing service")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The message to sign (used when no subcommand is given)
    #[arg(short, long, requires = "user_id")]
    message: Option<String>,

    /// The user ID for signing (used when no subcommand is given)
    #[arg(short, long, requires = "message")]
    user_id: Option<String>,

    /// The server URL
    #[arg(short, long, default_value = "https://127.0.0.1:3443", global = true)]
    server: String,

    /// Accept self-signed certificates (for development)
    #[arg(long, default_value_t = true, global = true)]
    danger_accept_invalid_certs: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Register a new signing key and get a UUID
    Register {
        /// Seed string for key generation
        seed: String,
    },
    /// Forget a user (delete their signing key)
    Forget {
        /// User ID to forget
        #[arg(short, long)]
        user_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    // Build client with TLS configuration
    let client = if args.danger_accept_invalid_certs {
        info!("Warning: Accepting self-signed certificates");
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?
    } else {
        reqwest::Client::new()
    };

    match args.command {
        Some(Commands::Register { seed }) => {
            register_user(&client, &args.server, &seed).await?;
        }
        Some(Commands::Forget { user_id }) => {
            forget_user(&client, &args.server, &user_id).await?;
        }
        None => {
            // Handle the default sign operation when no subcommand is given
            let user_id = args
                .user_id
                .ok_or_else(|| anyhow::anyhow!("User ID required (-u flag)"))?;
            let message = args
                .message
                .ok_or_else(|| anyhow::anyhow!("Message required (-m flag)"))?;

            sign_message(&client, &args.server, &user_id, &message).await?;
        }
    }

    Ok(())
}

async fn register_user(client: &reqwest::Client, server_url: &str, seed: &str) -> Result<()> {
    info!("Registering new user...");

    // Convert seed string to bytes
    let seed = seed.as_bytes().to_vec();

    let response = client
        .post(format!("{}/register", server_url))
        .json(&RegisterRequest { seed })
        .send()
        .await?;

    if response.status().is_success() {
        let result: RegisterResponse = response.json().await?;
        println!("{}", result.user_id);
        println!("{}", result.verifying_key);
        info!(
            "User registered successfully.\n UUID:\t{}\n Verifying key:\t{}",
            result.user_id, result.verifying_key
        );
    } else {
        let err: ErrorResponse = response.json().await?;
        error!("Registration failed: {}", err.error);
        anyhow::bail!("Registration failed: {}", err.error);
    }

    Ok(())
}

async fn sign_message(
    client: &reqwest::Client,
    server_url: &str,
    user_id: &str,
    message: &str,
) -> Result<()> {
    info!("Signing message...");

    let response = client
        .post(format!("{}/sign", server_url))
        .json(&SignRequest {
            user_id: user_id.to_string(),
            message: message.to_string(),
        })
        .send()
        .await?;

    if response.status().is_success() {
        let result: SignResponse = response.json().await?;
        println!("{}", result.signature);
        info!("Message signed successfully");
    } else if response.status() == 404 {
        error!("User not found. Please register first using 'sign register'");
        anyhow::bail!("User not found");
    } else {
        let err: ErrorResponse = response.json().await?;
        error!("Signing failed: {}", err.error);
        anyhow::bail!("Signing failed: {}", err.error);
    }

    Ok(())
}

async fn forget_user(client: &reqwest::Client, server_url: &str, user_id: &str) -> Result<()> {
    info!("Forgetting user {}...", user_id);

    let response = client
        .delete(format!("{}/forget", server_url))
        .json(&ForgetRequest {
            user_id: user_id.to_string(),
        })
        .send()
        .await?;

    if response.status().is_success() {
        let result: ForgetResponse = response.json().await?;
        println!("{}", result.message);
        info!("User {} forgotten successfully", user_id);
    } else {
        let err: ErrorResponse = response.json().await?;
        error!("Forget failed: {}", err.error);
        anyhow::bail!("Forget failed: {}", err.error);
    }

    Ok(())
}
