use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::state::AppState, domain::user::User};


pub async fn signup(state: State<AppState>, Json(request): Json<SignupRequest>) -> impl IntoResponse {
    let email = request.email;
    let requires_2fa = request.requires_2fa;
    let password = request.password;
    let user = User::new( email, requires_2fa, Some(password));
    println!("User: {:?}", user);
    let mut user_store = state.user_store.write().await;
    match user_store.add_user(user) {
        // TODO: Have common messaging object to generate messages.
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_string(),
            });
            (StatusCode::CREATED, response)
        },
        // Might be better to return more meaningful message later on
        // Maybe there is also a better way to do this.
        Err(_) => {
            let response = Json(SignupResponse{
                message: "Failed to create user".to_string(),
            });
            (StatusCode::UNPROCESSABLE_ENTITY, response)
        }
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Debug, PartialEq, Deserialize)]
pub struct SignupResponse {
    pub message: String,
}

