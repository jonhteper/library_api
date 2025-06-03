#[cfg(feature = "integration-tests")]
mod tests {
    use axum_test::TestServer;
    use http::header::AUTHORIZATION;
    use library_api::{
        api_keys::api_keys_application::ApiKeyCreationService,
        books::books_infrastructure::controllers::BookId, server::routes,
    };

    #[tokio::test]
    async fn home_works() {
        let app = routes().await;

        let server = TestServer::new(app).expect("Error al crear servidor de prueba");

        let response = server.get("/").await;
        assert_eq!(response.status_code(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn books_works() {
        let app = routes().await;

        let server = TestServer::new(app).expect("Error al crear servidor de prueba");

        let response = server.get("/books").await;
        assert_eq!(response.status_code(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn books_pagination_works() {
        let app = routes().await;

        let server = TestServer::new(app).expect("Error al crear servidor de prueba");

        let response = server.get("/books?page=2&pageSize=3").await;
        assert_eq!(response.status_code(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn book_lifecycle_works() {
        let app = routes().await;

        let server = TestServer::new(app).expect("Error al crear servidor de prueba");
        let raw_api_key = ApiKeyCreationService::get_instance()
            .create()
            .await
            .expect("Error al genera API Key");
        let api_key = format!("ApiKey {}", raw_api_key);

        let response = server
            .post("/books")
            .add_header(AUTHORIZATION, api_key.clone())
            .json(&serde_json::json!({
                "title": "El Quijote. Traducción Moderda",
                "authors": ["Miguel de Cervantes"],
                "publisher": "Fondo de Cultura Económica",
                "year": 2000,
                "isbn": "978-84-376-0000-0",
                "stored_quantity": 2
            }))
            .await;

        assert_eq!(response.status_code(), http::StatusCode::CREATED);

        let book_id = response.json::<BookId>();

        let response = server
            .get(&format!("/books/{}", book_id.id))
            .add_header(AUTHORIZATION, api_key.clone())
            .await;

        assert_eq!(response.status_code(), http::StatusCode::OK);

        let response = server
            .put(&format!("/books/{}", book_id.id))
            .add_header(AUTHORIZATION, api_key.clone())
            .json(&serde_json::json!({
                "title": "El Quijote. Traducción Moderda",
                "authors": ["Miguel de Cervantes"],
                "publisher": "Fondo de Cultura Económica",
                "year": 2000,
                "isbn": "978-84-376-0000-0",
                "stored_quantity": 3
            }))
            .await;

        assert_eq!(response.status_code(), http::StatusCode::NO_CONTENT);

        let response = server
            .delete(&format!("/books/{}", book_id.id))
            .add_header(AUTHORIZATION, format!("ApiKey {}", raw_api_key))
            .await;

        assert_eq!(response.status_code(), http::StatusCode::NO_CONTENT);
    }
}
