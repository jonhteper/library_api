#[cfg(not(feature = "cli"))]
#[tokio::main]
async fn main() {
    use library_api::config::Config;
    use library_api::init::{init_logger, run};

    // Cargar configuración
    let config = Config::get_instance();

    init_logger(&config.log_level);
    run(config.api_port)
        .await
        .expect("Error al iniciar el servidor");
}

#[cfg(feature = "cli")]
fn main() {}
