use crate::types::*;
use crate::error::{IndexerError, Result};
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                connection_string: "sqlite:./chia_blockchain.db".to_string(),
                max_connections: 10,
                connection_timeout_secs: 30,
            },
            blockchain: BlockchainConfig {
                start_height: 1,
                network_id: "mainnet".to_string(),
                max_peers_historical: 3,
                max_peers_realtime: 2,
            },
            network: NetworkConfig {
                introducers: vec![
                    "introducer.chia.net".to_string(),
                    "introducer-or.chia.net".to_string(),
                    "introducer-eu.chia.net".to_string(),
                ],
                default_port: 8444,
                connection_timeout_secs: 30,
                reconnect_delay_secs: 5,
            },
            sync: SyncConfig {
                gap_scan_interval_secs: 30,
                status_update_interval_secs: 10,
                batch_size: 100,
                max_concurrent_downloads: 5,
                enable_balance_refresh: true,
            },
            watchdog: WatchdogConfig {
                timeout_minutes: 10,
                check_interval_secs: 60,
                max_stuck_time_secs: 600,
                max_no_progress_checks: 5,
            },
        }
    }
}

impl IndexerConfig {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self> {
        let mut config_builder = Config::builder()
            .add_source(Config::try_from(&IndexerConfig::default())?)
            .add_source(Environment::with_prefix("INDEXER").separator("_"));

        // Try to load from config file if specified
        if let Ok(config_file) = env::var("INDEXER_CONFIG_FILE") {
            config_builder = config_builder.add_source(File::with_name(&config_file).required(false));
        }

        // Try common config file locations
        config_builder = config_builder
            .add_source(File::with_name("indexer.toml").required(false))
            .add_source(File::with_name("config/indexer.toml").required(false))
            .add_source(File::with_name("/etc/chia-indexer/config.toml").required(false));

        let config = config_builder.build()?;
        let indexer_config: IndexerConfig = config.try_deserialize()?;

        indexer_config.validate()?;
        Ok(indexer_config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let config = Config::builder()
            .add_source(Config::try_from(&IndexerConfig::default())?)
            .add_source(File::with_name(path))
            .build()?;

        let indexer_config: IndexerConfig = config.try_deserialize()?;
        indexer_config.validate()?;
        Ok(indexer_config)
    }

    /// Create configuration from environment variables only
    pub fn from_env() -> Result<Self> {
        let config = Config::builder()
            .add_source(Config::try_from(&IndexerConfig::default())?)
            .add_source(Environment::with_prefix("INDEXER").separator("_"))
            .build()?;

        let indexer_config: IndexerConfig = config.try_deserialize()?;
        indexer_config.validate()?;
        Ok(indexer_config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate database config
        if self.database.connection_string.is_empty() {
            return Err(IndexerError::Config(ConfigError::Message(
                "Database connection string cannot be empty".to_string(),
            )));
        }

        if self.database.max_connections == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Database max connections must be greater than 0".to_string(),
            )));
        }

        // Validate blockchain config
        if self.blockchain.network_id.is_empty() {
            return Err(IndexerError::Config(ConfigError::Message(
                "Network ID cannot be empty".to_string(),
            )));
        }

        if !["mainnet", "testnet11"].contains(&self.blockchain.network_id.as_str()) {
            return Err(IndexerError::Config(ConfigError::Message(
                "Network ID must be 'mainnet' or 'testnet11'".to_string(),
            )));
        }

        if self.blockchain.max_peers_historical == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Max peers for historical sync must be greater than 0".to_string(),
            )));
        }

        if self.blockchain.max_peers_realtime == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Max peers for realtime sync must be greater than 0".to_string(),
            )));
        }

        // Validate network config
        if self.network.introducers.is_empty() {
            return Err(IndexerError::Config(ConfigError::Message(
                "At least one introducer must be specified".to_string(),
            )));
        }

        if self.network.default_port == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Default port must be greater than 0".to_string(),
            )));
        }

        // Validate sync config
        if self.sync.batch_size == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Batch size must be greater than 0".to_string(),
            )));
        }

        if self.sync.max_concurrent_downloads == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Max concurrent downloads must be greater than 0".to_string(),
            )));
        }

        // Validate watchdog config
        if self.watchdog.timeout_minutes == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Watchdog timeout must be greater than 0".to_string(),
            )));
        }

        if self.watchdog.max_no_progress_checks == 0 {
            return Err(IndexerError::Config(ConfigError::Message(
                "Max no progress checks must be greater than 0".to_string(),
            )));
        }

        Ok(())
    }

    /// Get the database type from the connection string
    pub fn database_type(&self) -> chia_block_database::DatabaseType {
        if self.database.connection_string.starts_with("postgres://") 
            || self.database.connection_string.starts_with("postgresql://") {
            chia_block_database::DatabaseType::Postgres
        } else {
            chia_block_database::DatabaseType::Sqlite
        }
    }

    /// Check if this is a development environment
    pub fn is_development(&self) -> bool {
        env::var("INDEXER_ENV").unwrap_or_default() == "development"
            || env::var("RUST_ENV").unwrap_or_default() == "development"
    }

    /// Get the log level
    pub fn log_level(&self) -> String {
        env::var("INDEXER_LOG_LEVEL")
            .or_else(|_| env::var("RUST_LOG"))
            .unwrap_or_else(|_| {
                if self.is_development() {
                    "debug".to_string()
                } else {
                    "info".to_string()
                }
            })
    }
}

/// Initialize tracing with the given configuration
pub fn init_tracing(config: &IndexerConfig) -> Result<()> {
    use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

    let log_level = config.log_level();
    
    let filter = EnvFilter::builder()
        .with_default_directive(
            log_level.parse::<LevelFilter>()
                .unwrap_or(LevelFilter::INFO)
                .into()
        )
        .from_env_lossy()
        .add_directive("chia_full_indexer=debug".parse().unwrap())
        .add_directive("chia_block_database=info".parse().unwrap())
        .add_directive("dns_discovery=info".parse().unwrap());

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(config.is_development())
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = IndexerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_type_detection() {
        let mut config = IndexerConfig::default();
        
        config.database.connection_string = "sqlite:./test.db".to_string();
        assert!(matches!(config.database_type(), chia_block_database::DatabaseType::Sqlite));
        
        config.database.connection_string = "postgresql://user:pass@localhost/db".to_string();
        assert!(matches!(config.database_type(), chia_block_database::DatabaseType::Postgres));
    }

    #[test]
    fn test_invalid_network_id() {
        let mut config = IndexerConfig::default();
        config.blockchain.network_id = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_introducers() {
        let mut config = IndexerConfig::default();
        config.network.introducers.clear();
        assert!(config.validate().is_err());
    }
} 