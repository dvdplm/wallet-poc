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

/// Forget a user
pub async fn forget(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<ForgetRequest>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    state.forget(&req.user_id);
    (
        StatusCode::OK,
        Json(ForgetResponse {
            message: "User successfully forgotten".to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register() {
        let app_state = Arc::new(RwLock::new(AppState::new()));
        let req = RegisterRequest {
            seed: vec![1, 2, 3, 4, 5],
        };

        let response = register(State(app_state), Json(req)).await.into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_forget() {
        let app_state = Arc::new(RwLock::new(AppState::new()));

        // Register
        let user_id = {
            let mut state = app_state.write().await;
            let user = state.register_user(&[1, 2, 3, 4, 5]);
            user.id
        };

        // Sign something, check success
        let sign_req = SignRequest {
            user_id: user_id.clone(),
            message: "test message".to_string(),
        };

        let sign_response = sign(State(app_state.clone()), Json(sign_req))
            .await
            .into_response();

        assert_eq!(sign_response.status(), StatusCode::OK);

        // Forget
        let forget_req = ForgetRequest {
            user_id: user_id.clone(),
        };

        let forget_response = forget(State(app_state.clone()), Json(forget_req))
            .await
            .into_response();

        assert_eq!(forget_response.status(), StatusCode::OK);

        // Assert "not found"
        let sign_req_after = SignRequest {
            user_id: user_id.clone(),
            message: "test message after forget".to_string(),
        };

        let sign_response_after = sign(State(app_state), Json(sign_req_after))
            .await
            .into_response();

        assert_eq!(sign_response_after.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_sign_success() {
        let app_state = Arc::new(RwLock::new(AppState::new()));
        let user_id = {
            let mut state = app_state.write().await;
            let user = state.register_user(&[1, 2, 3, 4, 5]);
            user.id
        };

        let sign_req = SignRequest {
            user_id,
            message: "test message".to_string(),
        };

        let response = sign(State(app_state), Json(sign_req)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sign_fail() {
        let app_state = Arc::new(RwLock::new(AppState::new()));

        let sign_req = SignRequest {
            user_id: "non-existent-user".to_string(),
            message: "test message".to_string(),
        };

        let response = sign(State(app_state), Json(sign_req)).await.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
