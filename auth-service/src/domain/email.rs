use validator::{validate_email, ValidationError};

use super::parse::Parseable;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


impl Parseable<String, ValidationError> for Email {
    type Output = Email;
    fn parse(email: String) -> Result<Email, ValidationError>  {
        if validate_email(&email) {
            Ok(Email(email))
        } else {
            Err(ValidationError::new("invalid email address"))
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
        let email = Email::parse(valid_email.to_string());
        assert_eq!(email.is_ok(), true);
        assert_eq!(email.unwrap().as_ref(), valid_email);
    }

    #[test_case(
        Email::parse("teemo".to_string())
    )]
    #[test_case(
        Email::parse("teemo@gmail@badger.com".to_string())
    )]
    #[test_case(
        Email::parse("woo@min_at_asdsad".to_string())
    )]
    fn should_be_invalid_emails(invalid_email: Result<Email, ValidationError>) {
        assert_eq!(invalid_email.is_err(), true);
        if let Err(e) = invalid_email {
            assert_eq!(e, ValidationError::new("invalid email address"));
        }
    }

}
