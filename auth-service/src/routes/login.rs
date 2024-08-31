use ::serde::{Deserialize, Serialize};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, Secret};

use crate::{
    app_state::state::AppState,
    domain::{
        data_stores::{LoginAttemptId, TwoFACode},
        email::Email,
        error::AuthAPIError,
        parse::Parseable,
        password::Password,
    },
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "Login to the application", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // TODO: fix. Should be doing validation / error handling here
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Validation
    let user_store = &state.user_store.read().await;

    // raised if user does not exist
    user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    // Raised if username and password info does not match
    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    // Handle request based on user's 2FA configuration
    // Cases where this can fail:
    let response = match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await?,
        false => handle_no_2fa(&user.email, jar).await?,
    };
    Ok((response.0, response.1))
}

#[tracing::instrument(name = "Handle Two-factor Auth", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    // First, we must generate a new random login attempt ID and 2FA code
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    if let Err(e) = state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
    {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    // Send 2FA email
    let email_client = state.email_client.write().await;
    let subject = "2FA code has been sent!";
    if let Err(e) = email_client
        .send_email(&email, subject, two_fa_code.as_ref().expose_secret())
        .await
    {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    // Finally, we need to return the login attempt ID to the client
    let response: Json<LoginResponse> = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().expose_secret().to_string(),
    }));

    Ok((jar, (StatusCode::PARTIAL_CONTENT, response)))
}

#[tracing::instrument(name = "Handle no two-factor auth", skip_all)]
pub async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    // update cookie
    let auth_cookie = generate_auth_cookie(&email).map_err(|e| AuthAPIError::UnexpectedError(e))?;
    let updated_jar = jar.add(auth_cookie);
    Ok((
        updated_jar,
        (StatusCode::OK, axum::Json(LoginResponse::RegularAuth)),
    ))
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
}
