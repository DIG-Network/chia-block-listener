#![deny(clippy::all)]

mod block_parser_napi;
mod dns_discovery_napi;
mod event_emitter;
mod peer_pool_napi;

pub use block_parser_napi::ChiaBlockParser;
pub use dns_discovery_napi::DnsDiscoveryClient;
pub use event_emitter::ChiaBlockListener;
pub use peer_pool_napi::ChiaPeerPool;

use napi_derive::napi;

#[napi]
pub fn init_tracing() {
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
