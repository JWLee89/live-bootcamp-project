use std::collections::HashMap;

use crate::domain::user::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    InvalidPassword,
    UnexpectedError,
}

#[derive(Debug, PartialEq, Default)]
pub struct HashMapUserStore {
    user_store: HashMap<String, User>,
}

impl HashMapUserStore {
    pub fn count(&self) -> usize {
        self.user_store.keys().len()
    }

    fn user_already_exists(&self, email: &str) -> Result<(), UserStoreError> {
        if self.user_store.contains_key(email) {
            return Err(UserStoreError::UserAlreadyExists)
        }
        Ok(())
    }
        

    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Not the prettiest but it will work for now.
        // I assuming that this will be refactored in the future.
        self.user_already_exists(&user.email)?;
        self.user_store.insert(user.email.clone(), user);
        Ok(())
    }

    /// Return the cloned version of the user if it exists. 
    /// Otherwise, we return an Error
    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.user_store.get(email).cloned().ok_or_else(|| UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.user_store.get(email).ok_or(UserStoreError::UserNotFound)?;
        if &user.password != password {
            return Err(UserStoreError::InvalidPassword);
        }
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;
    use test_case::test_case;

    // Would be cool if we could re-use this object 
    fn empty_hashmap_user_store() -> HashMapUserStore {
        let store = HashMapUserStore::default();
        assert_eq!(store.count(), 0);
        store
    }

    #[test_case("username".to_owned(), "test_email".to_owned(), false)]
    #[test_case("new_user".to_owned(), "teemo@gmail.com".to_owned(), true)]
    fn test_add_user(username: String, email: String, requires_2fa: bool) {
        // TODO add username
        let user = User::new(email.clone(), requires_2fa, None);
        let mut user_store = empty_hashmap_user_store();
        assert_eq!(user_store.count(), 0);
        user_store.add_user(user.clone()).expect("Failed to add user");
        assert_eq!(user_store.count(), 1);
        // This should fail since user already exists
        let failed_insert: Result<(), UserStoreError> = user_store.add_user(user);
        let expected_error: Result<(), UserStoreError> = Err(UserStoreError::UserAlreadyExists);
        assert_eq!(failed_insert, expected_error);

    }
    
    #[test_case("username".to_owned(), "test_email".to_owned())]
    #[test_case("Captain teemo".to_string(), "email@hotmail.com".to_string())]
    fn test_get_user(username: String, email: String) {
        let user = User::new(email.clone(), false, None);
        let mut user_store = empty_hashmap_user_store();

        // Case 1. Should fail if user does not exist
        let error = user_store.get_user(email.as_str());
        let expected_error = Err(UserStoreError::UserNotFound);
        assert_eq!(error, expected_error);
        
        // Case 2. Added user and passed in user should be the same.
        user_store.add_user(user.clone()).expect("Failed to add user");
        let same_user = user_store.get_user(email.as_str()).unwrap();
        assert_eq!(user, same_user);
    }

    #[test_case("username".to_owned(), "test_email".to_owned())]
    #[test_case("Captain teemo".to_string(), "email@hotmail.com".to_string())]
    fn test_validate_user(username: String, email: String) {
        let user = User::new(email.clone(), false, None);
        let mut user_store = empty_hashmap_user_store();
        let wrong_password = "wrongPassword";
        // Case 1. UserStoreError::UserNotFound 
        let error = user_store.validate_user(email.as_str(), &wrong_password);
        assert_eq!(error, Err(UserStoreError::UserNotFound));
        
        // Case 2. UserStoreError::InvalidPassword
        user_store.add_user(user.clone()).unwrap();
        // Check to see whether password does not equal wrong password
        assert_ne!(user.password, wrong_password);
        let error = user_store.validate_user(email.as_str(), &wrong_password);
        assert_eq!(error, Err(UserStoreError::InvalidPassword));

        // Case 3: Ok
        let is_ok = user_store.validate_user(email.as_str(), &user.password).is_ok();
        assert_eq!(is_ok, true);
    }
}
