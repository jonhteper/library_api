use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::books::BookError;

use super::{Isbn, ValidatedStr};

pub type BookTitle = ValidatedStr;
pub type BookAuthor = ValidatedStr;
pub type BookPublisher = ValidatedStr;

/// Representa un libro en la librería
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
pub struct Book {
    pub id: Uuid,

    pub title: BookTitle,

    #[validate(length(min = 1))]
    pub authors: Vec<BookAuthor>,

    pub publisher: BookPublisher,

    #[validate(range(min = 1900, max = 2100))]
    pub year: u16,

    pub isbn: Isbn,

    #[validate(range(min = 1, max = 1000))]
    pub stored_quantity: u16,
}

#[async_trait::async_trait]
pub trait BookRepository: Send + Sync {
    async fn find(&self, criteria: ReadBookCriteria) -> Result<Option<Book>, BookError>;
    async fn find_multiple(
        &self,
        criteria: ReadMultipleBooksCriteria,
        page: u32,
        page_size: u8,
    ) -> Result<PaginatedBooks, BookError>;
    async fn create(&self, book: Book) -> Result<(), BookError>;
    async fn update(&self, book: Book) -> Result<(), BookError>;
    async fn delete(&self, id: &Uuid) -> Result<(), BookError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadBookCriteria {
    ById(Uuid),
    ByIsbn(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadMultipleBooksCriteria {
    All,
    ByTitle(String),
    ByAuthor(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedBooks {
    pub books: Vec<Book>,
    pub total: u64,
    pub page: u32,
    pub page_size: u8,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn book_validation_works() {
        let book = Book {
            id: Uuid::new_v4(),
            title: BookTitle::from_str("Title").expect("Failed to create BookTitle"),
            authors: vec![BookAuthor::from_str("Author").expect("Failed to create BookAuthor")],
            publisher: BookPublisher::from_str("Publisher")
                .expect("Failed to create BookPublisher"),
            year: 2022,
            isbn: Isbn::from_str("ISBN-10 0-596-52068-9").expect("Failed to create Isbn"),
            stored_quantity: 10,
        };
        assert!(book.validate().is_ok());
    }

    #[test]
    fn test_book_title_validation_works() {
        // Título válido
        let valid_title = BookTitle::from_str("El Quijote");
        assert!(valid_title.is_ok());

        // Título con caracteres especiales válidos
        let valid_special_chars = BookTitle::from_str("Historia del Arte: Renacimiento & Barroco");
        assert!(valid_special_chars.is_ok());

        // Título con acentos
        let valid_accents = BookTitle::from_str("Crónica de una muerte anunciada");
        assert!(valid_accents.is_ok());

        // Título vacío
        let empty_title = BookTitle::from_str("");
        assert!(matches!(empty_title, Err(BookError::EmptyString)));

        // Título con emojis
        let emoji_title = BookTitle::from_str("El libro 📚 de aventuras");
        assert!(matches!(emoji_title, Err(BookError::InvalidCharacters)));
    }

    #[test]
    fn test_book_author_validation() {
        // Autor válido
        let valid_author = BookAuthor::from_str("Gabriel García Márquez");
        assert!(valid_author.is_ok());

        // Autor con caracteres especiales válidos
        let valid_special_chars = BookAuthor::from_str("J.R.R. Tolkien");
        assert!(valid_special_chars.is_ok());

        // Autor vacío
        let empty_author = BookAuthor::from_str("");
        assert!(matches!(empty_author, Err(BookError::EmptyString)));

        // Autor con emojis
        let emoji_author = BookAuthor::from_str("Miguel de Cervantes 🖋️");
        assert!(matches!(emoji_author, Err(BookError::InvalidCharacters)));
    }

    #[test]
    fn test_book_publisher_validation() {
        // Editorial válida
        let valid_publisher = BookPublisher::from_str("Editorial Planeta");
        assert!(valid_publisher.is_ok());

        // Editorial con caracteres especiales válidos
        let valid_special_chars = BookPublisher::from_str("Ediciones B & Co.");
        assert!(valid_special_chars.is_ok());

        // Editorial vacía
        let empty_publisher = BookPublisher::from_str("");
        assert!(matches!(empty_publisher, Err(BookError::EmptyString)));

        // Editorial con emojis
        let emoji_publisher = BookPublisher::from_str("Penguin Random House 🏢");
        assert!(matches!(emoji_publisher, Err(BookError::InvalidCharacters)));
    }

    #[test]
    fn test_book_isbn_validation() {
        // ISBN válido
        let valid_isbn = Isbn::from_str("9783161484100");
        assert!(valid_isbn.is_ok());

        // ISBN con espacios
        let valid_spaces = Isbn::from_str("978 3 16 148410 0");
        assert!(valid_spaces.is_ok());

        // ISBN con guiones
        let valid_dashes = Isbn::from_str("978-3-16-148410-0");
        assert!(valid_dashes.is_ok());

        // ISBN con guiones y espacios
        let valid_extra_spaces = Isbn::from_str("978-3-16-148410-0 ");
        assert!(valid_extra_spaces.is_ok());

        // ISBN con guiones y espacios adicionales
        let valid_spaces_and_dashes_extra = Isbn::from_str("978-3 16-148410-0 ");
        assert!(valid_spaces_and_dashes_extra.is_ok());

        // ISBN con prefijo
        let valid_prefix = Isbn::from_str("ISBN 978-3-16-148410-0");
        assert!(valid_prefix.is_ok());

        // ISBN con guiones y espacios adicionales al inicio
        let mixed_separators_and_spaces = Isbn::from_str(" 978-3 16-148410 0");
        assert!(mixed_separators_and_spaces.is_ok());

        let prefixed_isbn = Isbn::from_str("ISBN 978-3-16-148410-0").unwrap();
        let numbered_isbn = Isbn::from_str("9783161484100").unwrap();
        assert!(prefixed_isbn.canonical() == numbered_isbn.canonical());

        // ISBN vacío
        let empty_isbn = Isbn::from_str("");
        assert!(matches!(empty_isbn, Err(BookError::InvalidIsbn)));

        // ISBN con emojis
        let emoji_isbn = Isbn::from_str("978-3-16-148410-0 📚");
        dbg!(&emoji_isbn);
        assert!(matches!(emoji_isbn, Err(BookError::InvalidIsbn)));

        // ISBN con letras
        let isbn_with_letters = Isbn::from_str("978-3-16-148410-A");
        assert!(matches!(isbn_with_letters, Err(BookError::InvalidIsbn)));
    }
}
