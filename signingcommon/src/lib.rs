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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_serialization() {
        let req = RegisterRequest {
            seed: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"seed\""));
    }

    #[test]
    fn test_register_request_deserialization() {
        let json = r#"{"seed":[1,2,3]}"#;
        let req: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.seed, vec![1, 2, 3]);
    }

    #[test]
    fn test_register_response_serialization() {
        let resp = RegisterResponse {
            user_id: "123".to_string(),
            verifying_key: "abc".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"user_id\":\"123\""));
        assert!(json.contains("\"verifying_key\":\"abc\""));
    }

    #[test]
    fn test_register_response_deserialization() {
        let json = r#"{"user_id":"456","verifying_key":"xyz"}"#;
        let resp: RegisterResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.user_id, "456");
        assert_eq!(resp.verifying_key, "xyz");
    }

    #[test]
    fn test_sign_request_serialization() {
        let req = SignRequest {
            user_id: "user1".to_string(),
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"user_id\":\"user1\""));
        assert!(json.contains("\"message\":\"hello\""));
    }

    #[test]
    fn test_sign_request_deserialization() {
        let json = r#"{"user_id":"user2","message":"world"}"#;
        let req: SignRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "user2");
        assert_eq!(req.message, "world");
    }

    #[test]
    fn test_sign_response_serialization() {
        let resp = SignResponse {
            signature: "sig123".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"signature":"sig123"}"#);
    }

    #[test]
    fn test_sign_response_deserialization() {
        let json = r#"{"signature":"sig456"}"#;
        let resp: SignResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.signature, "sig456");
    }

    #[test]
    fn test_forget_request_serialization() {
        let req = ForgetRequest {
            user_id: "user3".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"user_id":"user3"}"#);
    }

    #[test]
    fn test_forget_request_deserialization() {
        let json = r#"{"user_id":"user4"}"#;
        let req: ForgetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "user4");
    }

    #[test]
    fn test_forget_response_serialization() {
        let resp = ForgetResponse {
            message: "deleted".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"message":"deleted"}"#);
    }

    #[test]
    fn test_forget_response_deserialization() {
        let json = r#"{"message":"forgotten"}"#;
        let resp: ForgetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.message, "forgotten");
    }

    #[test]
    fn test_error_response_serialization() {
        let resp = ErrorResponse {
            error: "something went wrong".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"error":"something went wrong"}"#);
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"error":"user not found"}"#;
        let resp: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.error, "user not found");
    }

    #[test]
    fn test_register_request_clone() {
        let req1 = RegisterRequest {
            seed: vec![1, 2, 3],
        };
        let req2 = req1.clone();
        assert_eq!(req1.seed, req2.seed);
    }

    #[test]
    fn test_sign_request_clone() {
        let req1 = SignRequest {
            user_id: "id".to_string(),
            message: "msg".to_string(),
        };
        let req2 = req1.clone();
        assert_eq!(req1.user_id, req2.user_id);
        assert_eq!(req1.message, req2.message);
    }

    #[test]
    fn test_register_request_debug() {
        let req = RegisterRequest {
            seed: vec![1, 2, 3],
        };
        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("RegisterRequest"));
    }

    #[test]
    fn test_sign_request_debug() {
        let req = SignRequest {
            user_id: "user".to_string(),
            message: "msg".to_string(),
        };
        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("SignRequest"));
    }

    #[test]
    fn test_error_response_debug() {
        let resp = ErrorResponse {
            error: "test error".to_string(),
        };
        let debug_str = format!("{:?}", resp);
        assert!(debug_str.contains("ErrorResponse"));
        assert!(debug_str.contains("test error"));
    }
}
