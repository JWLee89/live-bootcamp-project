use validator::ValidationError;

use super::parse::Parseable;
use color_eyre::eyre::Result;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

const MINIMUM_PASSWORD_LEN: usize = 8;

impl Parseable<Secret<String>> for Password {
    fn parse(password: Secret<String>) -> Result<Self> {
        // Password cannot be longer
        if &password.expose_secret().len() < &MINIMUM_PASSWORD_LEN {
            Err(ValidationError::new("Invalid password: Must contain at least 8 characters").into())
        } else {
            Ok(Password(password))
        }
    }
}

impl PartialEq for Password {
    // New!
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret() // Updated!
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use test_case::test_case;

    #[test_case(
        "I am a long string".to_string()
    )]
    #[test_case(
        "lengthi8".to_string()
    )]
    fn should_be_valid(valid_password: String) {
        let password = Password::parse(Secret::new(valid_password.clone()));
        if let Err(_) = password {
            panic!("Should be a valid password: {}", valid_password);
        } else {
            let unwrapped = password.unwrap();
            let password = unwrapped.as_ref().expose_secret();
            assert_eq!(password, &valid_password);
        }
    }

    #[test_case(
        (0..7).fake::<String>()
    )]
    #[test_case(String::from(""))]
    fn should_be_invalid(invalid_password: String) {
        // using convenient function without providing locale
        let password = Password::parse(Secret::new(invalid_password));
        assert_eq!(password.is_err(), true);
    }
}
