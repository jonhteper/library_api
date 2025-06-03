use clap::Parser;

use crate::{
    api_keys::api_keys_application::{ApiKeyCreationService, ApiKeyDeletionService},
    init,
};

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    action: Action,
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.action {
            Action::Gen => {
                let api_key = ApiKeyCreationService::get_instance().create().await?;

                println!("{api_key}");
            }
            Action::Delete { id } => {
                ApiKeyDeletionService::get_instance().delete(&id).await?;

                println!("Deleted API key with ID: {}", id);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub enum Action {
    /// Crea una nueva API key y la muestra en pantalla (alias --gen)
    #[command(alias = "--gen")]
    Gen,

    /// Elimina una API key existente con base en su ID (alias --del)
    #[command(alias = "--del")]
    Delete {
        #[clap(value_parser, value_name = "APIKEY-ID")]
        id: String,
    },
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    init::init_db_services().await;
    let cli = Cli::parse();
    cli.run().await
}
