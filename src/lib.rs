pub mod api;
pub mod auth;
pub mod db;
pub mod model;
pub mod submission_actor;

use anyhow::Context;
use axum::{Router, body::Body, http::Request, routing::get};
use sqlx::ConnectOptions;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::{join, net::TcpListener};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::db::DbClient;
use crate::submission_actor::SubmissionActor;
use submission_actor::SubmissionTask;

pub struct Config {
    /* Submission processing configuration */
    /// Path to the assembler program, capable of producing RiscV object code.
    pub as_binary: PathBuf,
    /// Path to the linker program, capable of producing RiscV elfs.
    pub ld_binary: PathBuf,
    /// Path to a compiler risc-v-sim.
    pub simulator_binary: PathBuf,
    /// Path to a folder, that will be used to store submission artifacts.
    pub submissions_folder: PathBuf,
    /// Max of amount of risc-v-sim allowed.
    pub ticks_max: u32,
    /// Max size of the upload in bytes.
    pub codesize_max: u32,

    #[cfg(feature = "github_authentication")]
    pub github_authentication: auth::github_authentication::AuthConfig,
}

pub struct AppState {
    pub ticks_max: u32,
    pub codesize_max: u32,
    pub submissions_folder: PathBuf,
    pub db: Arc<DbClient>,
    pub task_send: Sender<SubmissionTask>,

    #[cfg(feature = "github_authentication")]
    pub github_authentication: auth::github_authentication::AuthState,
}

pub async fn health_handler() -> &'static str {
    "Ok"
}

pub async fn run(
    root_span: tracing::Span,
    listener: TcpListener,
    cfg: Config,
) -> anyhow::Result<()> {
    let conn_fut = sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::new_without_pgpass()
            .log_statements(log::LevelFilter::Debug),
    );
    let conn = tokio::time::timeout(Duration::from_secs(5), conn_fut)
        .await
        .map_err(anyhow::Error::from)
        .map(|x| x.map_err(anyhow::Error::from))
        .flatten()
        .context("Connect to DB")?;

    let (task_send, task_recv) = tokio::sync::mpsc::channel::<SubmissionTask>(100);
    let db_client = DbClient::new(conn).await;

    let state = Arc::new(AppState {
        db: Arc::new(db_client),
        task_send,
        ticks_max: cfg.ticks_max,
        codesize_max: cfg.codesize_max,
        submissions_folder: cfg.submissions_folder.clone(),

        #[cfg(feature = "github_authentication")]
        github_authentication: auth::github_authentication::AuthState::new(
            cfg.github_authentication,
        ),
    });

    let submission_actor = SubmissionActor::new(
        task_recv,
        state.db.clone(),
        cfg.as_binary,
        cfg.ld_binary,
        cfg.simulator_binary,
        cfg.submissions_folder,
    );

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
    let router = Router::new()
        .nest("/api", api::api_routes().with_state(state.clone()))
        .nest("/auth", auth::auth_routes().with_state(state.clone()))
        .route("/health", get(health_handler))
        .fallback_service(ServeDir::new("static"))
        .layer(ServiceBuilder::new().layer(tower_http::cors::CorsLayer::permissive()))
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http().make_span_with(move |request: &Request<Body>| {
                tracing::debug_span!(
                    parent: &root_span,
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            }),
        );

    let (res, _) = join!(axum::serve(listener, router), submission_actor.run(),);
    res.map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response, "Ok");
    }
}
