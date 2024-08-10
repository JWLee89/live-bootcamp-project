use std::borrow::BorrowMut;

use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use reqwest::Url;
use serde_json::Value;
use test_case::test_case;

use crate::helpers::{get_random_email, TestApp};
use crate::signup::get_test_case;

fn create_logout_payload(jwt: &str) -> Value {
    serde_json::json!({
        "jwt": jwt
    })
}
// #[test_case(serde_json::json!({"invalid key": "invalid value"}))]
// #[test_case(serde_json::json!({}))]
#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;
    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

// #[test_case(create_logout_payload("invalid_jwt"))]
#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 401);
}

#[test_case("captain teemo password", get_random_email(), true)]
#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie(password: &str, email: String, enable_2fa: bool) {
    let app = TestApp::new().await;
    test_logout_once(&app, &password, &email, enable_2fa).await;
}

async fn test_logout_once(app: &TestApp, password: &str, email: &str, enable_2fa: bool) {
    // Need to signup new usper
    let response = app
        .signup(&get_test_case(password, &email, enable_2fa))
        .await;
    assert_eq!(response.status().as_u16(), 201);

    // Do login
    let login_payload = serde_json::json!({
        "password": &password,
        "email": &email
    });
    let response = app.post_login(&login_payload).await;
    // TODO: Maybe change all the http status code from into to enum integers
    // 200 is quite easy, but some of the more obscure ones might be difficult
    // for people who are not web developers
    assert_eq!(response.status().as_u16(), 200);
    // TODO: duplicate code: add to helper
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Authentication cookie not found");
    assert!(!auth_cookie.value().is_empty());

    // Logout
    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 200);
}

#[test_case("captain teemo password", get_random_email(), true)]
#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row(
    password: &str,
    email: String,
    enable_2fa: bool,
) {
    let app = TestApp::new().await;
    test_logout_once(&app, &password, &email, enable_2fa).await;
    // This should fail since we logged out already
    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 400);
}
