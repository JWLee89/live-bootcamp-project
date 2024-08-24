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
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Unexpected Error")]
    UnexpectedError,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
}
