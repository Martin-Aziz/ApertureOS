use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::{AppError, AppResult};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub email: String,
    pub role: UserRole,
    pub token_type: TokenType,
    pub exp: usize,
    pub iat: usize,
    pub jti: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
}

impl From<&JwtClaims> for AuthenticatedUser {
    fn from(value: &JwtClaims) -> Self {
        Self {
            user_id: value.sub,
            email: value.email.clone(),
            role: value.role.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(self) -> AppResult<Self> {
        let email = self.email.trim().to_lowercase();
        let password = self.password;

        if email.is_empty() {
            return Err(AppError::BadRequest("Email is required".to_owned()));
        }

        if password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_owned(),
            ));
        }

        Ok(Self { email, password })
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl RefreshRequest {
    pub fn validate(self) -> AppResult<Self> {
        if self.refresh_token.trim().is_empty() {
            return Err(AppError::BadRequest("Refresh token is required".to_owned()));
        }

        Ok(self)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_seconds: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginResponse {
    pub tokens: TokenPair,
    pub user: AuthenticatedUser,
}
