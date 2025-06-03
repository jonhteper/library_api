use sqlx::{Pool, Sqlite, SqlitePool, query, query_as};

use crate::api_keys::{
    ApiKeyError,
    api_keys_domain::{ApiKey, ApiKeyRepository, EncryptedApiKey},
};

#[derive(Clone)]
pub struct ApiKeySqliteRepository {
    pub pool: Pool<Sqlite>,
}

impl ApiKeySqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_table(&self) -> Result<(), ApiKeyError> {
        query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                token TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApiKeyError::DatabaseError(format!("Error al crear tabla api_keys: {}", e)))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for ApiKeySqliteRepository {
    async fn find_encrypted(&self, id: &str) -> Result<Option<String>, ApiKeyError> {
        let result = query_as::<_, ApiKey>("SELECT id, token FROM api_keys WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiKeyError::DatabaseError(format!("Error al buscar ApiKey: {}", e)))?;

        let encrypted_token = result.map(|api_key| api_key.token().clone());

        Ok(encrypted_token)
    }

    async fn save(&self, id: &str, key: &EncryptedApiKey) -> Result<(), ApiKeyError> {
        let encrypted = key.as_str();

        query(
            r#"
            INSERT INTO api_keys (id, token)
            VALUES (?, ?)
            "#,
        )
        .bind(id)
        .bind(encrypted)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiKeyError::DatabaseError(format!("Error al guardar ApiKey: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), ApiKeyError> {
        query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiKeyError::DatabaseError(format!("Error al eliminar ApiKey: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::api_keys::api_keys_domain::_ApiKey;

    use super::*;
    use email_pass::Password;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> ApiKeySqliteRepository {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite database");

        let repo = ApiKeySqliteRepository { pool };
        repo.init_table().await.expect("Failed to initialize table");

        repo
    }

    #[tokio::test]
    async fn test_save_and_find_api_key() {
        // Arrange
        let repo = setup_test_db().await;
        let id = "test_key_id";
        let key = ApiKey::from(_ApiKey {
            id: id.to_string(),
            token: "ExampleToken".to_string(),
        });

        let encrypted_token = key.encrypt_token().expect("Failed to encrypt token");

        // Act
        repo.save(id, &encrypted_token)
            .await
            .expect("Error al guardar API key");
        let result = repo
            .find_encrypted(id)
            .await
            .expect("Error al encontrar API key");

        // Assert
        assert!(result.is_some(), "API key not found");
        let encrypted_token_from_db =
            Password::from_encrypt(&result.unwrap()).expect("El token no está encriptado");

        let is_valid = encrypted_token_from_db
            .verify_from_raw(key.token())
            .expect("Failed to verify token");

        assert!(is_valid, "Error al verificar token");
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        // Arrange
        let repo = setup_test_db().await;
        let id = "key_to_delete";
        let key = ApiKey::from(_ApiKey {
            id: id.to_string(),
            token: "ExampleToken".to_string(),
        });

        let encrypted_token = key.encrypt_token().expect("Failed to encrypt token");

        // Act
        repo.save(id, &encrypted_token)
            .await
            .expect("Failed to save API key");
        repo.delete(id).await.expect("Failed to delete API key");

        // Assert
        let result = repo
            .find_encrypted(id)
            .await
            .expect("Failed to query API key");
        assert!(result.is_none(), "API key should have been deleted");
    }

    #[tokio::test]
    async fn test_find_nonexistent_key() {
        // Arrange
        let repo = setup_test_db().await;
        let nonexistent_id = "nonexistent_key";

        // Act
        let result = repo
            .find_encrypted(nonexistent_id)
            .await
            .expect("Failed to query API key");

        // Assert
        assert!(result.is_none(), "Should return None for nonexistent key");
    }
}
