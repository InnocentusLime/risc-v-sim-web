use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

pub type ApiResult<T> = Result<Json<T>, ApiError>;

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        ApiError::InternalError(value)
    }
}

pub enum ApiError {
    InternalError(anyhow::Error),
    BadRequest(anyhow::Error),
    SubmissionNotFound(ulid::Ulid),
    SubmissionSourceNotFound(ulid::Ulid),
    SubmissionTraceNotFound(ulid::Ulid),
    Unauthorized,
}

impl ApiError {
    pub fn http_status(&self) -> StatusCode {
        match self {
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::SubmissionNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::SubmissionSourceNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::SubmissionTraceNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::InternalError(_) => "internal_error",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::SubmissionNotFound(_) => "submission_not_found",
            ApiError::SubmissionSourceNotFound(_) => "submission_source_not_found",
            ApiError::SubmissionTraceNotFound(_) => "submission_trace_not_found",
            ApiError::Unauthorized => "unauthorized",
        }
    }

    pub fn error_message(&self) -> String {
        match self {
            ApiError::InternalError(error) => format!("{error:#}"),
            ApiError::BadRequest(error) => format!("{error:#}"),
            ApiError::SubmissionNotFound(id) => format!("Submission {id} not found"),
            ApiError::SubmissionSourceNotFound(id) => format!("Submission {id} source not found"),
            ApiError::SubmissionTraceNotFound(id) => format!("Submission {id} source not found"),
            ApiError::Unauthorized => String::from("Unauthorized request"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.error_code();
        let err = self.error_message();
        tracing::error!(code, "error: {err}");

        let body = Json(ApiErrorResponse { code, err });

        (self.http_status(), body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: &'static str,
    pub err: String,
}
