use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::User,
};

#[derive(Debug, PartialEq, Default)]
pub struct HashMapUserStore {
    user_store: HashMap<Email, User>,
}

impl HashMapUserStore {
    pub fn count(&self) -> usize {
        self.user_store.keys().len()
    }

    fn user_already_exists(&self, email: &Email) -> Result<(), UserStoreError> {
        if self.user_store.contains_key(email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl UserStore for HashMapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Not the prettiest but it will work for now.
        // I assuming that this will be refactored in the future.
        self.user_already_exists(&user.email)?;
        self.user_store.insert(user.email.clone(), user);
        Ok(())
    }

    /// Return the cloned version of the user if it exists.
    /// Otherwise, we return an Error
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.user_store
            .get(email)
            .cloned()
            .ok_or_else(|| UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self
            .user_store
            .get(email)
            .ok_or(UserStoreError::UserNotFound)?;
        if user.password.as_ref() != password.as_ref() {
            return Err(UserStoreError::InvalidPassword);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{parse::Parseable, password::Password, user::User};
    use test_case::test_case;

    // Would be cool if we could re-use this object
    fn empty_hashmap_user_store() -> HashMapUserStore {
        let store = HashMapUserStore::default();
        assert_eq!(store.count(), 0);
        store
    }

    #[test_case("test_email@google.com".to_owned(), false)]
    #[test_case("teemo@gmail.com".to_owned(), true)]
    #[tokio::test]
    async fn test_add_user(email: String, requires_2fa: bool) {
        // TODO add username
        let user = User::new(
            Email::parse(email.clone()).unwrap(),
            requires_2fa,
            Password::parse("captain teemo".to_string()).unwrap(),
        );
        let mut user_store = empty_hashmap_user_store();
        assert_eq!(user_store.count(), 0);
        user_store
            .add_user(user.clone())
            .await
            .expect("Failed to add user");
        assert_eq!(user_store.count(), 1);
        // This should fail since user already exists
        let failed_insert: Result<(), UserStoreError> = user_store.add_user(user).await;
        let expected_error: Result<(), UserStoreError> = Err(UserStoreError::UserAlreadyExists);
        assert_eq!(failed_insert, expected_error);
    }

    #[test_case(
        Email::parse("test_email@gmail.com".to_owned()).unwrap()
    )]
    #[test_case(
        Email::parse("email@hotmail.com".to_string()).unwrap()
    )]
    #[tokio::test]
    async fn test_get_user(email: Email) {
        let user = User::new(
            email.clone(),
            false,
            Password::parse("a valid password".to_string()).unwrap(),
        );
        let mut user_store = empty_hashmap_user_store();

        // Case 1. Should fail if user does not exist
        let error = user_store.get_user(&email).await;
        let expected_error = Err(UserStoreError::UserNotFound);
        assert_eq!(error, expected_error);

        // Case 2. Added user and passed in user should be the same.
        user_store
            .add_user(user.clone())
            .await
            .expect("Failed to add user");
        let same_user = user_store.get_user(&email).await.unwrap();
        assert_eq!(user, same_user);
    }

    #[test_case(
        Email::parse("some_password@gmail.com".to_string()).unwrap(),
        Password::parse("valid password".to_owned()).unwrap()
    )]
    #[test_case(
        Email::parse("email@hotmail.com".to_string()).unwrap(),
        Password::parse("valid password".to_owned()).unwrap()
    )]
    #[tokio::test]
    async fn test_validate_user(email: Email, password: Password) {
        let user = User::new(email.clone(), false, password);
        let mut user_store = empty_hashmap_user_store();
        let wrong_password = Password::parse("wrongPassword".to_string()).unwrap();

        // Case 1. UserStoreError::UserNotFound
        let error = user_store.validate_user(&email, &wrong_password).await;
        assert_eq!(error, Err(UserStoreError::UserNotFound));

        // Case 2. UserStoreError::InvalidPassword
        user_store.add_user(user.clone()).await.unwrap();
        // Check to see whether password does not equal wrong password
        assert_ne!(user.password.as_ref(), wrong_password.as_ref());
        let error = user_store.validate_user(&email, &wrong_password).await;
        assert_eq!(error, Err(UserStoreError::InvalidPassword));

        // Case 3: Ok
        let is_ok = user_store
            .validate_user(&email, &user.password)
            .await
            .is_ok();
        assert_eq!(is_ok, true);
    }
}
