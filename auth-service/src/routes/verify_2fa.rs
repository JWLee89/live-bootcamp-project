use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::{app_state::state::AppState, domain::error::AuthAPIError};

pub async fn verify_2fa(State(state): State<AppState>) -> Result<impl IntoResponse, AuthAPIError> {
    Ok(StatusCode::OK.into_response())
}
