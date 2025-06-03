use std::sync::{Arc, LazyLock};

use proc_singleton::singleton_from_static_arc;

use crate::{
    api_keys::{
        api_keys_application::{
            ApiKeyCreationService, ApiKeyDeletionService, ApiKeyValidationService,
        },
        api_keys_infrastructure::AuthMiddleware,
    },
    init::get_pool,
};

use super::{ApiKeySqliteRepository, ApiKeyValidationStrategy};

#[singleton_from_static_arc(AuthMiddleware)]
static AUTH_MIDDLEWARE: LazyLock<Arc<AuthMiddleware>> = LazyLock::new(|| {
    Arc::new(AuthMiddleware {
        api_key_strategy: ApiKeyValidationStrategy::get_instance(),
    })
});

#[singleton_from_static_arc(ApiKeyValidationStrategy)]
static API_KEY_STRATEGY: LazyLock<Arc<ApiKeyValidationStrategy>> = LazyLock::new(|| {
    Arc::new(ApiKeyValidationStrategy {
        validator: ApiKeyValidationService::get_instance(),
    })
});

#[singleton_from_static_arc(ApiKeyValidationService)]
static API_KEY_VALIDATOR: LazyLock<Arc<ApiKeyValidationService>> = LazyLock::new(|| {
    Arc::new(ApiKeyValidationService {
        repo: ApiKeySqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(ApiKeySqliteRepository)]
static API_KEY_REPOSITORY: LazyLock<Arc<ApiKeySqliteRepository>> =
    LazyLock::new(|| Arc::new(ApiKeySqliteRepository { pool: get_pool() }));

#[singleton_from_static_arc(ApiKeyCreationService)]
static API_KEY_CREATION_SERVICE: LazyLock<Arc<ApiKeyCreationService>> = LazyLock::new(|| {
    Arc::new(ApiKeyCreationService {
        repo: ApiKeySqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(ApiKeyDeletionService)]
static API_KEY_DELETION_SERVICE: LazyLock<Arc<ApiKeyDeletionService>> = LazyLock::new(|| {
    Arc::new(ApiKeyDeletionService {
        repo: ApiKeySqliteRepository::get_instance(),
    })
});
