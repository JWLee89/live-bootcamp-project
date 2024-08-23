use crate::helpers::{TestApp, _assert_eq_status_code, get_random_email};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::StatusCode;
use serde_json::Value;
use test_case::test_case;

#[test_case(serde_json::json!({
    "invalid_key": "teemo"
}))]
#[test_case(serde_json::json!({
}))]
#[tokio::test]
async fn should_return_422_if_malformed_input(invalid_payload: Value) {
    let mut app = TestApp::new().await;
    let response = app.verify_token(&invalid_payload).await;
    _assert_eq_status_code(&response, StatusCode::UNPROCESSABLE_ENTITY);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.signup(&signup_body).await;

    _assert_eq_status_code(&response, StatusCode::CREATED);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.login(&login_body).await;

    _assert_eq_status_code(&response, StatusCode::OK);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    // do validation
    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value(),
    });
    let response = app.verify_token(&verify_token_body).await;
    _assert_eq_status_code(&response, StatusCode::OK);
    app.clean_up().await;
}

#[test_case(serde_json::json!({
    "token": "invalid_token",
}))]
#[tokio::test]
async fn should_return_401_if_invalid_token(invalid_token_payload: Value) {
    let mut app = TestApp::new().await;
    let response = app.verify_token(&invalid_token_payload).await;
    _assert_eq_status_code(&response, StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    // TODO: refactor
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.signup(&signup_body).await;

    _assert_eq_status_code(&response, StatusCode::CREATED);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.login(&login_body).await;

    _assert_eq_status_code(&response, StatusCode::OK);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    // ban current token
    if let Err(_) = app
        .banned_token_store
        .write()
        .await
        .insert(auth_cookie.value().to_string())
    {
        panic!("Ban token store addition feature is not working as expected")
    }

    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value(),
    });
    let response = app.verify_token(&verify_token_body).await;
    // Should not work because token is banned
    _assert_eq_status_code(&response, StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
