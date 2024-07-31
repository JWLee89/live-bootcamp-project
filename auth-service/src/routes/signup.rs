use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::state::AppState, domain::{error::AuthAPIError, user::User}};


pub async fn signup(state: State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    // We should do validation elesewhere
    if request.password.len() < 8 || request.email.is_empty() || !request.email.contains("@") {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new( request.email, request.requires_2fa, 
            Some(request.password));

    let mut user_store = state.user_store.write().await;
    match user_store.add_user(user) {
        // TODO: Have common messaging object to generate messages.
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_string(),
            });
            Ok((StatusCode::CREATED, response))
        },
        // Might be better to return more meaningful message later on
        // Maybe there is also a better way to do this.
        Err(_) => {
            Err(AuthAPIError::UserAlreadyExists)
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

