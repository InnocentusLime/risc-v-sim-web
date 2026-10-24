use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use std::sync::Arc;

use super::User;

use crate::AppState;
use crate::auth::UserData;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(noop_login))
        .route("/register", post(noop_register))
}

async fn noop_login(mut user: User, Query(user_params): Query<UserData>) -> impl IntoResponse {
    user.set_user_id(user_params.id).await;
    Redirect::to("/")
}

async fn noop_register(
    State(state): State<Arc<AppState>>,
    mut user: User,
    Query(user_params): Query<UserData>,
) -> impl IntoResponse {
    let id = user_params.id;
    let str = id.to_string();
    if let Err(e) = state.db.create_user(id, &str).await {
        tracing::error!("error registering: {e:#}")
    } else {
        tracing::info!("new user: {id} ({str:?})");
        user.set_user_id(user_params.id).await;
    }
    Redirect::to("/")
}
