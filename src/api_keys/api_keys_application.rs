use std::sync::Arc;

use email_pass::Password;
use prefixed_api_key::PrefixedApiKeyController;

use super::{
    ApiKeyError,
    api_keys_domain::{_ApiKey, ApiKey, ApiKeyRepository},
};

pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    pub fn generate() -> Result<ApiKey, ApiKeyError> {
        let controller = PrefixedApiKeyController::configure()
            .prefix("LibraryApi".to_string())
            .seam_defaults()
            .short_token_length(6)
            .long_token_length(25)
            .finalize()?;

        let key = controller
            .try_generate_key()
            .map_err(|e| ApiKeyError::Generation(e.to_string()))?;

        let api_key = ApiKey::from(_ApiKey {
            id: key.short_token().to_string(),
            token: key.long_token().to_string(),
        });

        Ok(api_key)
    }
}

#[derive(Clone)]
pub struct ApiKeyValidationService {
    pub repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyValidationService {
    /// Trata de obtener la ApiKey con base en el id y verificar si es válida
    pub async fn validate(&self, api_key: &ApiKey) -> Result<(), ApiKeyError> {
        let raw_encrypted = self
            .repo
            .find_encrypted(api_key.id())
            .await?
            .ok_or(ApiKeyError::NotFound)?;

        let encrypted = Password::from_encrypt(&raw_encrypted)
            .map_err(|e| ApiKeyError::BadEncryption(e.to_string()))?;

        let is_valid = encrypted
            .verify_from_raw(api_key.token())
            .map_err(|e| ApiKeyError::HashVerification(e.to_string()))?;

        if !is_valid {
            return Err(ApiKeyError::Invalid);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ApiKeyCreationService {
    pub repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyCreationService {
    /// Crea una nueva ApiKey y la guarda en la base de datos
    pub async fn create(&self) -> Result<ApiKey, ApiKeyError> {
        let api_key = ApiKeyGenerator::generate()?;
        let encrypted = api_key.encrypt_token()?;

        self.repo.save(api_key.id(), &encrypted).await?;

        Ok(api_key)
    }
}

#[derive(Clone)]
pub struct ApiKeyDeletionService {
    pub repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyDeletionService {
    /// Elimina una ApiKey de la base de datos
    pub async fn delete(&self, id: &str) -> Result<(), ApiKeyError> {
        self.repo.delete(id).await?;

        Ok(())
    }
}
