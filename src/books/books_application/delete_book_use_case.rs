use std::sync::Arc;
use uuid::Uuid;

use log::info;

use crate::books::{
    BookError,
    books_domain::{BookRepository, ReadBookCriteria},
};

#[derive(Clone)]
pub struct DeleteBookUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl DeleteBookUseCase {
    #[inline]
    async fn check_exists(&self, id: Uuid) -> Result<(), BookError> {
        let _book = self
            .repo
            .find(ReadBookCriteria::ById(id))
            .await?
            .ok_or(BookError::NotFound)?;

        Ok(())
    }

    /// Elimina un libro de la base de datos por su ID
    pub async fn delete_book(&self, id: Uuid) -> Result<(), BookError> {
        // Verificar que el libro exista antes de eliminarlo
        self.check_exists(id).await?;

        info!("Eliminando libro con ID: {}", &id);
        self.repo.delete(&id).await
    }
}
