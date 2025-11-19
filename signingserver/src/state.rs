use ed25519_dalek::{SECRET_KEY_LENGTH, Signer, SigningKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

const MAX_USERS: usize = 1_000;
const MASTER_KEY: &[u8] = b"s!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!t";

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

    /// Register a new user with a deterministically derived signing key
    pub fn register_user(&mut self, seed: &[u8]) -> User {
        // Derive a signing key from seed + master key using HKDF
        let hkdf = Hkdf::<Sha256>::new(Some(MASTER_KEY), seed);
        let mut signing_key_bytes = [0u8; SECRET_KEY_LENGTH];
        hkdf.expand(b"signing_key", &mut signing_key_bytes)
            .expect("okm has the correct length");
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);

        let user_id = Uuid::new_v4().to_string();
        let user = User {
            id: user_id.clone(),
            signing_key,
        };

        self.users.insert(user_id, user.clone());
        user
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
    pub fn forget(&mut self, user_id: &str) {
        self.users.remove(user_id);
    }
}
