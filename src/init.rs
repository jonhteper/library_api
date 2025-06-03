use std::sync::OnceLock;
use std::{io, net::SocketAddr};

use sqlx::SqlitePool;

use crate::server::routes;

// Singleton para el SqlitePool
static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn initialize_pool(connection_string: &str) {
    let pool = SqlitePool::connect(connection_string).await.unwrap();

    DB_POOL.get_or_init(|| pool);
}

pub fn get_pool() -> SqlitePool {
    let pool = DB_POOL
        .get()
        .expect("Pool no inicializado. Llama a initialize_pool primero.");

    pool.clone()
}

#[inline]
fn db_conn_str() -> String {
    #[cfg(feature = "integration-tests")]
    return "sqlite::memory:".to_string();

    #[cfg(not(feature = "integration-tests"))]
    return crate::config::Config::get_instance().database_url.clone();
}

pub async fn init_db_services() {
    // Inicializamos el pool de conexiones
    initialize_pool(&db_conn_str()).await;

    // Inicializamos las tablas
    let api_key_repo =
        crate::api_keys::api_keys_infrastructure::ApiKeySqliteRepository::get_instance();
    let book_repo = crate::books::books_infrastructure::BookSqliteRepository::get_instance();

    // Inicializamos las tablas en paralelo
    let (result_api_key, result_book) =
        tokio::join!(api_key_repo.init_table(), book_repo.init_tables());

    // Verificamos los resultados de las inicializaciones
    result_api_key.expect("Error al inicializar base de datos de ApiKey");
    result_book.expect("Error al inicializar base de datos de Book");

    #[cfg(not(feature = "cli"))]
    println!("Tablas inicializadas correctamente");
}

pub fn init_logger(level: &str) {
    use log::LevelFilter;
    // Inicializar el logger con el nivel especificado en la configuración
    let log_level = match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info, // Valor por defecto
    };

    env_logger::Builder::new().filter_level(log_level).init();
}

pub async fn run(port: u16) -> io::Result<()> {
    let routes = routes().await;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!(
        "Ejecutando Library API v{} en localhost:{}",
        env!("CARGO_PKG_VERSION"),
        port
    );

    axum_server::bind(addr)
        .serve(routes.into_make_service())
        .await
}
