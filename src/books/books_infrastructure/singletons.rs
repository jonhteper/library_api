use std::sync::{Arc, LazyLock};

use proc_singleton::singleton_from_static_arc;

use crate::{
    books::books_application::{
        create_book_use_case::CreateBookUseCase, delete_book_use_case::DeleteBookUseCase,
        get_book_by_id_use_case::GetBookByIdUseCase, get_books_use_case::GetBooksUseCase,
        search_books_use_case::SearchBooksUseCase, update_book_use_case::UpdateBookUseCase,
    },
    init::get_pool,
};

use super::BookSqliteRepository;

// Singleton para el repositorio de libros
#[singleton_from_static_arc(BookSqliteRepository)]
static REPO: LazyLock<Arc<BookSqliteRepository>> =
    LazyLock::new(|| Arc::new(BookSqliteRepository { pool: get_pool() }));

// Singletons para los casos de uso

#[singleton_from_static_arc(CreateBookUseCase)]
static CREATE_BOOK_USE_CASE: LazyLock<Arc<CreateBookUseCase>> = LazyLock::new(|| {
    Arc::new(CreateBookUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(GetBooksUseCase)]
static GET_BOOKS_USE_CASE: LazyLock<Arc<GetBooksUseCase>> = LazyLock::new(|| {
    Arc::new(GetBooksUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(GetBookByIdUseCase)]
static GET_BOOK_BY_ID_USE_CASE: LazyLock<Arc<GetBookByIdUseCase>> = LazyLock::new(|| {
    Arc::new(GetBookByIdUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(UpdateBookUseCase)]
static UPDATE_BOOK_USE_CASE: LazyLock<Arc<UpdateBookUseCase>> = LazyLock::new(|| {
    Arc::new(UpdateBookUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(DeleteBookUseCase)]
static DELETE_BOOK_USE_CASE: LazyLock<Arc<DeleteBookUseCase>> = LazyLock::new(|| {
    Arc::new(DeleteBookUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});

#[singleton_from_static_arc(SearchBooksUseCase)]
static SEARCH_BOOKS_USE_CASE: LazyLock<Arc<SearchBooksUseCase>> = LazyLock::new(|| {
    Arc::new(SearchBooksUseCase {
        repo: BookSqliteRepository::get_instance(),
    })
});
