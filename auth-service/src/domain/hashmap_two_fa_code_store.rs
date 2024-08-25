use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};
use color_eyre::eyre::Result;
use std::collections::HashMap;

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

// TODO: implement TwoFACodeStore for HashMapTwoFACodeStore
#[async_trait::async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    /// Add a Two Factor code to the store.
    ///
    /// ## Example
    ///
    /// ```
    /// // Imports
    /// use tokio_test;
    /// use auth_service::domain::data_stores::{TwoFACodeStore, TwoFACodeStoreError, TwoFACode, LoginAttemptId};
    /// use auth_service::domain::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
    /// use auth_service::domain::email::Email;
    /// use auth_service::domain::parse::Parseable;
    ///
    /// // Ingredients
    /// let mut code_store: HashMapTwoFACodeStore = Default::default();
    /// let login_attempt_id = LoginAttemptId::default();
    /// let code: TwoFACode = TwoFACode::default();
    /// let email = Email::parse("jaymo@gmail.com".to_string()).unwrap();
    ///
    /// tokio_test::block_on(async {
    ///   // Should be okay, since it does not exist
    ///   let add_result = code_store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await;
    ///   assert!(add_result.is_ok());
    ///
    ///   // Try again, we should fail
    ///   let add_result_fail = code_store.add_code(email, login_attempt_id, code).await;
    ///   assert!(add_result_fail.is_err());
    ///   if let Err(e) = add_result_fail {
    ///       assert_eq!(e, TwoFACodeStoreError::EmailAlreadyExists);
    ///   }
    /// })
    ///
    /// ```
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            return Err(TwoFACodeStoreError::EmailAlreadyExists);
        }
        // Guaranteed to always be None since we check before inserting
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    /// Remove a Two Factor code to the store.
    ///
    /// ## Example
    ///
    /// ```
    /// // Imports
    /// use tokio_test;
    /// use auth_service::domain::data_stores::{TwoFACodeStore, TwoFACodeStoreError, TwoFACode, LoginAttemptId};
    /// use auth_service::domain::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
    /// use auth_service::domain::email::Email;
    /// use auth_service::domain::parse::Parseable;
    ///
    /// // Ingredients
    /// let mut code_store: HashMapTwoFACodeStore = Default::default();
    /// let login_attempt_id = LoginAttemptId::default();
    /// let code: TwoFACode = TwoFACode::default();
    /// let email = Email::parse("jaymo@gmail.com".to_string()).unwrap();
    ///
    /// // Should return error if it does not exist
    /// tokio_test::block_on(async {
    /// // Try removing, should not work
    /// let remove_result = code_store.remove_code(&email).await;
    /// if let Err(e) = remove_result {
    ///     assert_eq!(e, TwoFACodeStoreError::LoginAttemptIdNotFound);
    /// } else {
    ///     panic!("Cannot remove a non-existing code from store.");
    /// }
    ///
    /// // Add code
    /// let add_result = code_store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await;
    /// assert!(add_result.is_ok());
    ///
    /// // Remove
    /// let remove_result = code_store.remove_code(&email).await;
    /// if let Err(e) = remove_result {
    ///     panic!("Should be able to remove code if email exists");
    /// }
    ///
    /// // Removing again should result in error
    /// let remove_result = code_store.remove_code(&email).await;
    /// if let Err(e) = remove_result {
    ///     assert_eq!(e, TwoFACodeStoreError::LoginAttemptIdNotFound);
    /// } else {
    ///     panic!("Cannot remove a non-existing code from store.");
    /// }
    ///
    /// # })
    /// ```
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        if let Some(_) = self.codes.remove(&email) {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    /// Get a Two Factor code object from the store.
    ///
    /// ## Example
    ///
    /// ```
    /// // Imports
    /// use tokio_test;
    /// use auth_service::domain::data_stores::{TwoFACodeStore, TwoFACodeStoreError, TwoFACode, LoginAttemptId};
    /// use auth_service::domain::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
    /// use auth_service::domain::email::Email;
    /// use auth_service::domain::parse::Parseable;
    ///
    /// // Ingredients
    /// let mut code_store: HashMapTwoFACodeStore = Default::default();
    /// let login_attempt_id = LoginAttemptId::default();
    /// let code: TwoFACode = TwoFACode::default();
    /// let email = Email::parse("jaymo@gmail.com".to_string()).unwrap();
    ///
    /// // Should return error if it does not exist
    /// tokio_test::block_on(async {
    /// // Get code: should fail since it does not exist
    /// let get_result = code_store.get_code(&email).await;
    /// if let Err(e) = get_result {
    ///     assert_eq!(e, TwoFACodeStoreError::LoginAttemptIdNotFound);
    /// } else {
    ///     panic!("get_code() should fail if Login attempt has not been made");
    /// }
    ///
    /// // Add code
    /// let add_result = code_store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await;
    /// assert!(add_result.is_ok());
    ///
    /// let get_result = code_store.get_code(&email).await;
    /// if let Ok((map_id, map_code)) = get_result {
    ///     assert_eq!(&map_id, &login_attempt_id);
    ///     assert_eq!(&map_code, &code);
    /// } else {
    ///     panic!("get_code() should not fail since the login attempt has been made.");
    /// }
    /// # })
    /// ```
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        if let Some((id, code)) = self.codes.get(&email) {
            // Is this the best idea? cloning each time when getting can be quite costly.
            Ok((id.to_owned(), code.to_owned()))
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }
}
