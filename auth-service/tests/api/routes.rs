use crate::helpers::TestApp;

/// Check whether a status code is expected value
fn _assert_eq_status_code(response: &reqwest::Response, http_response_code: HttpStatusCode) {
    assert_eq!(response.status().as_u16(), http_response_code as u16);
}

fn _assert_eq_response(response: &reqwest::Response, key: &str, expected_value: &str) {
    // Note that this can raise an error
    let value = response.headers().get(key).unwrap();
    assert_eq!(value, expected_value);
}

// TODO: See whether rust provides common enum
// for HTTP status codes
#[repr(u16)]
#[derive(PartialEq, Debug, Default, Eq, Hash, Clone)]
enum HttpStatusCode {
    #[default]
    OK = 200,
}

// Tokio's test macro is use to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response: reqwest::Response = app.get_root().await;

    _assert_eq_status_code(&response, HttpStatusCode::OK);
    _assert_eq_response(&response, "content-type", "text/html");
}

#[tokio::test]
async fn signup_works() {
    let app = TestApp::new().await;
    let response = app.signup().await;
    // TODO: Create or use enums for common status codes like 200, 404, etc.
    _assert_eq_status_code(&response, HttpStatusCode::OK);
}

#[tokio::test]
async fn login_works() {
    let app = TestApp::new().await;
    let response = app.login().await;

    _assert_eq_status_code(&response, HttpStatusCode::OK)
}


#[tokio::test]
async fn logout_works() {
    let app = TestApp::new().await;
    let response = app.logout().await;

    _assert_eq_status_code(&response, HttpStatusCode::OK)
}

#[tokio::test]
async fn verify_2fa() {
    // Note the following two lines are repeated. 
    // See if we can remove duplicate code later
    let app = TestApp::new().await;
    let response = app.verify_2fa().await;
    _assert_eq_status_code(&response, HttpStatusCode::OK)
}



#[tokio::test]
async fn verify_token() {
    let app = TestApp::new().await;
    let response = app.verify_token().await;
    _assert_eq_status_code(&response, HttpStatusCode::OK)
}