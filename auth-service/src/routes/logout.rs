use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let token = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_owned(),
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    // TODO: Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    if let Ok(_) = validate_token(&token).await {
        let jar = jar.remove(JWT_COOKIE_NAME);
        (jar, Ok(StatusCode::OK))
    } else {
        (jar, Err(AuthAPIError::InvalidToken))
    }
}
