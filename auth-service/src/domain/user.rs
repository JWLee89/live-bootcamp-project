use super::{email::Email, password::Password};

/// Represents a new user
#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub email: Email,
    pub requires_2fa: bool,
    pub password: Password,
}

impl User {
    /// Create a new user
    /// ```
    /// use crate::auth_service::domain::user::User;
    /// use crate::auth_service::domain::parse::Parseable;
    /// use crate::auth_service::domain::email::Email;
    /// use crate::auth_service::domain::password::Password;
    ///
    /// let email = Email::parse("cowbell@email.com".to_string()).unwrap();
    /// let password = Password::parse("PasswordValid".to_string()).unwrap();
    /// let requires_two_factor_auth = false;
    /// let user = User::new(email, requires_two_factor_auth, password);
    /// ```
    pub fn new(email: Email, requires_2fa: bool, password: Password) -> Self {
        // Note: unwrap here is dangerous. It might be good to return
        User {
            email: email,
            requires_2fa,
            password: password,
        }
    }
}
