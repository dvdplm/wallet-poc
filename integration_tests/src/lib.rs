#[cfg(test)]
mod tests {
    use anyhow::Result;
    use reqwest::Client;
    use signingcommon::{
        ErrorResponse, ForgetRequest, ForgetResponse, RegisterRequest, RegisterResponse,
        SignRequest, SignResponse,
    };
    use std::process::{Child, Command, Stdio};
    use std::time::Duration;
    use tokio::time::sleep;

    struct TestServer {
        process: Child,
        port: u16,
        client: Client,
    }

    impl TestServer {
        async fn start() -> Result<Self> {
            // Use a random port to avoid conflicts
            let port = 3443; // For now, using the default port

            // Start the server process
            let mut process = Command::new("cargo")
                .args(&["run", "--bin", "signingserver"])
                .current_dir("../signingserver")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            // Create a client that accepts self-signed certificates
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(10))
                .build()?;

            // Wait for the server to be ready
            let server_url = format!("https://127.0.0.1:{}", port);
            for _ in 0..30 {
                if let Ok(_) = client.get(format!("{}/health", server_url)).send().await {
                    println!("Server is ready on port {}", port);
                    return Ok(TestServer {
                        process,
                        port,
                        client,
                    });
                }
                sleep(Duration::from_millis(100)).await;
            }

            // If we get here, server didn't start
            process.kill()?;
            anyhow::bail!("Server failed to start within 3 seconds")
        }

        fn url(&self) -> String {
            format!("https://127.0.0.1:{}", self.port)
        }

        async fn register(&self, seed: &str) -> Result<RegisterResponse> {
            let response = self
                .client
                .post(format!("{}/register", self.url()))
                .json(&RegisterRequest {
                    seed: seed.as_bytes().to_vec(),
                })
                .send()
                .await?;

            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let err: ErrorResponse = response.json().await?;
                anyhow::bail!("Registration failed: {}", err.error)
            }
        }

        async fn sign(&self, user_id: &str, message: &str) -> Result<SignResponse> {
            let response = self
                .client
                .post(format!("{}/sign", self.url()))
                .json(&SignRequest {
                    user_id: user_id.to_string(),
                    message: message.to_string(),
                })
                .send()
                .await?;

            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let err: ErrorResponse = response.json().await?;
                anyhow::bail!("Signing failed: {}", err.error)
            }
        }

        async fn forget(&self, user_id: &str) -> Result<ForgetResponse> {
            let response = self
                .client
                .delete(format!("{}/forget", self.url()))
                .json(&ForgetRequest {
                    user_id: user_id.to_string(),
                })
                .send()
                .await?;

            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                let err: ErrorResponse = response.json().await?;
                anyhow::bail!("Forget failed: {}", err.error)
            }
        }

        async fn health_check(&self) -> Result<String> {
            let response = self
                .client
                .get(format!("{}/health", self.url()))
                .send()
                .await?;

            Ok(response.text().await?)
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            // Kill the server process when the test ends
            let _ = self.process.kill();
        }
    }

    #[tokio::test]
    async fn test_health_check() -> Result<()> {
        let server = TestServer::start().await?;
        let health = server.health_check().await?;
        assert_eq!(health, "OK");
        Ok(())
    }

    #[tokio::test]
    async fn test_full_registration_signing_flow() -> Result<()> {
        let server = TestServer::start().await?;

        // Register a new user
        let seed = "test-seed-12345";
        let reg_response = server.register(seed).await?;
        assert!(!reg_response.user_id.is_empty());
        assert!(!reg_response.verifying_key.is_empty());

        // Sign a message
        let message = "Hello, World!";
        let sign_response = server.sign(&reg_response.user_id, message).await?;
        assert!(!sign_response.signature.is_empty());

        // Sign another message with the same user
        let message2 = "Another message";
        let sign_response2 = server.sign(&reg_response.user_id, message2).await?;
        assert!(!sign_response2.signature.is_empty());

        // Signatures should be different for different messages
        assert_ne!(sign_response.signature, sign_response2.signature);

        Ok(())
    }

    #[tokio::test]
    async fn test_forget_user() -> Result<()> {
        let server = TestServer::start().await?;

        // Register a user
        let seed = "forget-test-seed";
        let reg_response = server.register(seed).await?;
        let user_id = reg_response.user_id.clone();

        // Verify we can sign
        let message = "Test message";
        let sign_response = server.sign(&user_id, message).await?;
        assert!(!sign_response.signature.is_empty());

        // Forget the user
        let forget_response = server.forget(&user_id).await?;
        assert_eq!(forget_response.message, "User successfully forgotten");

        // Try to sign again - should fail
        let sign_result = server.sign(&user_id, message).await;
        assert!(sign_result.is_err());
        if let Err(e) = sign_result {
            assert!(e.to_string().contains("Signing failed"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_sign_with_nonexistent_user() -> Result<()> {
        let server = TestServer::start().await?;

        // Try to sign with a non-existent user ID
        let fake_uuid = "12345678-1234-1234-1234-123456789abc";
        let result = server.sign(fake_uuid, "test message").await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Signing failed"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_users() -> Result<()> {
        let server = TestServer::start().await?;

        // Register multiple users
        let user1 = server.register("user1-seed").await?;
        let user2 = server.register("user2-seed").await?;
        let user3 = server.register("user3-seed").await?;

        // Each should have unique IDs
        assert_ne!(user1.user_id, user2.user_id);
        assert_ne!(user2.user_id, user3.user_id);
        assert_ne!(user1.user_id, user3.user_id);

        // Each should have unique verifying keys (different seeds)
        assert_ne!(user1.verifying_key, user2.verifying_key);
        assert_ne!(user2.verifying_key, user3.verifying_key);
        assert_ne!(user1.verifying_key, user3.verifying_key);

        // All users should be able to sign
        let message = "Common message";
        let sig1 = server.sign(&user1.user_id, message).await?;
        let sig2 = server.sign(&user2.user_id, message).await?;
        let sig3 = server.sign(&user3.user_id, message).await?;

        // Signatures should be different (different keys)
        assert_ne!(sig1.signature, sig2.signature);
        assert_ne!(sig2.signature, sig3.signature);
        assert_ne!(sig1.signature, sig3.signature);

        Ok(())
    }

    #[tokio::test]
    async fn test_same_seed_different_registrations() -> Result<()> {
        let server = TestServer::start().await?;

        let seed = "duplicate-seed-test";

        // Register with the same seed twice
        let reg1 = server.register(seed).await?;
        let reg2 = server.register(seed).await?;

        assert_ne!(reg1.user_id, reg2.user_id);

        // But same verifying key (deterministic from seed)
        assert_eq!(reg1.verifying_key, reg2.verifying_key);

        // Both users should be able to sign
        let message = "Test message";
        let sig1 = server.sign(&reg1.user_id, message).await?;
        let sig2 = server.sign(&reg2.user_id, message).await?;

        // Signatures should be the same (same key, same message)
        assert_eq!(sig1.signature, sig2.signature);

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_message_signing() -> Result<()> {
        let server = TestServer::start().await?;

        let reg = server.register("empty-msg-test").await?;

        // Sign an empty message
        let sig = server.sign(&reg.user_id, "").await?;
        assert!(!sig.signature.is_empty());

        // Empty message should produce a different signature than non-empty
        let sig2 = server.sign(&reg.user_id, "not empty").await?;
        assert_ne!(sig.signature, sig2.signature);

        Ok(())
    }

    #[tokio::test]
    async fn test_large_message_signing() -> Result<()> {
        let server = TestServer::start().await?;

        let reg = server.register("large-msg-test").await?;

        // Create a large message (1MB)
        let large_message = "A".repeat(1_000_000);
        let sig = server.sign(&reg.user_id, &large_message).await?;
        assert!(!sig.signature.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_forget_nonexistent_user() -> Result<()> {
        let server = TestServer::start().await?;

        // Forgetting a non-existent user should succeed (idempotent)
        let fake_uuid = "87654321-4321-4321-4321-210987654321";
        let result = server.forget(fake_uuid).await?;
        assert_eq!(result.message, "User successfully forgotten");

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_uuid_handling() -> Result<()> {
        let server = TestServer::start().await?;

        // Try to sign with an invalid UUID format
        let invalid_uuid = "not-a-uuid";
        let result = server.sign(invalid_uuid, "test").await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_signature_consistency() -> Result<()> {
        let server = TestServer::start().await?;

        let reg = server.register("consistency-test").await?;
        let message = "Consistent message";

        // Sign the same message multiple times
        let sig1 = server.sign(&reg.user_id, message).await?;
        let sig2 = server.sign(&reg.user_id, message).await?;
        let sig3 = server.sign(&reg.user_id, message).await?;

        // All signatures should be identical (deterministic signing)
        assert_eq!(sig1.signature, sig2.signature);
        assert_eq!(sig2.signature, sig3.signature);

        Ok(())
    }
}
