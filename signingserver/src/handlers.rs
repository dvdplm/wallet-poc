use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::state::AppState;
use signingcommon::{
    ErrorResponse, ForgetRequest, ForgetResponse, RegisterRequest, RegisterResponse, SignRequest,
    SignResponse,
};

/// Register a new user and generate a signing key
pub async fn register(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    debug!("Register request for user: {:?}", req.seed);

    let mut state = state.write().await;
    let user = state.register_user(&req.seed);
    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            verifying_key: hex::encode(user.signing_key.verifying_key().as_bytes()),
        }),
    )
        .into_response()
}

/// Sign a message for a user
pub async fn sign(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    info!("Sign request for user: {}", req.user_id);

    let state = state.read().await;

    match state.sign_message(&req.user_id, &req.message) {
        Ok(signature) => {
            info!("Message signed successfully for user: {}", req.user_id);
            (
                StatusCode::OK,
                Json(SignResponse {
                    signature: hex::encode(&signature),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Signing failed: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Signing failed: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Delete a user (forget)
pub async fn forget(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<ForgetRequest>,
) -> impl IntoResponse {
    info!("Forget user: {}", req.user_id);

    let mut state = state.write().await;

    match state.delete_user(&req.user_id) {
        Ok(_) => {
            info!("User forgotten: {}", req.user_id);
            (
                StatusCode::OK,
                Json(ForgetResponse {
                    message: "User data deleted successfully".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Forget failed: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Forget failed: {}", e),
                }),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register() {
        // Setup state and request params
        let app_state = Arc::new(RwLock::new(AppState::new()));
        let req = RegisterRequest {
            seed: vec![1, 2, 3, 4, 5],
        };

        // Call the register handler
        let response = register(State(app_state), Json(req)).await.into_response();

        // Assert the success - verify status code is CREATED (201)
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
