use email_pass::Password;
use getset::Getters;
use named_ctor::NamedCtor;
use parse_display::{Display, FromStr};

use super::ApiKeyError;

pub type EncryptedApiKey = Password;

#[derive(Clone, Debug, FromStr, Display, NamedCtor, Eq, PartialEq, Getters, sqlx::FromRow)]
#[display("LibraryApi_{id}_{token}")]
#[getset(get = "pub")]
pub struct ApiKey {
    pub id: String,
    pub token: String,
}

impl ApiKey {
    pub fn encrypt_token(&self) -> Result<EncryptedApiKey, ApiKeyError> {
        EncryptedApiKey::from_raw(&self.token.to_string())
            .to_encrypt_default()
            .map_err(|e| ApiKeyError::Encryption(e.to_string()))
    }
}

#[async_trait::async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn find_encrypted(&self, id: &str) -> Result<Option<String>, ApiKeyError>;
    async fn save(&self, id: &str, key: &EncryptedApiKey) -> Result<(), ApiKeyError>;
    async fn delete(&self, id: &str) -> Result<(), ApiKeyError>;
}

#[cfg(test)]
#[test]
fn api_key_parsing_works() {
    use std::str::FromStr;

    let api_key = ApiKey {
        id: "tokenId".to_string(),
        token: "TOkenC0ntent".to_string(),
    };

    let api_key_str = api_key.to_string();
    let same_api_key = ApiKey::from_str(&api_key_str).expect("Error al convertir &str -> ApiKey");

    assert_eq!(same_api_key, api_key);
}
