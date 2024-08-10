use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code, get_random_email};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::Value;
use test_case::test_case;

#[test_case(serde_json::json!({
    "invalid_key": "teemo"
}))]
#[test_case(serde_json::json!({
}))]
#[tokio::test]
async fn should_return_422_if_malformed_input(invalid_payload: Value) {
    let test_app = TestApp::new().await;
    let response = test_app.verify_token(&invalid_payload).await;
    _assert_eq_status_code(&response, HttpStatusCode::MalformedInput);
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.signup(&signup_body).await;

    _assert_eq_status_code(&response, HttpStatusCode::Created);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.login(&login_body).await;

    _assert_eq_status_code(&response, HttpStatusCode::OK);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    // do validation
    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value(),
    });
    let response = app.verify_token(&verify_token_body).await;
    _assert_eq_status_code(&response, HttpStatusCode::OK);
}

#[test_case(serde_json::json!({
    "token": "invalid_token",
}))]
#[tokio::test]
async fn should_return_401_if_invalid_token(invalid_token_payload: Value) {
    let app = TestApp::new().await;
    let response = app.verify_token(&invalid_token_payload).await;
    _assert_eq_status_code(&response, HttpStatusCode::Unauthorized);
}
