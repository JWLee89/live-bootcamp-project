use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    parse::Parseable,
    password::Password,
    user::User,
};
use color_eyre::eyre::{Context, Result};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Try hashing password prior to insertion
        let hashed_password =
            match compute_password_hash(String::from(user.password.as_ref())).await {
                Ok(password) => password,
                Err(e) => return Err(UserStoreError::UnexpectedError(e.into())),
            };
        // TODO: See if there is a better way to do this!
        let result = sqlx::query(
            r#"
            insert into USERS
            (email, password_hash, requires_2fa)
            values ($1, $2, $3)
            "#,
        )
        .bind(user.email.as_ref())
        .bind(&hashed_password)
        .bind(&user.requires_2fa)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(UserStoreError::UnexpectedError(e.into())),
        }
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let result: (String, String, bool) = sqlx::query_as(
            "
            SELECT email, password_hash, requires_2fa
            FROM users
            WHERE email = $1",
        )
        .bind(email.as_ref())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;
        let user = User::new(
            Email::parse(result.0)
                .wrap_err("Cannot parse email")
                .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
            result.2,
            Password::parse(result.1)
                .wrap_err("Cannot parse password")
                .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
        );
        Ok(user)
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let password_from_db: (String,) = sqlx::query_as(
            "
            SELECT password_hash
            FROM users
            WHERE email = $1
            ",
        )
        .bind(email.as_ref())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        // Compare with hashed password
        if let Ok(_) = verify_password_hash(password_from_db.0, password.as_ref().to_string()).await
        {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify password hash")
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current();
    let compute_password_hash_task = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
            // Err(eyre!("oh no!")) // New!
        })
    });
    compute_password_hash_task.await?
}
