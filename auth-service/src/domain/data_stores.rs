use color_eyre::eyre::Result;
use color_eyre::eyre::{eyre, Context, Report};
use rand::Rng;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::services::postmark_email_client::PostmarkEmailClient;
use crate::utils::constants::prod::email_client::SENDER;
use crate::utils::constants::{prod, POSTMARK_AUTH_TOKEN};
use crate::{
    get_postgres_pool, get_redis_client,
    utils::constants::{DATABASE_URL, REDIS_HOST_NAME},
};

use super::parse::Parseable;
use super::{email::Email, password::Password, user::User};
use std::collections::HashSet;

pub fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

// New!
pub fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(Secret::new(SENDER.to_owned())).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
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
    async fn insert(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError>;
    async fn token_exists(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Token already exists")]
    TokenAlreadyExists,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
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
    /// use secrecy::Secret;
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// tokio_test::block_on(async {
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = "asduashfiasbnfd".to_string();
    /// let result = store.insert(Secret::new(sample_token.clone())).await;
    /// assert!(result.is_ok());
    /// });
    /// ```
    async fn insert(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        self.store.insert(token.expose_secret().to_owned());
        Ok(())
    }

    /// Check if token exists
    /// ```
    /// use secrecy::Secret;
    /// use crate::auth_service::domain::data_stores::{BannedTokenStore, HashsetBannedTokenStore};
    /// use tokio::test;
    /// tokio_test::block_on(async {
    /// let mut store = HashsetBannedTokenStore::new();
    /// let sample_token = Secret::new("asduashfiasbnfd".to_string());
    /// let result = store.insert(sample_token.clone()).await;
    /// assert_eq!(result.is_ok(), true);
    /// assert!(store.token_exists(&sample_token).await.expect("Token should exist after inserting"));
    /// });
    /// ```
    async fn token_exists(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        Ok(self.store.contains(token.expose_secret()))
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
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::InvalidPassword, Self::InvalidPassword)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
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

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("Email adread exists")]
    EmailAlreadyExists,
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::EmailAlreadyExists, Self::EmailAlreadyExists)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Error)]
pub enum TwoFACodeError {
    #[error("Invalid UUID")]
    InvalidUuid,
    #[error("Code out of range")]
    CodeOutOfRange,
}

impl PartialEq for TwoFACodeError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::InvalidUuid, Self::InvalidUuid) | (Self::CodeOutOfRange, Self::CodeOutOfRange)
        )
    }
}

#[derive(Debug, Clone)]
pub struct LoginAttemptId(Secret<String>);

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl LoginAttemptId {
    /// Parse uid, which should be a valid UUID
    /// ```
    /// use auth_service::domain::data_stores::{LoginAttemptId, TwoFACodeError};
    /// let bad_login_attempt_id = LoginAttemptId::parse("should_not_work".to_string());
    /// assert!(bad_login_attempt_id.is_err());
    /// ```
    pub fn parse(id: String) -> Result<Self> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        let parsed_id = Uuid::parse_str(id.as_str()).wrap_err("Invalid login attempt id")?;
        Ok(Self(Secret::new(parsed_id.to_string())))
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

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct TwoFACode(Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl TwoFACode {
    /// Parse a 2fa code
    /// ```
    /// use secrecy::ExposeSecret;
    /// use crate::auth_service::domain::data_stores::{TwoFACode, TwoFACodeError};
    /// let num = 912304;
    /// let valid_two_fa = TwoFACode::parse(num.to_string()).unwrap();
    /// assert_eq!(valid_two_fa.as_ref().expose_secret().parse::<u32>().unwrap(), num);
    ///
    /// // This should fail: passing in a non number
    /// let invalid = TwoFACode::parse("I am not a number".to_string());
    /// assert!(invalid.is_err());
    /// ```
    pub fn parse(code: String) -> Result<Self> {
        let code_as_u32 = code.parse::<u32>().wrap_err("Invalid UUID")?; // Updated!
        if (100_000..=999_999).contains(&code_as_u32) {
            Ok(Self(Secret::new(code)))
        } else {
            Err(eyre!("Code out of range"))
        }
    }
}

impl Default for TwoFACode {
    /// Use the `rand` crate to generate a random 2FA code.
    /// The code should be 6 digits (ex: 834629)
    /// ```
    /// use secrecy::ExposeSecret;
    /// use crate::auth_service::domain::data_stores::{TwoFACode};
    /// let default_val = TwoFACode::default();
    /// let val = default_val.as_ref();
    /// let char_count = val.expose_secret().chars().count();
    /// let is_between_0_and_6 = char_count < 7;
    /// assert!(is_between_0_and_6);
    /// ```
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        // Maybe it is not a good idea to hard-code these numbers here.
        // Makes writing tests difficult
        let number: u32 = rng.gen_range(100000..999999);
        Self::parse(format!("{}", number)).unwrap()
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}
