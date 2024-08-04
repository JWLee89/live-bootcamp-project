use std::fmt;

use auth_service::{routes::SignupResponse, ErrorResponse};
use serde_json::Value;
use test_case::test_case;
use uuid::Uuid;

use crate::helpers::TestApp;

async fn get_test_app() -> TestApp {
    return TestApp::new().await;
}

// TODO: Create a general enum that can do to_string effectively
enum SignUpKeys {
    PASSWORD,
    EMAIL,
    REQUIRES2FA,
}

// TODO: See if we can create a general solution for this problem
// where the enum is not exactly the same
impl fmt::Display for SignUpKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_repr = match &self {
            SignUpKeys::EMAIL => "email",
            SignUpKeys::PASSWORD => "password",
            SignUpKeys::REQUIRES2FA => "requires2FA",
        };
        f.write_str(&string_repr)
        // Normally, we would do this, but we have
        // mix of upper and lower-case characters in our API
        // write!(f, "{:?}", self)
    }
}

fn get_test_case(password: &str, email: &str, requires_2fa: bool) -> Value {
    serde_json::json!({
        format!("{}", SignUpKeys::PASSWORD) : password,
        format!("{}", SignUpKeys::EMAIL): email,
        format!("{}", SignUpKeys::REQUIRES2FA): requires_2fa
    })
}

#[test_case(get_test_case("password123", "Invalid email right here", true))]
#[test_case(
    get_test_case("2short", get_random_email().as_str(), true)
)]
#[tokio::test]
async fn should_return_400_if_invalid_input(invalid_json_payload: Value) {
    let app = get_test_app().await;
    // Do HTTP Request
    let response = app.signup(&invalid_json_payload).await;
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters
    assert_eq!(
        response.status().as_u16(),
        400,
        "Failed for input: {:?}",
        invalid_json_payload
    );

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid credentials".to_owned()
    );
}

#[test_case(
    get_test_case("new_user", get_random_email().as_str(), true)
)]
#[test_case(
    get_test_case("captain_teemo", get_random_email().as_str(), false)
)]
#[tokio::test]
async fn should_return_409_if_email_already_exists(new_user: Value) {
    // Call the signup route twice.
    // The second request should fail with a 409 HTTP status code
    let app = get_test_app().await;
    // Do HTTP Request
    let response = app.signup(&new_user).await;
    // Should work the first time
    assert_eq!(response.status().as_u16(), 201);

    let response = app.signup(&new_user).await;
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}

#[test_case(get_test_case("password123", get_random_email().as_str(), true))]
#[tokio::test]
async fn should_return_201_if_valid_input(test_case: Value) {
    let app = get_test_app().await;

    let response = app.signup(&test_case).await;
    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    // Assert that we are getting the correct response body!
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

#[test_case(
    serde_json::json!({
        "password": "password123",
        "email": get_random_email(),
        "requires_teemoA": true
    })
)]
#[test_case(
    serde_json::json!({
        "email": get_random_email(),
        "password": "captain_teemo",
    })
)]
#[test_case(
    serde_json::json!({})
)]
#[tokio::test]
async fn should_return_422_if_malformed_input(test_case: Value) {
    let app = get_test_app().await;
    let response = app.signup(&test_case).await;
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
