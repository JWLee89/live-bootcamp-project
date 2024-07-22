use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};


#[tokio::test]
async fn signup_works() {
    let app = TestApp::new().await;
    let response = app.signup().await;
    // TODO: Create or use enums for common status codes like 200, 404, etc.
    _assert_eq_status_code(&response, HttpStatusCode::OK);
}