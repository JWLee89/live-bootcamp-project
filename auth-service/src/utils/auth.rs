use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{app_state::state::BannedTokenStoreType, domain::email::Email};

use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};

// Create cookie with a new JWT auth token
#[tracing::instrument("Generate authentication cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create cookie and set the value to the passed-in token string
#[tracing::instrument("Create authentication cookie", skip_all)]
fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .path("/") // apple cookie to all URLs on the server
        .http_only(true) // prevent JavaScript from accessing the cookie
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .build();

    cookie
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}

// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

// Create JWT auth token
#[tracing::instrument("Generate authentication token", skip_all)]
fn generate_auth_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("failed to create 10 minute time delta")?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("failed to add 10 minutes to current time"))?
        .timestamp();

    // Cast exp to a usize, which is what Claims expects
    let exp: usize = exp.try_into().wrap_err(format!(
        "failed to cast exp time to usize. exp time: {}",
        exp
    ))?;

    let sub = email.as_ref().expose_secret().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims)
}

// Check if JWT auth token is valid by decoding it using the JWT secret
#[tracing::instrument("Validate authentication token", skip_all)]
pub async fn validate_token(
    token: &Secret<String>,
    banned_token_store: BannedTokenStoreType,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    // If exists in banned token store
    match banned_token_store.read().await.token_exists(token).await {
        Ok(value) => {
            if value {
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidToken,
                ));
            }
        }
        Err(_) => {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }
    }

    decode::<Claims>(
        token.expose_secret(),
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

// Create JWT auth token by encoding claims using the JWT secret
fn create_token(claims: &Claims) -> Result<String> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .wrap_err("Failed to create token")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use secrecy::Secret;
    use tokio::sync::RwLock;

    use crate::domain::{data_stores::HashsetBannedTokenStore, parse::Parseable};

    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let token = Secret::new(generate_auth_token(&email).unwrap());
        let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));

        // Validate
        let result = validate_token(&token, banned_token_store).await.unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = Secret::new("invalid_token".to_owned());
        let banned_token_store: Arc<RwLock<HashsetBannedTokenStore>> =
            Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
        let result = validate_token(&token, banned_token_store).await;
        assert!(result.is_err());
    }
}
