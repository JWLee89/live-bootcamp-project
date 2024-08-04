use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};

#[tokio::test]
async fn verify_2fa() {
    // Note the following two lines are repeated.
    // See if we can remove duplicate code later
    let app = TestApp::new().await;
    let response = app.verify_2fa().await;
    _assert_eq_status_code(&response, HttpStatusCode::OK)
}
