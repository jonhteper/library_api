#[cfg(feature = "cli")]
use library_api::api_keys::api_keys_infrastructure::cli::run;

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

#[cfg(not(feature = "cli"))]
fn main() {}
