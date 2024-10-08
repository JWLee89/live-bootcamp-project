use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument("Insert banned token", skip_all)]
    async fn insert(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(token.expose_secret());
        // 2. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        // The value should simply be a `true` (boolean value).
        // The expiration time should be set to TOKEN_TTL_SECONDS.
        // NOTE: The TTL is expected to be a u64 so you will have to cast TOKEN_TTL_SECONDS to a u64.
        // Return BannedTokenStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let value = true;

        // Convert to u64
        let time_to_live: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        let _: () = self
            .conn
            .write()
            .await
            .set_ex(key, value, time_to_live)
            .map_err(|_| BannedTokenStoreError::TokenAlreadyExists)?;
        Ok(())
    }

    #[tracing::instrument("Check if banned token exists", skip_all)]
    async fn token_exists(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        // Check if the token exists by calling the exists method on the Redis connection
        let key = get_key(&token.expose_secret());
        let is_banned: bool = self
            .conn
            .write()
            .await
            .exists(&key)
            .wrap_err("failed to check if token exists in Redis")
            .map_err(|_| BannedTokenStoreError::TokenNotFound)?;
        Ok(is_banned)
    }
}
