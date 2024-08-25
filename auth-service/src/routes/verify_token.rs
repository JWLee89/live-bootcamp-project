use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{app_state::state::AppState, domain::error::AuthAPIError, utils::auth::validate_token};

#[derive(Deserialize, Debug)]
pub struct VerifyTokenRequest {
    token: String,
}

#[tracing::instrument("Verify authentication token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    let token = request.token.as_str();
    let banned_token_store = state.banned_token_store;
    match validate_token(&token, banned_token_store).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => return Err(AuthAPIError::InvalidToken),
    }
}
