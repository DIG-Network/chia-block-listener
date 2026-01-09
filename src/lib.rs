#![deny(clippy::all)]

pub mod error;
pub mod types;
pub mod peer;
pub mod peer_pool;
pub mod protocol;
pub mod tls;
pub mod block_listener;

pub use block_listener::BlockListener;
pub use types::ListenerConfig;

pub fn init_tracing() {
    // Initialize logging with a filter for our crate
    use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("chia_block_listener=debug".parse().unwrap());

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
}
