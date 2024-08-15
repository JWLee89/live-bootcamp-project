use std::sync::Arc;

use auth_service::{
    app_state::state::AppState,
    domain::{
        data_stores::HashsetBannedTokenStore, hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    },
    services::hashmap_user_store::HashMapUserStore,
    utils::constants::prod,
    Application,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let store: Arc<RwLock<HashMapUserStore>> = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::new()));
    let two_fa_code_store: Arc<RwLock<HashMapTwoFACodeStore>> =
        Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));
    let app_state: AppState = AppState::new(store, banned_token_store, two_fa_code_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
