use std::str::FromStr;

use sqlx::FromRow;
use uuid::Uuid;

use crate::books::{
    BookError,
    books_domain::{Book, BookAuthor, BookPublisher, BookTitle, Isbn},
};

/// DTO para la tabla books
#[derive(Debug, FromRow)]
pub struct DbBook {
    pub id: String,
    pub title: String,
    pub publisher: String,
    pub year: i64,
    pub isbn: String,
    pub stored: i64,
}

impl DbBook {
    pub async fn to_domain_with_authors(
        &self,
        authors: Vec<BookAuthor>,
    ) -> Result<Book, BookError> {
        let id = Uuid::try_from(self.id.clone()).map_err(|e| {
            BookError::DatabaseError(format!("Error al crear BookId: {:?}", e).into())
        })?;

        let title = BookTitle::from_str(&self.title).map_err(|e| {
            BookError::DatabaseError(format!("Error al crear BookTitle: {:?}", e).into())
        })?;

        let publisher = BookPublisher::from_str(&self.publisher).map_err(|e| {
            BookError::DatabaseError(format!("Error al crear BookPublisher: {:?}", e).into())
        })?;

        let isbn = Isbn::from_str(&self.isbn).map_err(|e| {
            BookError::DatabaseError(format!("Error al crear Isbn: {:?}", e).into())
        })?;

        let book = Book {
            id,
            title,
            authors,
            publisher,
            year: self.year as u16,
            isbn,
            stored_quantity: self.stored as u16,
        };

        Ok(book)
    }
}

/// DTO para la tabla authors
#[derive(Debug, FromRow)]
pub struct DbAuthor {
    pub id: i64,
    pub name: String,
}

/// DTO para obtener solo el nombre del autor en consultas
#[derive(Debug, FromRow)]
pub struct DbAuthorName {
    pub name: String,
}

impl DbAuthorName {
    pub fn to_domain(&self) -> Result<BookAuthor, BookError> {
        BookAuthor::try_from(self.name.clone()).map_err(|e| {
            BookError::DatabaseError(format!("Error al crear BookAuthor: {:?}", e).into())
        })
    }
}
