use rand::Rng;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    get_postgres_pool, get_redis_client,
    utils::constants::{DATABASE_URL, REDIS_HOST_NAME},
};

use super::{email::Email, password::Password, user::User};
use std::collections::HashSet;

pub fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

pub async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL).await.expect(
        format!(
            "Failed to create Postgres connection pool! Target url: {}",
            DATABASE_URL.to_string()
        )
        .as_str(),
    );
    println!("Created pool with: {}", DATABASE_URL.to_string());

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn insert(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn token_exists(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenAlreadyExists,
    TokenNotFound,
    UnexpectedError,
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

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    /// Insert into the store
    /// ```
    /// use tokio_test;
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// tokio_test::block_on(async {
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = "asduashfiasbnfd".to_string();
    /// let result = store.insert(sample_token.clone()).await;
    /// assert!(result.is_ok());
    /// });
    /// ```
    async fn insert(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.store.insert(token);
        Ok(())
    }

    /// Check if token exists
    /// ```
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// use tokio::test;
    /// tokio_test::block_on(async {
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = "asduashfiasbnfd".to_string();
    /// let result = store.insert(sample_token.clone()).await;
    /// assert_eq!(result.is_ok(), true);
    /// assert_eq!(store.token_exists(&sample_token).await, Ok(true));
    /// });
    /// ```
    async fn token_exists(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.store.contains(token))
    }
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Unexpected Error")]
    UnexpectedError,
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError, Self::UnexpectedError)
        )
    }
}

// This trait represents the interface all concrete 2FA code stores should implement
#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    EmailAlreadyExists,
    EmailDoesNotExist,
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeError {
    InvalidUuid,
    CodeOutOfRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    /// Parse uid, which should be a valid UUID
    /// ```
    /// use auth_service::domain::data_stores::{LoginAttemptId, TwoFACodeError};
    /// let bad_login_attempt_id = LoginAttemptId::parse("should_not_work".to_string());
    /// assert!(bad_login_attempt_id.is_err());
    /// if let Err(e) = bad_login_attempt_id {
    ///     assert_eq!(e, TwoFACodeError::InvalidUuid);
    /// }
    /// ```
    pub fn parse(id: String) -> Result<Self, TwoFACodeError> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        let _ = Uuid::parse_str(id.as_str()).map_err(|_| TwoFACodeError::InvalidUuid)?;
        Ok(Self(id))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        // Not sure if this is safe, since we are relying on uuid
        // implementation guaranteeing that parse_str will always
        // be able to parse without throwing an error
        Self::parse(uuid::Uuid::new_v4().to_string()).unwrap()
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    /// Parse a 2fa code
    /// ```
    /// use crate::auth_service::domain::data_stores::{TwoFACode, TwoFACodeError};
    /// let num = 91234;
    /// let valid_two_fa = TwoFACode::parse(num.to_string()).unwrap();
    /// assert_eq!(valid_two_fa.as_ref().parse::<u32>().unwrap(), num);
    ///
    /// // This should fail: passing in a non number
    /// let invalid = TwoFACode::parse("I am not a number".to_string());
    /// if let Err(e) = invalid {
    ///     assert_eq!(e, TwoFACodeError::InvalidUuid);
    /// }
    /// ```
    pub fn parse(code: String) -> Result<Self, TwoFACodeError> {
        let code_as_int = code
            .parse::<u32>()
            .map_err(|_| TwoFACodeError::InvalidUuid)?;
        if code_as_int > 999999 {
            Err(TwoFACodeError::CodeOutOfRange)
        } else {
            Ok(Self(code))
        }
    }
}

impl Default for TwoFACode {
    /// Use the `rand` crate to generate a random 2FA code.
    /// The code should be 6 digits (ex: 834629)
    /// ```
    /// use crate::auth_service::domain::data_stores::{TwoFACode};
    /// let default_val = TwoFACode::default();
    /// let val = default_val.as_ref();
    /// let char_count = val.chars().count();
    /// let is_between_0_and_6 = char_count >= 0 && char_count < 7;
    /// assert!(is_between_0_and_6);
    /// ```
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        // Maybe it is not a good idea to hard-code these numbers here.
        // Makes writing tests difficult
        let number: u32 = rng.gen_range(0..999999);
        Self::parse(format!("{}", number)).unwrap()
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
