use std::sync::Arc;

use auth_service::{
    app_state::state::AppState,
    domain::{
        data_stores::{configure_postgresql, configure_redis},
        redis_two_fa_code_store::RedisTwoFACodeStore,
    },
    services::{
        data_stores::PostgresUserStore, mock_email_client::MockEmailClient,
        redis_banned_token_store::RedisBannedTokenStore,
    },
    utils::{constants::prod, tracing::init_tracing},
    Application,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    // Redis and PostgreSQL
    let redis_connection = Arc::new(RwLock::new(configure_redis()));
    let pool = configure_postgresql().await;

    // Store initializations
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pool)));
    let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
        redis_connection.clone(),
    )));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));

    // Old hashmap-based stores
    // let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
    // let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));

    let app_state: AppState = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
