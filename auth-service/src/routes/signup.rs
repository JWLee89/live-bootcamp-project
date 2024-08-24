use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::state::AppState,
    domain::{email::Email, error::AuthAPIError, parse::Parseable, password::Password, user::User},
};

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Invalid values will raise error
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user = User::new(email, request.requires_2fa, password);

    let mut user_store = state.user_store.write().await;
    match user_store.add_user(user).await {
        // TODO: Have common messaging object to generate messages.
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_string(),
            });
            Ok((StatusCode::CREATED, response))
        }
        // Might be better to return more meaningful message later on
        // Maybe there is also a better way to do this.
        Err(_) => Err(AuthAPIError::UserAlreadyExists),
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
