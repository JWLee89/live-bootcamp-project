use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_response, _assert_eq_status_code};

// Tokio's test macro is use to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response: reqwest::Response = app.get_root().await;

    _assert_eq_status_code(&response, HttpStatusCode::OK);
    _assert_eq_response(&response, "content-type", "text/html");
}
