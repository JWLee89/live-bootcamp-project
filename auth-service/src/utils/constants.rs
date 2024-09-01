use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

// Define a lazily evaluated static.
// lazy_static is needed because std_env::var is not a const function.
lazy_static! {
    pub static ref JWT_SECRET: String = get_token();
    pub static ref DATABASE_URL: String = get_database_url();
    pub static ref REDIS_HOST_NAME: String = get_redis_host();
    pub static ref POSTMARK_AUTH_TOKEN: Secret<String> = get_postmark_auth_token(); // New!
}

/// Add a variable key
fn retrieve_dot_env_variable(variable_key: String) -> String {
    dotenv().ok();
    let value =
        std_env::var(&variable_key).expect(format!("{} must be set", variable_key).as_str());
    if value.is_empty() {
        panic!("{}", format!("{} must not be empty", variable_key).as_str())
    }
    value
}

fn get_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

fn get_postmark_auth_token() -> Secret<String> {
    Secret::new(retrieve_dot_env_variable(String::from(
        env::POSTMARK_AUTH_TOKEN,
    )))
}

fn get_database_url() -> String {
    retrieve_dot_env_variable(String::from(env::DATABASE_URL))
}

fn get_token() -> String {
    retrieve_dot_env_variable(String::from(env::JWT_SECRET_ENV_VAR))
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const POSTMARK_AUTH_TOKEN: &str = "POSTMARK_AUTH_TOKEN";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
// Needed to get it working in production
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/email";
        // If you created your own Postmark account, make sure to use your email address!
        pub const SENDER: &str = "bogdan@codeiron.io";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}
