use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::state::AppState,
    domain::{email::Email, error::AuthAPIError, parse::Parseable},
    utils::auth::validate_token,
};

pub async fn verify_2fa(State(state): State<AppState>) -> Result<impl IntoResponse, AuthAPIError> {
    Ok(StatusCode::OK.into_response())
}
