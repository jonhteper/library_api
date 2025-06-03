use std::sync::Arc;

use log::info;
use serde::{Deserialize, Serialize};

use crate::books::{
    BookError,
    books_domain::{BookRepository, PaginatedBooks, ReadMultipleBooksCriteria},
};

#[derive(Clone)]
pub struct SearchBooksUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl SearchBooksUseCase {
    /// Busca libros por título o autor
    pub async fn search_books(&self, dto: SearchBooksDto) -> Result<PaginatedBooks, BookError> {
        let criteria = match (&dto.title, &dto.author) {
            (Some(title), None) | (Some(title), Some(_)) if !title.trim().is_empty() => {
                info!("Buscando libros por título: {}", title);
                ReadMultipleBooksCriteria::ByTitle(title.clone())
            }
            (None, Some(author)) if !author.trim().is_empty() => {
                info!("Buscando libros por autor: {}", author);
                ReadMultipleBooksCriteria::ByAuthor(author.clone())
            }
            _ => {
                info!(
                    "No se proporcionaron criterios de búsqueda válidos, devolviendo todos los libros"
                );
                ReadMultipleBooksCriteria::All
            }
        };

        self.repo
            .find_multiple(criteria, dto.page, dto.page_size)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBooksDto {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page: u32,
    pub page_size: u8,
}
