use validator::ValidationError;

use super::error::AuthAPIError;

pub fn validation_to_incorrect_credentials_error(e: ValidationError) -> AuthAPIError {
    AuthAPIError::IncorrectCredentials
}
