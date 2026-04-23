use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct RuntimeConfig {
    pub worker_threads: u32,
}

#[derive(Deserialize, Clone, Copy)]
pub struct WriterConfig {
    pub max_batch_commands: usize,
    pub flush_bytes: usize,
    pub flush_interval_ms: u64,
}

#[derive(Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub writer: WriterConfig,
    pub runtime: RuntimeConfig,
}

pub struct AppConfig {
    pub runtime: RuntimeConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Config> {
        let file_content = std::fs::read_to_string(path)?;
        let data: Config = serde_yaml::from_str(&file_content)?;

        Ok(data)
    }

    pub fn get_default_values() -> Config {
        let runtime = RuntimeConfig { worker_threads: 4 };

        let writer = WriterConfig {
            max_batch_commands: 65536,
            flush_bytes: 33554432,
            flush_interval_ms: 200,
        };

        let storage = StorageConfig {
            data_dir: "./m-cc/data".to_string(),
        };

        let server = ServerConfig {
            address: "0.0.0.0".to_string(),
            port: 3000,
        };

        Config {
            server,
            storage,
            writer,
            runtime,
        }
    }
}
