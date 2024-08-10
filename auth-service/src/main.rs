use std::sync::Arc;

use auth_service::{
    app_state::state::AppState, services::hashmap_user_store::HashMapUserStore,
    utils::constants::prod, Application,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let store: Arc<RwLock<HashMapUserStore>> = Arc::new(RwLock::new(HashMapUserStore::default()));

    let app_state: AppState = AppState::new(store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
