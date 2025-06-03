use std::cmp::max;

use sqlx::{Pool, Sqlite, query, query_as};
use uuid::Uuid;

use crate::books::{
    BookError,
    books_domain::{
        Book, BookAuthor, BookRepository, PaginatedBooks, ReadBookCriteria,
        ReadMultipleBooksCriteria,
    },
    books_infrastructure::db_dtos::{DbAuthorName, DbBook},
};

#[derive(Clone)]
pub struct BookSqliteRepository {
    pub pool: Pool<Sqlite>,
}

impl BookSqliteRepository {
    pub async fn init_tables(&self) -> Result<(), BookError> {
        // Crear tabla de libros
        query(
            r#"
            CREATE TABLE IF NOT EXISTS books (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                publisher TEXT NOT NULL,
                year INTEGER NOT NULL,
                isbn TEXT NOT NULL,
                stored INTEGER NOT NULL,
                UNIQUE(isbn)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            BookError::DatabaseError(format!("Error al crear tabla books: {}", e).into())
        })?;

        // Crear tabla de autores
        query(
            r#"
            CREATE TABLE IF NOT EXISTS authors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            BookError::DatabaseError(format!("Error al crear tabla authors: {}", e).into())
        })?;

        // Crear tabla de relación libro-autor
        query(
            r#"
            CREATE TABLE IF NOT EXISTS book_authors (
                book_id TEXT NOT NULL,
                author_id INTEGER NOT NULL,
                PRIMARY KEY (book_id, author_id),
                FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
                FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            BookError::DatabaseError(format!("Error al crear tabla book_authors: {}", e).into())
        })?;

        Ok(())
    }

    async fn get_or_create_author(&self, author: &BookAuthor) -> Result<i64, BookError> {
        // Intentamos obtener el autor
        let author_name = author.as_str();
        let result = query_as::<_, (i64,)>("SELECT id FROM authors WHERE name = ?")
            .bind(author_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al buscar autor: {}", e).into())
            })?;

        if let Some(row) = result {
            return Ok(row.0);
        }

        // Si no existe, lo creamos
        let result = query_as::<_, (i64,)>("INSERT INTO authors (name) VALUES (?) RETURNING id")
            .bind(author_name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| BookError::DatabaseError(format!("Error al crear autor: {}", e).into()))?;

        Ok(result.0)
    }

    async fn save_book_authors(
        &self,
        book_id: Uuid,
        authors: &[BookAuthor],
    ) -> Result<(), BookError> {
        // Primero eliminamos las relaciones existentes
        query("DELETE FROM book_authors WHERE book_id = ?")
            .bind(book_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(
                    format!("Error al eliminar relaciones libro-autor: {}", e).into(),
                )
            })?;

        // Ahora creamos las nuevas relaciones
        for author in authors {
            let author_id = self.get_or_create_author(author).await?;

            query("INSERT INTO book_authors (book_id, author_id) VALUES (?, ?)")
                .bind(book_id.to_string())
                .bind(author_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    BookError::DatabaseError(
                        format!("Error al guardar relación libro-autor: {}", e).into(),
                    )
                })?;
        }

        Ok(())
    }

    async fn load_book_authors(&self, book_id: &str) -> Result<Vec<BookAuthor>, BookError> {
        let authors = query_as::<_, DbAuthorName>(
            r#"
            SELECT a.name
            FROM authors a
            JOIN book_authors ba ON a.id = ba.author_id
            WHERE ba.book_id = ?
            ORDER BY a.name
            "#,
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            BookError::DatabaseError(format!("Error al cargar autores del libro: {}", e).into())
        })?;

        let mut result = Vec::with_capacity(authors.len());
        for author in authors {
            result.push(author.to_domain()?);
        }

        Ok(result)
    }

    async fn get_total_books(
        &self,
        criteria: &ReadMultipleBooksCriteria,
    ) -> Result<u64, BookError> {
        let result = match criteria {
            ReadMultipleBooksCriteria::All => query_as::<_, (i64,)>("SELECT COUNT(*) FROM books")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    BookError::DatabaseError(format!("Error al contar libros: {}", e).into())
                })?,
            ReadMultipleBooksCriteria::ByTitle(title) => {
                query_as::<_, (i64,)>("SELECT COUNT(*) FROM books WHERE title LIKE ?")
                    .bind(format!("%{}%", title))
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        BookError::DatabaseError(
                            format!("Error al contar libros por título: {}", e).into(),
                        )
                    })?
            }
            ReadMultipleBooksCriteria::ByAuthor(author) => query_as::<_, (i64,)>(
                r#"
                    SELECT COUNT(DISTINCT b.id)
                    FROM books b
                    JOIN book_authors ba ON b.id = ba.book_id
                    JOIN authors a ON ba.author_id = a.id
                    WHERE a.name LIKE ?
                    "#,
            )
            .bind(format!("%{}%", author))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al contar libros por autor: {}", e).into())
            })?,
        };

        Ok(result.0 as u64)
    }
}

#[async_trait::async_trait]
impl BookRepository for BookSqliteRepository {
    async fn find(&self, criteria: ReadBookCriteria) -> Result<Option<Book>, BookError> {
        let (param, query) = match criteria {
            ReadBookCriteria::ById(uuid) => (
                uuid.to_string(),
                r#"
                SELECT *
                FROM books
                WHERE id = ?
                "#,
            ),
            ReadBookCriteria::ByIsbn(isbn) => (
                isbn.to_string(),
                r#"
                SELECT *
                FROM books
                WHERE isbn = ?
                "#,
            ),
        };

        let book = query_as::<_, DbBook>(query)
            .bind(&param)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al buscar libro: {}", e).into())
            })?;

        if let Some(db_book) = book {
            let authors = self.load_book_authors(&db_book.id).await?;
            db_book.to_domain_with_authors(authors).await.map(Some)
        } else {
            Ok(None)
        }
    }

    async fn find_multiple(
        &self,
        criteria: ReadMultipleBooksCriteria,
        page: u32,
        page_size: u8,
    ) -> Result<PaginatedBooks, BookError> {
        let page = max(page, 1); // prevenimos un substract-overflow
        let offset = (page - 1) as i64 * page_size as i64;
        let limit = page_size as i64;

        let books = match &criteria {
            ReadMultipleBooksCriteria::All => query_as::<_, DbBook>(
                r#"
                    SELECT *
                    FROM books
                    ORDER BY title
                    LIMIT ? OFFSET ?
                    "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al buscar libros: {}", e).into())
            })?,
            ReadMultipleBooksCriteria::ByTitle(title) => query_as::<_, DbBook>(
                r#"
                    SELECT *
                    FROM books
                    WHERE title LIKE ?
                    ORDER BY title
                    LIMIT ? OFFSET ?
                    "#,
            )
            .bind(format!("%{}%", title))
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al buscar libros por título: {}", e).into())
            })?,
            ReadMultipleBooksCriteria::ByAuthor(author) => query_as::<_, DbBook>(
                r#"
                    SELECT DISTINCT b.*
                    FROM books b
                    JOIN book_authors ba ON b.id = ba.book_id
                    JOIN authors a ON ba.author_id = a.id
                    WHERE a.name LIKE ?
                    ORDER BY b.title
                    LIMIT ? OFFSET ?
                    "#,
            )
            .bind(format!("%{}%", author))
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al buscar libros por autor: {}", e).into())
            })?,
        };

        let mut domain_books = Vec::with_capacity(books.len());
        for db_book in books {
            let authors = self.load_book_authors(&db_book.id).await?;
            let book = db_book.to_domain_with_authors(authors).await?;
            domain_books.push(book);
        }

        let total = self.get_total_books(&criteria).await?;

        Ok(PaginatedBooks {
            books: domain_books,
            total,
            page,
            page_size,
        })
    }

    async fn create(&self, book: Book) -> Result<(), BookError> {
        // Crear el libro en la tabla de libros
        query(
            r#"
            INSERT INTO books (id, title, publisher, year, isbn, stored)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(book.id.to_string())
        .bind(book.title.as_str())
        .bind(book.publisher.as_str())
        .bind(book.year)
        .bind(book.isbn.canonical())
        .bind(book.stored_quantity as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| BookError::DatabaseError(format!("Error al crear libro: {}", e).into()))?;

        // Guardar los autores del libro
        self.save_book_authors(book.id, &book.authors).await?;

        Ok(())
    }

    async fn update(&self, book: Book) -> Result<(), BookError> {
        // Verificar si existe el libro
        let existing = self.find(ReadBookCriteria::ById(book.id)).await?;

        if existing.is_none() {
            return Err(BookError::NotFound);
        }

        // Actualizar el libro
        query(
            r#"
            UPDATE books
            SET title = ?, year = ?, publisher = ?, stored = ?, isbn = ?
            WHERE id = ?
            "#,
        )
        .bind(book.title.as_str())
        .bind(book.year)
        .bind(book.publisher.as_str())
        .bind(book.stored_quantity as i64)
        .bind(book.isbn.as_str())
        .bind(book.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            BookError::DatabaseError(format!("Error al actualizar libro: {}", e).into())
        })?;

        // Actualizar los autores del libro
        self.save_book_authors(book.id, &book.authors).await?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), BookError> {
        // Verificar si existe el libro
        let existing = self.find(ReadBookCriteria::ById(*id)).await?;

        if existing.is_none() {
            return Err(BookError::NotFound);
        }

        // SQLite elimina automáticamente las filas relacionadas en book_authors
        // debido a la restricción ON DELETE CASCADE
        query("DELETE FROM books WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| {
                BookError::DatabaseError(format!("Error al eliminar libro: {}", e).into())
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::books::books_domain::{BookAuthor, BookPublisher, BookTitle, Isbn};
    use sqlx::SqlitePool;

    async fn setup_test_db() -> BookSqliteRepository {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite database");

        let repo = BookSqliteRepository { pool };
        repo.init_tables()
            .await
            .expect("Failed to initialize tables");

        repo
    }

    fn create_test_book() -> Book {
        let id = Uuid::new_v4();
        let title = BookTitle::try_from("Test Book".to_string()).unwrap();
        let author1 = BookAuthor::try_from("Author One".to_string()).unwrap();
        let author2 = BookAuthor::try_from("Author Two".to_string()).unwrap();
        let publisher = BookPublisher::try_from("Test Publisher".to_string()).unwrap();
        let isbn = Isbn::try_from("978-3-16-148410-0".to_string()).unwrap();

        Book {
            id,
            title,
            authors: vec![author1, author2],
            year: 2023,
            publisher,
            stored_quantity: 5,
            isbn,
        }
    }

    #[tokio::test]
    async fn test_create_and_find_by_id() {
        // Arrange
        let repo = setup_test_db().await;
        let book = create_test_book();
        let book_id = book.id;

        // Act
        repo.create(book).await.expect("Failed to create book");
        let result = repo
            .find(ReadBookCriteria::ById(book_id))
            .await
            .expect("Failed to find book");

        // Assert
        assert!(result.is_some(), "Book not found");
        let found_book = result.unwrap();
        assert_eq!(&found_book.id, &book_id);
        assert_eq!(found_book.authors.len(), 2);
        assert_eq!(found_book.year, 2023);
        assert_eq!(found_book.stored_quantity, 5);
    }

    #[tokio::test]
    async fn test_update_book() {
        // Arrange
        let repo = setup_test_db().await;
        let book = create_test_book();
        let book_id = book.id;

        // Act - Create and then update
        repo.create(book).await.expect("Failed to create book");

        let found_book = repo
            .find(ReadBookCriteria::ById(book_id))
            .await
            .expect("Failed to find book")
            .unwrap();
        let new_title = BookTitle::try_from("Updated Title".to_string()).unwrap();
        let new_author = BookAuthor::try_from("New Author".to_string()).unwrap();

        // Create updated book with same ID but different fields
        let updated_book = Book {
            id: found_book.id,
            title: new_title,
            authors: vec![new_author],
            year: 2024,
            publisher: found_book.publisher.clone(),
            stored_quantity: 10,
            isbn: found_book.isbn.clone(),
        };

        repo.update(updated_book)
            .await
            .expect("Failed to update book");

        // Assert
        let result = repo
            .find(ReadBookCriteria::ById(book_id))
            .await
            .expect("Failed to find updated book");
        assert!(result.is_some(), "Updated book not found");

        let found_book = result.unwrap();
        assert_eq!(found_book.id, book_id);
        assert_eq!(found_book.title.as_str(), "Updated Title");
        assert_eq!(found_book.authors.len(), 1);
        assert_eq!(found_book.authors[0].as_str(), "New Author");
        assert_eq!(found_book.year, 2024);
        assert_eq!(found_book.stored_quantity, 10);
    }

    #[tokio::test]
    async fn test_delete_book() {
        // Arrange
        let repo = setup_test_db().await;
        let book = create_test_book();
        let book_id = book.id;

        // Act - Create and then delete
        repo.create(book).await.expect("Failed to create book");
        repo.delete(&book_id).await.expect("Failed to delete book");

        // Assert
        let result = repo
            .find(ReadBookCriteria::ById(book_id))
            .await
            .expect("Failed to query book");
        assert!(result.is_none(), "Book should have been deleted");
    }

    #[tokio::test]
    async fn test_find_multiple_all() {
        // Arrange
        let repo = setup_test_db().await;
        let base_isbn = "978-3-16-148410-";

        // Create several books
        for i in 1..=5 {
            let id = Uuid::new_v4();
            let title = BookTitle::try_from(format!("Book {}", i)).unwrap();
            let author = BookAuthor::try_from(format!("Author {}", i)).unwrap();
            let publisher = BookPublisher::try_from("Publisher".to_string()).unwrap();

            let isbn = Isbn::try_from(format!("{}{}", base_isbn, i)).unwrap();
            let book = Book {
                id,
                title,
                authors: vec![author],
                year: 2020 + i,
                publisher,
                stored_quantity: i,
                isbn,
            };

            repo.create(book).await.expect("Failed to create book");
        }

        // Act
        let result = repo
            .find_multiple(ReadMultipleBooksCriteria::All, 1, 10)
            .await
            .expect("Failed to find books");

        // Assert
        assert_eq!(result.total, 5);
        assert_eq!(result.books.len(), 5);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
    }

    #[tokio::test]
    async fn test_find_multiple_by_title() {
        // Arrange
        let repo = setup_test_db().await;

        // Create books with specific titles
        let titles = [
            "Apple Book",
            "Banana Book",
            "Apple Cookbook",
            "Orange Guide",
            "Pear Manual",
        ];
        let base_isbn = "978-3-16-148410-";

        for (index, title) in titles.iter().enumerate() {
            let id = Uuid::new_v4();
            let book_title = BookTitle::try_from(title.to_string()).unwrap();
            let author = BookAuthor::try_from("Some Author".to_string()).unwrap();
            let publisher = BookPublisher::try_from("Publisher".to_string()).unwrap();
            let isbn = Isbn::try_from(format!("{}{}", base_isbn, index + 1)).unwrap();
            let book = Book {
                id,
                title: book_title,
                authors: vec![author],
                year: 2023,
                publisher,
                stored_quantity: 1,
                isbn,
            };

            repo.create(book).await.expect("Failed to create book");
        }

        // Act
        let result = repo
            .find_multiple(
                ReadMultipleBooksCriteria::ByTitle("Apple".to_string()),
                1,
                10,
            )
            .await
            .expect("Failed to find books by title");

        // Assert
        assert_eq!(result.total, 2); // Should find "Apple Book" and "Apple Cookbook"
        assert_eq!(result.books.len(), 2);
    }

    #[tokio::test]
    async fn test_find_multiple_by_author() {
        // Arrange
        let repo = setup_test_db().await;

        // Create books with specific authors
        let author_combinations = [
            ("Book 1", vec!["John Smith", "Jane Doe"]),
            ("Book 2", vec!["John Smith"]),
            ("Book 3", vec!["Alice M'Carty", "Bob Brown"]),
            ("Book 4", vec!["Jane Doe"]),
            ("Book 5", vec!["Charlie Davis"]),
        ];
        let base_isbn = "978-3-16-148410-";
        for (index, (title, authors)) in author_combinations.iter().enumerate() {
            let id = Uuid::new_v4();
            let book_title = BookTitle::try_from(title.to_string()).unwrap();
            let book_authors: Vec<BookAuthor> = authors
                .iter()
                .map(|a| BookAuthor::try_from(a.to_string()).unwrap())
                .collect();
            let publisher = BookPublisher::try_from("Publisher".to_string()).unwrap();
            let isbn = Isbn::try_from(format!("{}{}", base_isbn, index + 1)).unwrap();
            let book = Book {
                id,
                title: book_title,
                authors: book_authors,
                year: 2023,
                publisher,
                stored_quantity: 1,
                isbn,
            };

            repo.create(book).await.expect("Failed to create book");
        }

        // Act
        let result = repo
            .find_multiple(
                ReadMultipleBooksCriteria::ByAuthor("John".to_string()),
                1,
                10,
            )
            .await
            .expect("Failed to find books by author");

        // Assert
        assert_eq!(result.total, 2); // Should find "Book 1" and "Book 2" which have "John Smith"
        assert_eq!(result.books.len(), 2);
    }

    #[tokio::test]
    async fn test_pagination() {
        // Arrange
        let repo = setup_test_db().await;
        let base_isbn = "978-3-16-148410-";
        // Create 9 books
        for i in 1..=9 {
            let id = Uuid::new_v4();
            let title = BookTitle::try_from(format!("Book {}", i)).unwrap();
            let author = BookAuthor::try_from("Author".to_string()).unwrap();
            let publisher = BookPublisher::try_from("Publisher".to_string()).unwrap();
            let isbn = Isbn::try_from(format!("{}{}", base_isbn, i)).unwrap();
            let book = Book {
                id,
                title,
                authors: vec![author],
                year: 2023,
                publisher,
                stored_quantity: 1,
                isbn,
            };

            repo.create(book).await.expect("Failed to create book");
        }

        // Act - Get first page with 3 items
        let page1 = repo
            .find_multiple(ReadMultipleBooksCriteria::All, 1, 3)
            .await
            .expect("Failed to get first page");

        // Get second page with 3 items
        let page2 = repo
            .find_multiple(ReadMultipleBooksCriteria::All, 2, 3)
            .await
            .expect("Failed to get second page");

        // Assert
        assert_eq!(page1.total, 9); // Total should be 9 for both queries
        assert_eq!(page2.total, 9);

        assert_eq!(page1.books.len(), 3); // Each page should have 3 items
        assert_eq!(page2.books.len(), 3);

        assert_eq!(page1.page, 1);
        assert_eq!(page2.page, 2);

        // Verify different books on different pages
        let first_page_titles: Vec<String> =
            page1.books.iter().map(|b| b.title.to_string()).collect();

        let second_page_titles: Vec<String> =
            page2.books.iter().map(|b| b.title.to_string()).collect();

        for title in &first_page_titles {
            assert!(
                !second_page_titles.contains(title),
                "Books should not appear on multiple pages"
            );
        }
    }
}
