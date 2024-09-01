use std::{str::FromStr, sync::Arc};

use auth_service::{
    app_state::state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType},
    domain::{
        data_stores::configure_redis, email::Email, parse::Parseable,
        redis_two_fa_code_store::RedisTwoFACodeStore,
    },
    get_postgres_pool,
    services::{
        data_stores::PostgresUserStore, postmark_email_client::PostmarkEmailClient,
        redis_banned_token_store::RedisBannedTokenStore,
    },
    utils::constants::{test, DATABASE_URL},
    Application,
};
use reqwest::{cookie::Jar, Client, StatusCode};
use secrecy::Secret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub db_name: String,
    pub clean_up_called: bool,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub email_server: MockServer,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

/// Check whether a status code is expected value
pub fn _assert_eq_status_code(response: &reqwest::Response, http_response_code: StatusCode) {
    assert_eq!(response.status().as_u16(), http_response_code.as_u16());
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub fn _assert_eq_response(response: &reqwest::Response, key: &str, expected_value: &str) {
    // Note that this can raise an error
    let value = response.headers().get(key).unwrap();
    assert_eq!(value, expected_value);
}

// New!
fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender = Email::parse(Secret::new(test::email_client::SENDER.to_owned())).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect(
            format!(
                "Failed to create Postgres connection pool! Current url: {}",
                postgresql_conn_url_with_db
            )
            .as_str(),
        )
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let redis_connection = Arc::new(RwLock::new(configure_redis()));
        let pg_pool = configure_postgresql(&db_name).await;
        // Set up a mock email server
        let email_server = MockServer::start().await; // New!
        let base_url = email_server.uri(); // New!
        let email_client = Arc::new(RwLock::new(configure_postmark_email_client(base_url))); // Updated!

        // Required stores
        let store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
            redis_connection.clone(),
        )));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));
        // let email_client = Arc::new(RwLock::new(MockEmailClient::default()));

        let app_state: AppState = AppState::new(
            store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap(); // Create a Reqwest http client instance

        let clean_up_called = false;
        // Create new `TestApp` instance and return it
        Self {
            address,
            db_name,
            clean_up_called,
            cookie_jar,
            http_client,
            email_server,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true;
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // TODO: Implement helper functions for all other routes
    // (signup, login, logout, verify-2fa, and verify-token)
    pub async fn signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to handle signup request")
    }

    /// Handle user login
    pub async fn login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to handle login")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to handle logout")
    }

    pub async fn verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to verify two factor authentication")
    }

    pub async fn verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("Clean up should always be called after running TestApp")
        }
    }
}
