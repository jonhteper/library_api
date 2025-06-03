use std::borrow::Cow;

use validator::ValidationErrors;

use crate::AppErrorKind;

#[derive(Debug, Clone, thiserror::Error, web_proc_macros::ErrorKind)]
#[error_kind(ApiKeyErrorKind)]
pub enum BookError {
    #[error("Libro no encontrado")]
    #[error_kind(AppErrorKind, NotFound)]
    NotFound,

    #[error("Cadena de texto vacía")]
    #[error_kind(AppErrorKind, InvalidData)]
    EmptyString,

    #[error("Caracteres inválidos")]
    #[error_kind(AppErrorKind, InvalidData)]
    InvalidCharacters,

    #[error("ISBN inválido")]
    #[error_kind(AppErrorKind, InvalidData)]
    InvalidIsbn,

    #[error("Validación fallida")]
    #[error_kind(AppErrorKind, InvalidData)]
    Validation(#[from] ValidationErrors),

    #[error("Libro con el ISBN: {0} ya existe")]
    #[error_kind(AppErrorKind, InvalidData)]
    AlreadyExists(String),

    #[error("Error de base de datos")]
    #[error_kind(AppErrorKind, Infrastructure)]
    DatabaseError(Cow<'static, str>),
}
