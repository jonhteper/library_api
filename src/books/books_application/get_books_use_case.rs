use std::sync::Arc;

use log::info;
use serde::{Deserialize, Serialize};

use crate::books::{
    BookError,
    books_domain::{BookRepository, PaginatedBooks, ReadMultipleBooksCriteria},
};

#[derive(Clone)]
pub struct GetBooksUseCase {
    pub repo: Arc<dyn BookRepository>,
}

impl GetBooksUseCase {
    /// Obtiene todos los libros de la base de datos con paginación
    pub async fn get_all_books(&self, dto: GetBooksDto) -> Result<PaginatedBooks, BookError> {
        info!(
            "Obteniendo libros, página: {}, tamaño: {}",
            dto.page, dto.page_size
        );

        self.repo
            .find_multiple(ReadMultipleBooksCriteria::All, dto.page, dto.page_size)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBooksDto {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_page_size")]
    pub page_size: u8,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u8 {
    10
}
