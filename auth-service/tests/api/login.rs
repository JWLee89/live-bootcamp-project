use super::{helpers::get_random_email, signup::SignUpKeys};
use auth_service::{
    domain::{email::Email, parse::Parseable},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
};
use reqwest::StatusCode;
use serde_json::Value;
use std::fmt;
use test_case::test_case;

use crate::helpers::{TestApp, _assert_eq_status_code};

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
    let mut app = TestApp::new().await;
    let response = app.login(&body).await;
    _assert_eq_status_code(&response, StatusCode::UNPROCESSABLE_ENTITY);
    app.clean_up().await;
}

#[test_case(get_test_case("", ""))]
#[test_case(get_test_case("lol---gmail.com", "some random password"))]
#[tokio::test]
async fn should_return_400_if_invalid_input(body: Value) {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let mut app = TestApp::new().await;
    let response = app.login(&body).await;
    _assert_eq_status_code(&response, StatusCode::BAD_REQUEST);
    app.clean_up().await;
}

#[test_case(get_test_case("dont_exist@hotmail.com", "some random password"))]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(test_case: Value) {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let mut app = TestApp::new().await;
    let response = app.login(&test_case).await;
    _assert_eq_status_code(&response, StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}

pub async fn prepare_login(requires_2fa: bool) -> (TestApp, Value) {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let email_str = "password123";
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": email_str,
        "requires2FA": requires_2fa
    });

    let response = app.signup(&signup_body).await;

    _assert_eq_status_code(&response, StatusCode::CREATED);

    (
        app,
        serde_json::json!({
            "email": random_email,
            "password": email_str,
        }),
    )
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let enable_efa = false;
    let (mut app, login_body) = prepare_login(enable_efa).await;
    let response: reqwest::Response = app.login(&login_body).await;

    _assert_eq_status_code(&response, StatusCode::OK);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let enable_2fa = true;
    let (mut app, login_body) = prepare_login(enable_2fa).await;
    let response = app.login(&login_body).await;
    _assert_eq_status_code(&response, StatusCode::PARTIAL_CONTENT);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());
    app.clean_up().await;

    // TODO: assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
    let two_fa_store = app.two_fa_code_store.read().await;

    let mut email_string = login_body
        .as_object()
        .unwrap()
        .get("email")
        .unwrap()
        .to_string();
    // Remove " character from start and end of string. Otherwise, we get an error
    email_string.pop();
    email_string.remove(0);

    let email = Email::parse(email_string).unwrap();
    let get_code_result = two_fa_store.get_code(&email).await;
    if let Err(e) = get_code_result {
        let error_msg = format!(
            "LoginAttempId should be added to two factor code store: {:?}",
            e
        );
        panic!("{}", error_msg);
    }
}
