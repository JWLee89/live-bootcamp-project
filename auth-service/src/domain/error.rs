use color_eyre::eyre::Report;
use thiserror::Error;
/// Enums for Authentication API-related Errors.
/// Note: this does not map errors to HTTP status codes.
///
/// ## Specifications
///
/// UserAlreadyExists: Should occur if the user already exists
/// InvalidCredentials: Username and / or password is incorrect
/// UnexpectedError: Any other error that does not already exists
/// inside of the enum
#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}
