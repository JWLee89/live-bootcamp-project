mod login;
mod logout;
mod signup;
mod verify_2fa;
mod verify_token;

use axum::{routing::post, Router};
use tower_http::services::ServeDir;

// re-export items from sub-modules
pub use login::*;
pub use logout::*;
pub use signup::*;
pub use verify_2fa::*;
pub use verify_token::*;

pub fn routes() -> Router {
    Router::new()
        .nest_service("/", ServeDir::new("assets"))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/verify-2fa", post(verify_2fa))
        .route("/verify-token", post(verify_token))
}
