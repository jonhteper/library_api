use crate::AppErrorKind;

#[derive(Debug, Clone, thiserror::Error, web_proc_macros::ErrorKind)]
#[error_kind(ApiKeyErrorKind)]
pub enum ApiKeyError {
    #[error("Error al encriptar ApiKey: {0}")]
    #[error_kind(AppErrorKind, Application)]
    Encryption(String),

    #[error("Error al configurar ApiKey: {0}")]
    #[error_kind(AppErrorKind, Application)]
    Configuration(#[from] prefixed_api_key::BuilderError),

    #[error("Error al generar ApiKey: {0}")]
    #[error_kind(AppErrorKind, Application)]
    Generation(String),

    #[error("ApiKey no encontrado")]
    #[error_kind(AppErrorKind, NotFound)]
    NotFound,

    #[error("La ApiKey no fue correctamente encriptada: {0}")]
    #[error_kind(AppErrorKind, Infrastructure)]
    BadEncryption(String),

    #[error("Error al verificar hash de la ApiKey: {0}")]
    #[error_kind(AppErrorKind, Application)]
    HashVerification(String),

    #[error("ApiKey no válida")]
    #[error_kind(AppErrorKind, Application)]
    Invalid,

    #[error("Error al interactuar con la base de datos: {0}")]
    #[error_kind(AppErrorKind, Infrastructure)]
    DatabaseError(String),
}
