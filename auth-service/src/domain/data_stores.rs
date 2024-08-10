use super::{email::Email, password::Password, user::User};
use std::collections::HashSet;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    fn insert(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    fn token_exists(&self, token: &str) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenAlreadyExists,
}

pub struct HashsetBannedTokenStore {
    store: HashSet<String>,
}

impl HashsetBannedTokenStore {
    pub fn new() -> Self {
        Self {
            store: HashSet::new(),
        }
    }
}

impl BannedTokenStore for HashsetBannedTokenStore {
    /// Insert into the store
    /// ```
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = "asduashfiasbnfd".to_string();
    /// let result = store.insert(sample_token.clone());
    /// assert_eq!(result.is_ok(), true);
    /// let failed_result = store.insert(sample_token.clone());
    /// assert_eq!(failed_result.is_err(), true);
    /// ```
    fn insert(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.store.insert(token) {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenAlreadyExists)
        }
    }

    /// Check if token exists
    /// ```
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = "asduashfiasbnfd".to_string();
    /// let result = store.insert(sample_token.clone());
    /// assert_eq!(result.is_ok(), true);
    /// assert_eq!(store.token_exists(&sample_token), true);
    /// ```
    fn token_exists(&self, token: &str) -> bool {
        self.store.contains(token)
    }
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    InvalidPassword,
    UnexpectedError,
}
