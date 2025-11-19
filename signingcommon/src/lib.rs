use serde::{Deserialize, Serialize};

/// Request to register a new user and generate a signing key
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterRequest {
    pub seed: Vec<u8>,
}

/// Response after successful registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub verifying_key: String,
}

/// Request to sign a message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignRequest {
    pub user_id: String,
    pub message: String,
}

/// Response with the signature
#[derive(Debug, Serialize, Deserialize)]
pub struct SignResponse {
    pub signature: String,
}

/// Request to forget a user
#[derive(Debug, Serialize, Deserialize)]
pub struct ForgetRequest {
    pub user_id: String,
}

/// Response after forgetting a user
#[derive(Debug, Serialize, Deserialize)]
pub struct ForgetResponse {
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
