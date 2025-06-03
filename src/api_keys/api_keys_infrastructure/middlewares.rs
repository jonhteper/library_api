use std::{borrow::Cow, str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::{StatusCode, header::AUTHORIZATION};
use serde_json::json;

use crate::api_keys::{api_keys_application::ApiKeyValidationService, api_keys_domain::ApiKey};

#[derive(Clone)]
pub struct AuthMiddleware {
    pub api_key_strategy: Arc<ApiKeyValidationStrategy>,
}

impl AuthMiddleware {
    pub async fn auth_from_header(&self, auth_header: Option<&str>) -> Result<(), AuthError> {
        match AuthHeader::from_header_value(auth_header) {
            AuthHeader::ApiKey(raw_key) => self.api_key_strategy.validate(raw_key).await,
            AuthHeader::None => Err(AuthError {
                message: "Se requiere autenticación. Usar formato 'ApiKey YOURKEY'".into(),
                code: StatusCode::UNAUTHORIZED,
            }),
        }
    }
}

enum AuthHeader<'a> {
    ApiKey(&'a str),
    None,
}

impl<'a> AuthHeader<'a> {
    pub fn from_header_value(header: Option<&'a str>) -> Self {
        match header {
            Some(header) if header.starts_with("ApiKey ") => {
                let raw_api_key = header.trim_start_matches("ApiKey ").trim();
                AuthHeader::ApiKey(raw_api_key)
            }
            _ => AuthHeader::None,
        }
    }
}

pub struct ApiKeyValidationStrategy {
    pub validator: Arc<ApiKeyValidationService>,
}

impl ApiKeyValidationStrategy {
    pub async fn validate(&self, raw_api_key: &str) -> Result<(), AuthError> {
        let key = ApiKey::from_str(raw_api_key).map_err(|e| AuthError {
            message: format!("ApiKey inválida: {}", e).into(),
            code: StatusCode::UNPROCESSABLE_ENTITY,
        })?;

        self.validator.validate(&key).await.map_err(|e| AuthError {
            message: format!("Error al validar ApiKey: {e}").into(),
            code: StatusCode::from(e.kind()),
        })
    }
}

pub struct AuthError {
    message: Cow<'static, str>,
    code: StatusCode,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = json!({
            "message": self.message,
        });

        (self.code, Json(body)).into_response()
    }
}

pub async fn api_key_middleware(req: Request, next: Next) -> Result<Response, Response> {
    // Extraer el header de autorización
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_middleware = AuthMiddleware::get_instance();

    // Validar la API key
    match auth_middleware.auth_from_header(auth_header).await {
        Ok(()) => {
            // Si la autenticación es exitosa, continuar con la solicitud
            Ok(next.run(req).await)
        }
        Err(err) => {
            // Si la autenticación falla, devolver el error
            Err(err.into_response())
        }
    }
}
