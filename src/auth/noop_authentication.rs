use axum::Router;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use std::sync::Arc;

use super::User;

use crate::AppState;
use crate::auth::UserData;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/login", post(noop_login))
}

async fn noop_login(mut user: User, Query(user_params): Query<UserData>) -> impl IntoResponse {
    user.set_user_id(user_params.id).await;
    Redirect::to("/")
}
