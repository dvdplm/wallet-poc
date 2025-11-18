use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, Signer, SigningKey};
use rand::RngCore;
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

/// Represents a user in the system
#[derive(Clone, Debug)]
pub struct User {
    pub id: String, //FIXME: want more type safety here. And length restrictions. Maybe just use random bytes?
    #[allow(dead_code)]
    pub username: String, // FIXME: should be a salted hash. Or not be here at all.
    pub secret_key: [u8; SECRET_KEY_LENGTH],
    pub public_key: [u8; PUBLIC_KEY_LENGTH],
}

/// Application state managing all users and their keys
#[derive(Debug)]
pub struct AppState {
    users: HashMap<String, User>,
}

impl AppState {
    pub fn new() -> Self {
        debug!("Creating new AppState");
        AppState {
            users: HashMap::new(),
        }
    }

    /// Register a new user with a generated signing key
    pub fn register_user(&mut self, username: String) -> anyhow::Result<User> {
        debug!("Registering new user: {}", username);

        // Generate a new ED25519 key pair
        let mut secret_key_bytes = [0u8; SECRET_KEY_LENGTH];
        // FIXME: we want to control which CSPRNG we use here. Not safe to use OS defaults.
        rand::thread_rng().fill_bytes(&mut secret_key_bytes);
        let secret_key = SigningKey::from_bytes(&secret_key_bytes);
        let public_key = secret_key.verifying_key();

        let user_id = Uuid::new_v4().to_string();
        let user = User {
            id: user_id.clone(),
            username,
            secret_key: secret_key.to_bytes(),
            public_key: public_key.to_bytes(),
        };

        debug!("User created with ID: {}", user_id);
        self.users.insert(user_id, user.clone());
        Ok(user)
    }

    /// Get a user by ID
    fn user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).cloned()
    }

    /// Sign a message for a user
    pub fn sign_message(&self, user_id: &str, message: &str) -> anyhow::Result<Vec<u8>> {
        debug!("Signing message for user: {}", user_id);

        let user = self
            .user(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let secret_key = SigningKey::from_bytes(&user.secret_key);

        let signature = secret_key.sign(message.as_bytes());

        debug!("Message signed successfully for user: {}", user_id);
        // FIXME: we probably don't need to allocate here?
        Ok(signature.to_bytes().to_vec())
    }

    /// Delete a user (forget)
    pub fn delete_user(&mut self, user_id: &str) -> anyhow::Result<()> {
        debug!("Deleting user: {}", user_id);

        self.users
            .remove(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        debug!("User deleted successfully: {}", user_id);
        Ok(())
    }
}
