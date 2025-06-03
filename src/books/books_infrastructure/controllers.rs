use axum::{
    Json,
    extract::{Path, Query},
};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    AppError,
    books::{
        books_application::{
            create_book_use_case::{CreateBookDto, CreateBookUseCase},
            delete_book_use_case::DeleteBookUseCase,
            get_book_by_id_use_case::GetBookByIdUseCase,
            get_books_use_case::{GetBooksDto, GetBooksUseCase},
            search_books_use_case::{SearchBooksDto, SearchBooksUseCase},
            update_book_use_case::UpdateBookUseCase,
        },
        books_domain::{Book, PaginatedBooks},
    },
};

pub async fn create_book_controller(
    Json(dto): Json<CreateBookDto>,
) -> Result<(StatusCode, Json<BookId>), AppError> {
    let use_case = CreateBookUseCase::get_instance();

    let book_id = use_case.create_book(dto).await?;
    let response = BookId { id: book_id };

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BookId {
    pub id: Uuid,
}

pub async fn get_book_controller(Path(id): Path<String>) -> Result<Json<Book>, AppError> {
    let id = Uuid::try_from(id).map_err(|_| AppError::Custom("id inválido".into()))?;

    let use_case = GetBookByIdUseCase::get_instance();

    let book = use_case.get_book_by_id(id).await?;

    Ok(Json(book))
}

pub async fn update_book_controller(
    Path(id): Path<String>,
    Json(dto): Json<CreateBookDto>,
) -> Result<StatusCode, AppError> {
    let id = Uuid::try_from(id).map_err(|_| AppError::Custom("id inválido".into()))?;
    let book = Book::try_from((id, dto))?;

    let use_case = UpdateBookUseCase::get_instance();

    use_case.update_book(book).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_book_controller(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let id = Uuid::try_from(id).map_err(|_| AppError::Custom("id inválido".into()))?;

    let use_case = DeleteBookUseCase::get_instance();

    use_case.delete_book(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_books_controller(
    Query(dto): Query<GetBooksDto>,
) -> Result<Json<PaginatedBooks>, AppError> {
    let use_case = GetBooksUseCase::get_instance();

    let books = use_case.get_all_books(dto).await?;

    Ok(Json(books))
}

pub async fn search_books_controller(
    Query(dto): Query<SearchBooksDto>,
) -> Result<Json<PaginatedBooks>, AppError> {
    let use_case = SearchBooksUseCase::get_instance();

    let books = use_case.search_books(dto).await?;

    Ok(Json(books))
}
