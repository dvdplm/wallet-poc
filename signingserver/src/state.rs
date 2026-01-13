use anyhow::anyhow;
use ed25519_dalek::{SECRET_KEY_LENGTH, Signature, Signer, SigningKey, VerifyingKey};
use heapless::index_map::FnvIndexMap;
use hkdf::Hkdf;
use sha2::Sha256;
use uuid::Uuid;

const MAX_KEYS: usize = 1_024;

// The master key is used to salt user key derivation. This should be carefully guarded.
const MASTER_KEY: &[u8] = b"s!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!ts!kr!t";

/// Application state managing keys
#[derive(Debug)]
pub struct AppState {
    // TODO: Should probably instantiate this with SIP rather than FNV. `heapless` does not provide
    // a builtin alias something like this should work:
    // 	`pub type FnvIndexMap<K, V, const N: usize> = IndexMap<K, V, BuildHasherDefault<SipHasher>, N>;`
    keys: FnvIndexMap<Uuid, SigningKey, MAX_KEYS>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            keys: FnvIndexMap::new(),
        }
    }

    /// Register a new user with a deterministically derived signing key
    pub fn register_user(&mut self, seed: &[u8]) -> anyhow::Result<(Uuid, VerifyingKey)> {
        // Derive a signing key from seed + master key using HKDF.
        let hkdf = Hkdf::<Sha256>::new(Some(MASTER_KEY), seed);
        let mut signing_key_bytes = [0u8; SECRET_KEY_LENGTH];
        hkdf.expand(b"signing_key", &mut signing_key_bytes)
            .expect("okm has valid and hardcoded length");
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let user_id = Uuid::new_v4();

        self.keys
            .insert(user_id.clone(), signing_key)
            .map_err(|_| anyhow!("Server is at capacity. Sorry."))?;
        Ok((user_id, verifying_key))
    }

    // Get a user by UUID
    fn user(&self, user_id: &str) -> anyhow::Result<SigningKey> {
        let user_id = Uuid::parse_str(user_id)?;
        self.keys
            .get(&user_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No such user"))
    }

    /// Sign a message for a user
    pub fn sign_message(&self, user_id: &str, message: &str) -> anyhow::Result<Signature> {
        let signing_key = self.user(user_id)?;

        let signature = signing_key.sign(message.as_bytes());

        Ok(signature)
    }

    /// Delete a user (forget)
    pub fn forget(&mut self, user_id: &str) {
        let user_id = Uuid::parse_str(user_id).unwrap_or_else(|_| Uuid::new_v4());
        self.keys.remove(&user_id);
    }
}
