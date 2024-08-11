use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::state::AppState,
    domain::error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let token = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_owned(),
        None => return Err(AuthAPIError::MissingToken),
    };

    // Return if invalid token
    let banned_token_store = &state.banned_token_store;
    validate_token(&token, banned_token_store.clone())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    // Try adding token to ban list
    state
        .banned_token_store
        .write()
        .await
        .insert(token.to_owned())
        .map_err(|_| AuthAPIError::UnexpectedError)?;
    // Remove JWT cookie
    let jar: CookieJar = jar.remove(JWT_COOKIE_NAME);
    Ok((jar, StatusCode::OK))
}
