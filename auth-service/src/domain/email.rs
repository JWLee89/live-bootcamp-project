use std::borrow::Cow;
use std::hash::Hash;

use color_eyre::eyre::Result;
use secrecy::{ExposeSecret, Secret};
use validator::{validate_email, ValidationError};

use super::parse::Parseable;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Parseable<Secret<String>> for Email {
    fn parse(email: Secret<String>) -> Result<Self> {
        let email_str = email.expose_secret();
        if validate_email(email_str) {
            Ok(Email(email))
        } else {
            let mut val_err = ValidationError::new("invalid email address");
            val_err.add_param(Cow::Borrowed("input"), &email_str);
            Err(val_err.into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("teemo@gmail.com")]
    #[test_case("mooomin@hotmail.com")]
    fn expect_valid_emails(valid_email: &str) {
        let email = Email::parse(Secret::new(valid_email.to_string()));
        assert_eq!(email.is_ok(), true);
        assert_eq!(email.unwrap().as_ref().expose_secret(), valid_email);
    }

    #[test_case(
        Email::parse(Secret::new("teemo".to_string()))
    )]
    #[test_case(
        Email::parse(Secret::new("teemo@gmail@badger.com".to_string()))
    )]
    #[test_case(
        Email::parse(Secret::new("woo@min_at_asdsad".to_string()))
    )]
    fn should_be_invalid_emails(invalid_email: Result<Email>) {
        // TODO: update tests to make it more robust
        assert_eq!(invalid_email.is_err(), true);
    }
}
