use std::borrow::Cow;

use axum::{Json, response::IntoResponse};
use http::StatusCode;
use serde_json::json;

use crate::{api_keys::ApiKeyError, books::BookError};

#[derive(Debug, Clone, thiserror::Error, web_proc_macros::ErrorKind)]
#[error_kind(AppErrorKind)]
pub enum AppError {
    #[error("Error: {0}")]
    #[error_kind(AppErrorKind, Unknown)]
    Custom(Cow<'static, str>),

    #[error("Error con ApiKey: {0}")]
    #[error_kind(transparent)]
    ApiKey(#[from] ApiKeyError),

    #[error("Error al manejar libro: {0}")]
    #[error_kind(transparent)]
    Book(#[from] BookError),

    #[error("Error al cargar configuración: {0}")]
    #[error_kind(AppErrorKind, Infrastructure)]
    ConfigLoad(String),

    #[error("Error al cargar variable de entorno: {0}")]
    #[error_kind(AppErrorKind, Infrastructure)]
    EnvVarLoad(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AppErrorKind {
    /// Se necesita autenticación para realizar la acción
    Auhtentication,

    /// El recurso no existe
    NotFound,

    /// El recurso ya existe
    Duplicate,

    /// Los datos ingresados no son válidos
    InvalidData,

    /// Fallo en capa de infraestructura
    Infrastructure,

    /// Fallo en capa de aplicación
    Application,

    /// Error desconocido
    Unknown,
}

impl From<AppErrorKind> for StatusCode {
    fn from(value: AppErrorKind) -> Self {
        match value {
            AppErrorKind::Auhtentication => StatusCode::UNAUTHORIZED,
            AppErrorKind::NotFound => StatusCode::NOT_FOUND,
            AppErrorKind::Duplicate => StatusCode::CONFLICT,
            AppErrorKind::InvalidData => StatusCode::BAD_REQUEST,
            AppErrorKind::Infrastructure | AppErrorKind::Application | AppErrorKind::Unknown => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from(self.kind());
        let body = Json(json!({
            "message": self.to_string()
        }));
        (status, body).into_response()
    }
}
