use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};

#[tokio::test]
async fn verify_token() {
    let app = TestApp::new().await;
    let response = app.verify_token().await;
    _assert_eq_status_code(&response, HttpStatusCode::OK)
}
