use uuid::Uuid;

use crate::helpers::{TestApp};


fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "email": get_random_email(),
            "requires_teemoA": true
        }),
        // It might be good to store these inside 
        // of a reusable enum or struct
        serde_json::json!({
            "email": get_random_email(),
            "password": "captain_teemo",
        }),
        serde_json::json!({})
    ];

    for test_case in test_cases.iter() {
        let response = app.signup(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}