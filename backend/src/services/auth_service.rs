use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::models::auth::{
    AuthenticatedUser, JwtClaims, LoginRequest, LoginResponse, TokenPair, TokenType, UserRole,
};
use crate::utils::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct AuthService {
    config: AppConfig,
    refresh_tokens: Arc<RwLock<HashMap<Uuid, RefreshTokenRecord>>>,
    seed_user: SeedUser,
}

#[derive(Clone)]
struct SeedUser {
    user_id: Uuid,
    email: String,
    password: String,
    role: UserRole,
}

#[derive(Clone)]
struct RefreshTokenRecord {
    user_id: Uuid,
    expires_at: DateTime<Utc>,
}

impl AuthService {
    pub fn new(config: AppConfig) -> Self {
        let seed_user = SeedUser {
            user_id: Uuid::from_u128(1),
            email: config.demo_admin_email.clone(),
            password: config.demo_admin_password.clone(),
            role: UserRole::Admin,
        };

        Self {
            config,
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            seed_user,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        let payload = request.validate()?;

        if payload.email != self.seed_user.email || payload.password != self.seed_user.password {
            return Err(AppError::Unauthorized("Invalid credentials".to_owned()));
        }

        let tokens = self.issue_token_pair(&self.seed_user).await?;

        Ok(LoginResponse {
            tokens,
            user: AuthenticatedUser {
                user_id: self.seed_user.user_id,
                email: self.seed_user.email.clone(),
                role: self.seed_user.role.clone(),
            },
        })
    }

    pub async fn refresh(&self, refresh_token: String) -> AppResult<TokenPair> {
        let claims = self.decode_token(
            &refresh_token,
            &self.config.jwt_refresh_secret,
            TokenType::Refresh,
        )?;

        let mut refresh_tokens = self.refresh_tokens.write().await;
        let existing = refresh_tokens
            .remove(&claims.jti)
            .ok_or_else(|| AppError::Unauthorized("Refresh token is no longer valid".to_owned()))?;

        if existing.expires_at < Utc::now() || existing.user_id != claims.sub {
            return Err(AppError::Unauthorized("Refresh token has expired".to_owned()));
        }

        drop(refresh_tokens);
        self.issue_token_pair(&self.seed_user).await
    }

    pub fn validate_access_token(&self, token: &str) -> AppResult<AuthenticatedUser> {
        let claims = self.decode_token(token, &self.config.jwt_access_secret, TokenType::Access)?;
        Ok(AuthenticatedUser::from(&claims))
    }

    async fn issue_token_pair(&self, user: &SeedUser) -> AppResult<TokenPair> {
        let now = Utc::now().timestamp();

        let access_claims = JwtClaims {
            sub: user.user_id,
            email: user.email.clone(),
            role: user.role.clone(),
            token_type: TokenType::Access,
            exp: (now + self.config.jwt_access_ttl_seconds) as usize,
            iat: now as usize,
            jti: Uuid::new_v4(),
        };

        let refresh_claims = JwtClaims {
            sub: user.user_id,
            email: user.email.clone(),
            role: user.role.clone(),
            token_type: TokenType::Refresh,
            exp: (now + self.config.jwt_refresh_ttl_seconds) as usize,
            iat: now as usize,
            jti: Uuid::new_v4(),
        };

        let access_token = self.encode_token(&access_claims, &self.config.jwt_access_secret)?;
        let refresh_token = self.encode_token(&refresh_claims, &self.config.jwt_refresh_secret)?;

        let refresh_exp = Utc
            .timestamp_opt(refresh_claims.exp as i64, 0)
            .single()
            .ok_or_else(|| AppError::Internal("Invalid refresh token expiry".to_owned()))?;

        self.refresh_tokens.write().await.insert(
            refresh_claims.jti,
            RefreshTokenRecord {
                user_id: user.user_id,
                expires_at: refresh_exp,
            },
        );

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in_seconds: self.config.jwt_access_ttl_seconds,
        })
    }

    fn encode_token(&self, claims: &JwtClaims, secret: &str) -> AppResult<String> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|error| AppError::Internal(format!("Failed to encode token: {error}")))
    }

    fn decode_token(&self, token: &str, secret: &str, expected_type: TokenType) -> AppResult<JwtClaims> {
        let validation = Validation::default();
        let decoded = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AppError::Unauthorized("Invalid token".to_owned()))?;

        if decoded.claims.token_type != expected_type {
            return Err(AppError::Unauthorized("Invalid token type".to_owned()));
        }

        Ok(decoded.claims)
    }
}
