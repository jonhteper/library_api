use std::sync::Arc;
use uuid::Uuid;

use log::info;

use crate::books::{
    BookError,
    books_domain::{Book, BookRepository, ReadBookCriteria},
};

#[derive(Clone)]
pub struct GetBookByIdUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl GetBookByIdUseCase {
    /// Obtiene un libro por su ID
    pub async fn get_book_by_id(&self, id: Uuid) -> Result<Book, BookError> {
        info!("Buscando libro con ID: {}", id);

        let book = self
            .repo
            .find(ReadBookCriteria::ById(id))
            .await?
            .ok_or(BookError::NotFound)?;

        Ok(book)
    }
}
