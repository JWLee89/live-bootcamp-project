use std::sync::Arc;

use auth_service::{
    app_state::state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType},
    domain::{
        data_stores::HashsetBannedTokenStore, hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    },
    get_postgres_pool,
    services::{
        data_stores::PostgresUserStore, hashmap_user_store::HashMapUserStore,
        mock_email_client::MockEmailClient,
    },
    utils::constants::{test, DATABASE_URL},
    Application,
};
use reqwest::{cookie::Jar, StatusCode};
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
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

async fn configure_postgresql() -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
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

impl TestApp {
    pub async fn new() -> Self {
        // TODO: Abstract this out
        // let store: Arc<RwLock<HashMapUserStore>> =
        //     Arc::new(RwLock::new(HashMapUserStore::default()));
        let pg_pool = configure_postgresql().await;
        let store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
        let two_fa_code_store: Arc<RwLock<HashMapTwoFACodeStore>> =
            Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
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

        // Create new `TestApp` instance and return it
        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
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
