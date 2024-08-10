/// Enums for Authentication API-related Errors.
/// Note: this does not map errors to HTTP status codes.
///
/// ## Specifications
///
/// UserAlreadyExists: Should occur if the user already exists
/// InvalidCredentials: Username and / or password is incorrect
/// UnexpectedError: Any other error that does not already exists
/// inside of the enum
#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    MissingToken,
    InvalidToken,
    UnexpectedError,
    // When trying to login
    IncorrectCredentials,
}
