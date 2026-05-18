use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
};

use clap::Parser;
use m_cc::{
    AppState, app,
    config::{AppConfig, Config},
    managers::{
        offset_manager::{self, OffsetManager},
        topic_manager::TopicManager,
    },
    types::metrics::Metrics,
};

#[derive(clap::Parser, Debug)]
#[command(name = "m-cc", about = "Configurable MQ server")]
struct Cli {
    /// Path to YAML config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let metrics = Arc::new(Metrics {
        delivered: AtomicU64::new(0),
        received: AtomicU64::new(0),
    });

    let default_path = "./config/default.yaml";

    let path = cli.config.unwrap_or_else(|| default_path.into());

    let config = Config::load(path.to_str().unwrap_or(default_path)).unwrap_or_else(|e| {
        eprintln!("Failed to load config file with error: {}", e);
        Config::get_default_values()
    });

    let data_directory = PathBuf::from(&config.storage.data_dir);

    if let Err(_) = data_directory.try_exists() {
        println!("Trying to create directory");
        if let Err(err) = fs::create_dir_all(&data_directory) {
            eprintln!(
                "Could not create storage directory, please check your data storage config, {}",
                err
            );
            return;
        }
    }

    let topic_manager = Arc::new(TopicManager::new(&config.storage.data_dir, config.writer));
    let offset_manager = Arc::new(OffsetManager::new(&config.storage.data_dir));

    let app_config = AppConfig {
        runtime: config.runtime,
    };

    let state = Arc::new(AppState {
        topic_manager,
        offset_manager,
        metrics,
        app_config: Arc::new(app_config),
    });

    let app = app(state);

    let addr = format!("{}:{}", config.server.address, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
