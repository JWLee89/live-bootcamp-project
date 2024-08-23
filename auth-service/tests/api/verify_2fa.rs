use crate::{
    helpers::{TestApp, _assert_eq_status_code},
    login::prepare_login,
};
use auth_service::{
    domain::{data_stores::LoginAttemptId, email::Email, parse::Parseable},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
};
use reqwest::StatusCode;
use serde_json::Value;
use test_case::test_case;

#[test_case(serde_json::json!({}))]
#[test_case(serde_json::json!({
    "email": "bob@gmail.com",
    "loginAttemptId": LoginAttemptId::default().as_ref(),
    "2FACodeIsWrong": "invalidcode",
}))]
#[test_case(serde_json::json!({
    "email": "bob@gmail.com",
    "loginAttemptIdKeyIsWrong": "",
    "2FACode": "123452",
}))]
#[tokio::test]
async fn should_return_422_if_malformed_input(malformed_body: Value) {
    let mut app = TestApp::new().await;
    let response = app.verify_2fa(&malformed_body).await;
    _assert_eq_status_code(&response, StatusCode::UNPROCESSABLE_ENTITY);
    app.clean_up().await;
}

async fn prepare_200_case() -> (TestApp, Value, Value) {
    let enable_2fa = true;
    let (app, login_body) = prepare_login(enable_2fa).await;
    let login_body_map = login_body.as_object().unwrap();
    let email = login_body_map.get("email").unwrap().as_str().unwrap();

    // Call login
    let response_login = app.login(&login_body).await;
    let email_object = Email::parse(email.to_string()).unwrap();

    // Get two factor auth code
    let two_fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email_object)
        .await
        .expect("Two fractor authentication Code should have been registered");

    // Get response body
    let response_body = response_login
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    // Create packet for verify_2fa endpoint
    let login_attempt_id = response_body.login_attempt_id;
    let verify_2fa_body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code.1.as_ref(),
    });
    (app, verify_2fa_body, login_body)
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let (mut app, verify_2fa_body, _) = prepare_200_case().await;
    // This should pass, since we are calling verify-2fa for the first time
    let response_verify_fa = app.verify_2fa(&verify_2fa_body).await;
    _assert_eq_status_code(&response_verify_fa, StatusCode::OK);
    app.clean_up().await;
}

#[test_case(serde_json::json!({
    "email": "bob@gmail.com",
    "loginAttemptId": LoginAttemptId::default().as_ref(),
    "2FACode": "invalidcode",
}))]
#[test_case(serde_json::json!({
    "email": "another_email@hotmal.com",
    "loginAttemptId": LoginAttemptId::default().as_ref(),
    "2FACode": "123452",
}))]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(incorrect_body: Value) {
    let mut app = TestApp::new().await;
    let response = app.verify_2fa(&incorrect_body).await;
    _assert_eq_status_code(&response, StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let (mut app, verify_2fa_body, _) = prepare_200_case().await;
    // This should pass, since we are calling verify-2fa for the first time
    let response_verify_fa = app.verify_2fa(&verify_2fa_body).await;
    _assert_eq_status_code(&response_verify_fa, StatusCode::OK);

    let auth_cookie = response_verify_fa
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    let response = app.verify_2fa(&verify_2fa_body).await;
    _assert_eq_status_code(&response, StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
