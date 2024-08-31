use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::state::AppState,
    domain::{
        data_stores::{LoginAttemptId, TwoFACode},
        email::Email,
        error::AuthAPIError,
        parse::Parseable,
    },
    utils::auth::generate_auth_cookie,
};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}

#[tracing::instrument(name = "Verify two-factor auth", skip_all)]
pub async fn verify_2fa(
    state: State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // perform input validation
    // TODO: Can we reduce duplicate: |_| AuthAPIError::InvalidCredentials
    let email =
        Email::parse(Secret::new(request.email)).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id)
        .map_err(|_| AuthAPIError::InvalidCredentials)?;
    let two_fa_code =
        TwoFACode::parse(request.two_fa_code).map_err(|_| AuthAPIError::IncorrectCredentials)?;

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    // Call `two_fa_code_store.get_code`. If the call fails
    // return a `AuthAPIError::IncorrectCredentials`.
    let (stored_login_attempt_id, stored_two_fa_code) =
        match two_fa_code_store.get_code(&email).await {
            Ok((login_attempt_id, two_fa_code)) => (login_attempt_id, two_fa_code),
            Err(_) => return Err(AuthAPIError::IncorrectCredentials),
        };

    // Validate code
    if stored_login_attempt_id != login_attempt_id || stored_two_fa_code != two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    // The cookie jar should get updated with a new JWT auth cookie if the email,
    // login attempt ID, and 2FA code are correct.
    if let Err(e) = two_fa_code_store.remove_code(&email).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(e) => return Err(AuthAPIError::UnexpectedError(e.into())),
    };

    let updated_jar = jar.add(cookie);
    Ok((updated_jar, StatusCode::OK.into_response()))
}
