use axum::{
    Json, Router, middleware,
    routing::{get, post},
};
use http::StatusCode;

use serde_json::{Value, json};
use tower_http::trace::{self, TraceLayer};
use tracing_core::Level;

use crate::{
    api_keys::api_keys_infrastructure::api_key_middleware,
    books::books_infrastructure::controllers::{
        create_book_controller, delete_book_controller, get_all_books_controller,
        get_book_controller, update_book_controller,
    },
};
use crate::{books::books_infrastructure::controllers::search_books_controller, init};

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Ruta no encontrada")
}

pub async fn routes() -> Router {
    // Asegurar que el pool esté inicializado y las tablas creadas
    init::init_db_services().await;

    let books_router = books_routes();

    Router::new()
        .route("/", get(index))
        .nest("/books", books_router)
        .fallback(fallback)
        // Añadir TraceLayer para logging de peticiones HTTP
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

fn books_routes() -> Router {
    // Rutas públicas que no requieren autenticación
    let public_routes = Router::new()
        .route("/", get(get_all_books_controller))
        .route("/search", get(search_books_controller));

    // Rutas protegidas que requieren autenticación
    let protected_routes = Router::new()
        .route("/", post(create_book_controller))
        .route(
            "/{id}",
            get(get_book_controller)
                .put(update_book_controller)
                .delete(delete_book_controller),
        )
        .layer(middleware::from_fn(api_key_middleware));

    // Combinar las rutas públicas y protegidas
    public_routes.merge(protected_routes)
}

async fn index() -> Json<Value> {
    let version = env!("CARGO_PKG_VERSION");
    let response = json! ({
        "version": version,
        "description": "API Rest para gestión de biblioteca"
    });

    Json(response)
}
