use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{app_state::state::AppState, domain::{email::Email, error::AuthAPIError, parse::Parseable, password::Password, user::User}};

pub async fn login(
    State(state): State<AppState>, // New!
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    // Validation
    let user_store = &state.user_store.read().await;
    // Raised if username and password info does not match
    user_store.validate_user(&email, &password).await.map_err(|_| AuthAPIError::IncorrectCredentials)?;
    // raised if user does not exist
    user_store.get_user(&email).await.map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Okay, it is now safe to perform login
    let user: User = User::new(email, false, password);

    Ok(StatusCode::OK.into_response())

}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
