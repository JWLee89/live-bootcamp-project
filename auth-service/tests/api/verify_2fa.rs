use crate::helpers::{HttpStatusCode, TestApp, _assert_eq_status_code};
use serde_json::Value;
use test_case::test_case;

// #[test_case(serde_json::json!({
//     "invalid_key": 10
// }))]
// #[tokio::test]
// async fn should_return_422_if_malformed_input(invalid_payload: Value) {
//     // Note the following two lines are repeated.
//     // See if we can remove duplicate code later
//     let app = TestApp::new().await;
//     let response = app.verify_2fa(&invalid_payload).await;
//     _assert_eq_status_code(&response, HttpStatusCode::MalformedInput)
// }
