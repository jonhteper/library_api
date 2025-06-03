use std::sync::Arc;

use log::info;
use uuid::Uuid;
use validator::Validate;

use crate::books::{
    BookError,
    books_domain::{Book, BookRepository, Isbn, ReadBookCriteria},
};

#[derive(Clone)]
pub struct UpdateBookUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl UpdateBookUseCase {
    /// Evita que dos libros tengan el mismo ISBN
    #[inline]
    async fn check_collision(&self, id: Uuid, isbn: &Isbn) -> Result<(), BookError> {
        let saved_book = self
            .repo
            .find(ReadBookCriteria::ByIsbn(isbn.canonical()))
            .await?;

        if let Some(book) = saved_book {
            if book.id != id {
                return Err(BookError::AlreadyExists(isbn.to_string()));
            }
        }

        Ok(())
    }

    /// Actualiza un libro existente en la base de datos
    pub async fn update_book(&self, book: Book) -> Result<Book, BookError> {
        book.validate()?;

        self.check_collision(book.id, &book.isbn).await?;

        info!(
            "Actualizando libro ID: {}, ISBN: {}",
            book.id,
            book.isbn.canonical()
        );

        self.repo.update(book.clone()).await?;

        Ok(book)
    }
}
