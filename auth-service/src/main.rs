use std::sync::Arc;

use auth_service::{
    app_state::state::AppState,
    domain::{
        data_stores::HashsetBannedTokenStore, hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    },
    get_postgres_pool,
    services::{hashmap_user_store::HashMapUserStore, mock_email_client::MockEmailClient},
    utils::constants::{prod, DATABASE_URL},
    Application,
};
use sqlx::PgPool;
use tokio::sync::RwLock;

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL).await.expect(
        format!(
            "Failed to create Postgres connection pool! Target url: {}",
            DATABASE_URL.to_string()
        )
        .as_str(),
    );

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

#[tokio::main]
async fn main() {
    let store = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
    let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
    let app_state: AppState =
        AppState::new(store, banned_token_store, two_fa_code_store, email_client);

    // Temp placeholder:
    let pg_pool = configure_postgresql().await;

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
