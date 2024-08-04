use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};

#[tokio::test]
async fn logout_works() {
    let app = TestApp::new().await;
    let response = app.logout().await;

    _assert_eq_status_code(&response, HttpStatusCode::OK)
}
