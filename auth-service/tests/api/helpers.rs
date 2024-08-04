use std::sync::Arc;

use auth_service::{
    app_state::state::AppState, services::hashmap_user_store::HashMapUserStore, Application,
};
use tokio::sync::RwLock;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

// TODO: See whether rust provides common enum
// for HTTP status codes
#[repr(u16)]
#[derive(PartialEq, Debug, Default, Eq, Hash, Clone)]
pub enum HttpStatusCode {
    #[default]
    OK = 200,
}

/// Check whether a status code is expected value
pub fn _assert_eq_status_code(response: &reqwest::Response, http_response_code: HttpStatusCode) {
    assert_eq!(response.status().as_u16(), http_response_code as u16);
}

pub fn _assert_eq_response(response: &reqwest::Response, key: &str, expected_value: &str) {
    // Note that this can raise an error
    let value = response.headers().get(key).unwrap();
    assert_eq!(value, expected_value);
}

impl TestApp {
    pub async fn new() -> Self {
        let store: Arc<RwLock<HashMapUserStore>> =
            Arc::new(RwLock::new(HashMapUserStore::default()));
        let app_state: AppState = AppState::new(store);
        let app = Application::build(app_state, "0.0.0.0:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new(); // Create a Reqwest http client instance

        // Create new `TestApp` instance and return it
        TestApp {
            address,
            http_client,
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
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
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

    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("Failed to verify two factor authentication")
    }

    pub async fn verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .send()
            .await
            .expect("Failed to verify token")
    }
}
