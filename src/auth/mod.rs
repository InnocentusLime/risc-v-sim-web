#[cfg(feature = "github_authentication")]
pub mod github_authentication;
#[cfg(feature = "noop_authentication")]
pub mod noop_authentication;

use axum::{Router, extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tower_sessions::Session;

use crate::AppState;

pub fn auth_routes() -> Router<Arc<AppState>> {
    #[allow(unused_mut)]
    let mut res = Router::new();

    #[cfg(feature = "noop_authentication")]
    {
        res = res.nest("/noop", noop_authentication::routes());
    }

    #[cfg(feature = "github_authentication")]
    {
        res = res.nest("/github", github_authentication::routes());
    }

    res
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub id: ulid::Ulid,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            id: ulid::Ulid::nil(),
        }
    }
}

pub struct User {
    session: Session,
    user_data: UserData,
}

impl User {
    const GUEST_DATA_KEY: &'static str = "user.data";

    pub fn id(&self) -> ulid::Ulid {
        self.user_data.id
    }

    #[allow(dead_code)]
    async fn set_user_id(&mut self, id: ulid::Ulid) {
        self.user_data.id = id;
        Self::update_session(&self.session, &self.user_data).await
    }

    async fn update_session(session: &Session, guest_data: &UserData) {
        session
            .insert(Self::GUEST_DATA_KEY, guest_data.clone())
            .await
            .unwrap()
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User").field("id", &self.id()).finish()
    }
}

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        let user_data: UserData = session
            .get(Self::GUEST_DATA_KEY)
            .await
            .unwrap()
            .unwrap_or_default();

        Self::update_session(&session, &user_data).await;

        Ok(Self { session, user_data })
    }
}
