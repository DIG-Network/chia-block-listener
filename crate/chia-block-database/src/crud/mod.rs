pub mod blocks;
pub mod coins;
pub mod spends;
pub mod nfts;
pub mod cats;
pub mod system_preferences;
pub mod sync_status;

pub use blocks::BlocksCrud;
pub use coins::CoinsCrud;
pub use spends::SpendsCrud;
pub use nfts::NftsCrud;
pub use cats::CatsCrud;
pub use system_preferences::SystemPreferencesCrud;
pub use sync_status::SyncStatusCrud; 