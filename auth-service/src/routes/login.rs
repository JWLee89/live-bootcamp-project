use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::state::AppState,
    domain::{email::Email, error::AuthAPIError, parse::Parseable, password::Password, user::User},
    utils::auth::generate_auth_cookie,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // TODO: fix. Should be doing validation / error handling here
    let email = match Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials) {
        Ok(email) => email,
        Err(err) => return (jar, Err(err)),
    };
    let password =
        match Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials) {
            Ok(password) => password,
            Err(err) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        };

    // Validation
    let user_store = &state.user_store.read().await;

    // raised if user does not exist
    match user_store.get_user(&email).await {
        Ok(_) => {}
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // Raised if username and password info does not match
    match user_store.validate_user(&email, &password).await {
        Ok(_) => {}
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    // update cookie
    let auth_cookie = generate_auth_cookie(&email).unwrap();
    let updated_jar = jar.add(auth_cookie);

    // Okay, it is now safe to perform login
    let user: User = User::new(email, false, password);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
