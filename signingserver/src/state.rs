use ed25519_dalek::{SECRET_KEY_LENGTH, Signer, SigningKey};
use rand::RngCore;
use std::collections::HashMap;
use uuid::Uuid;

const MAX_USERS: usize = 1_000;

/// Represents a user in the system
#[derive(Clone, Debug)]
pub struct User {
    pub id: String, //FIXME: want more type safety here. And length restrictions. Maybe just use random bytes?
    pub signing_key: SigningKey,
}

/// Application state managing all users and their keys
#[derive(Debug)]
pub struct AppState {
    users: HashMap<String, User>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            // FIXME: replace with non-allocating data structure?
            users: HashMap::with_capacity(MAX_USERS),
        }
    }

    /// Register a new user with a generated signing key
    pub fn register_user(&mut self, seed: &[u8]) -> anyhow::Result<User> {
        // Generate a new ED25519 key pair
        let mut secret_key_bytes = [0u8; SECRET_KEY_LENGTH];
        // FIXME: we want to control which CSPRNG we use here. Not safe to use OS defaults.
        rand::thread_rng().fill_bytes(&mut secret_key_bytes);
        // TODO: derive key from seed + masterkey
        let secret_key = SigningKey::from_bytes(&secret_key_bytes);

        let user_id = Uuid::new_v4().to_string();
        let user = User {
            id: user_id.clone(),
            signing_key: secret_key,
        };

        self.users.insert(user_id, user.clone());
        Ok(user)
    }

    /// Get a user by ID
    fn user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).cloned()
    }

    /// Sign a message for a user
    pub fn sign_message(&self, user_id: &str, message: &str) -> anyhow::Result<Vec<u8>> {
        let user = self
            .user(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let signature = user.signing_key.sign(message.as_bytes());

        // FIXME: we probably don't need to allocate here?
        Ok(signature.to_bytes().to_vec())
    }

    /// Delete a user (forget)
    pub fn delete_user(&mut self, user_id: &str) -> anyhow::Result<()> {
        self.users
            .remove(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        Ok(())
    }
}
