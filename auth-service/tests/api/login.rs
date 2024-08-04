use serde_json::Value;
use std::fmt;
use test_case::test_case;

use crate::helpers::TestApp;

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

#[test_case(get_test_case("lol@gmail.com", "some random password"))]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(body: Value) {
    let app = TestApp::new().await;
    let response = app.post_login(&body).await;
}

#[test_case(get_test_case("lol---gmail.com", "some random password"))]
#[tokio::test]
async fn should_return_400_if_invalid_input(body: Value) {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[test_case(get_test_case("dont_exist@hotmail.com", "some random password"))]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(test_case: Value) {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let response = app.post_login(&test_case).await;
    assert_eq!(response.status().as_u16(), 401);
}
