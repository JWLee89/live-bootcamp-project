use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

// Define a lazily evaluated static.
// lazy_static is needed because std_env::var is not a const function.
lazy_static! {
    pub static ref JWT_SECRET: String = get_token();
    pub static ref POSTGRES_PASSWORD: String = get_postgres_password();
    pub static ref DATABASE_URL: String = get_database_url();
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

///
fn get_database_url() -> String {
    retrieve_dot_env_variable(String::from(env::DATABASE_URL))
}

fn get_postgres_password() -> String {
    retrieve_dot_env_variable(String::from(env::POSTGRES_PASSWORD_ENV_VAR))
}

fn get_token() -> String {
    retrieve_dot_env_variable(String::from(env::JWT_SECRET_ENV_VAR))
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const POSTGRES_PASSWORD_ENV_VAR: &str = "POSTGRESS_PASSWORD";
    pub const DATABASE_URL: &str = "DATABASE_URL";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
