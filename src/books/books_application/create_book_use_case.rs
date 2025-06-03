use std::{str::FromStr, sync::Arc};

use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::books::{
    BookError,
    books_domain::{
        Book, BookAuthor, BookPublisher, BookRepository, BookTitle, Isbn, ReadBookCriteria,
    },
};

#[derive(Clone)]
pub struct CreateBookUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl CreateBookUseCase {
    #[inline]
    async fn check_collition(&self, isbn: Isbn) -> Result<(), BookError> {
        let saved_book = self
            .repo
            .find(ReadBookCriteria::ByIsbn(isbn.canonical()))
            .await?;

        if saved_book.is_some() {
            Err(BookError::AlreadyExists(isbn.to_string()))?;
        }

        Ok(())
    }

    /// Guarda un libro en la base de datos, previene el uso de ISBN duplicado
    pub async fn create_book(&self, dto: CreateBookDto) -> Result<Uuid, BookError> {
        let book_id = Uuid::new_v4();
        let book = Book::try_from((book_id, dto))?;

        self.check_collition(book.isbn.clone()).await?;
        info!("Creando nuevo libro: {}", book.isbn.canonical());

        self.repo.create(book).await?;

        Ok(book_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookDto {
    pub title: String,
    pub authors: Vec<String>,
    pub publisher: String,
    pub year: u16,
    pub isbn: String,
    pub stored_quantity: u16,
}

impl TryFrom<(Uuid, CreateBookDto)> for Book {
    type Error = BookError;

    fn try_from((id, dto): (Uuid, CreateBookDto)) -> Result<Self, Self::Error> {
        let authors = dto
            .authors
            .into_iter()
            .map(|author| BookAuthor::from_str(&author))
            .collect::<Result<Vec<BookAuthor>, _>>()?;

        let book = Book {
            id,
            title: BookTitle::from_str(&dto.title)?,
            authors,
            publisher: BookPublisher::from_str(&dto.publisher)?,
            year: dto.year,
            isbn: Isbn::from_str(&dto.isbn)?,
            stored_quantity: dto.stored_quantity,
        };

        book.validate()?;

        Ok(book)
    }
}
