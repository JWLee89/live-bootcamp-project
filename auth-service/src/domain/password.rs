use validator::ValidationError;

use super::parse::Parseable;


#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Password(String);


impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

const MINIMUM_PASSWORD_LEN: usize = 8;

impl Parseable<String, ValidationError> for Password {
    fn parse(password: String) -> Result<Password, ValidationError> {
        // Password cannot be longer 
        if &password.len() < &MINIMUM_PASSWORD_LEN {
            Err(ValidationError::new("Invalid password: Must contain at least 8 characters"))
        } else {
            Ok(Password(password))
        }
    }
    
    type Output = Password;
}


#[cfg(test)]
mod tests {
    use test_case::test_case;
    use fake::Fake;
    use super::*;

    #[test_case(
        "I am a long string".to_string()
    )]
    #[test_case(
        "lengthi8".to_string()
    )]
    fn should_be_valid(valid_password: String) {
        let password = Password::parse(valid_password.to_string());
        assert_eq!(password.is_ok(), true);
        assert_eq!(password.unwrap().as_ref(), valid_password);
    }

   
    #[test_case(
        (0..7).fake::<String>()
    )]
    fn should_be_invalid(invalid_password: String) {
        // using convenient function without providing locale
        let password = Password::parse(invalid_password);
        assert_eq!(password.is_err(), true);
    }


}
