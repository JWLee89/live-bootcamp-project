use super::{helpers::get_random_email, signup::SignUpKeys};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use serde_json::Value;
use std::fmt;
use test_case::test_case;

use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};

#[derive(Debug)]
enum LoginKeys {
    EMAIL,
    PASSWORD,
}

impl fmt::Display for LoginKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Get test case
fn get_test_case(email: &str, password: &str) -> Value {
    serde_json::json!({
        format!("{}", LoginKeys::EMAIL) : email,
        format!("{}", LoginKeys::PASSWORD): password,
    })
}

// Missing password
#[test_case(serde_json::json!({
    format!("{}", SignUpKeys::EMAIL): "lol@gmail.com",
    format!("{}", SignUpKeys::REQUIRES2FA): true
}))]
// Missing Email
#[test_case(serde_json::json!({
    format!("{}", SignUpKeys::PASSWORD): "asjdoasfa21",
    format!("{}", SignUpKeys::REQUIRES2FA): true
}))]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(body: Value) {
    let app = TestApp::new().await;
    let response = app.login(&body).await;
    _assert_eq_status_code(&response, HttpStatusCode::MalformedInput);
}

#[test_case(get_test_case("", ""))]
#[test_case(get_test_case("lol---gmail.com", "some random password"))]
#[tokio::test]
async fn should_return_400_if_invalid_input(body: Value) {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let response = app.login(&body).await;
    _assert_eq_status_code(&response, HttpStatusCode::BadRequest);
}

#[test_case(get_test_case("dont_exist@hotmail.com", "some random password"))]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(test_case: Value) {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let response = app.login(&test_case).await;
    _assert_eq_status_code(&response, HttpStatusCode::Unauthorized);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
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

    assert!(!auth_cookie.value().is_empty());
}
