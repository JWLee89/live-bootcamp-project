use uuid::Uuid;


/// Represents a new user
#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub email: String,
    pub requires_2fa: bool,
    pub password: String,
}

impl User {
    /// Create a new user
    /// ```
    /// use crate::auth_service::domain::user::User;
    /// let email = "cowbell@email.com".to_string();
    /// let requires_two_factor_auth = false;
    /// let user = User::new(email, requires_two_factor_auth, None);
    /// ```
    pub fn new(
               email: String, 
               requires_2fa: bool, 
               password: Option<String>) -> Self {
        let new_password = match password {
            Some(password) => password,
            None => Uuid::new_v4().to_string(),
        };
        User {
            email,
            requires_2fa,
            password: new_password
        }
    }
}
