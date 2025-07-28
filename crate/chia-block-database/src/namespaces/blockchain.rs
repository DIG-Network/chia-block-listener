use crate::crud::{BlocksCrud, CoinsCrud, SpendsCrud};
use crate::database::DatabaseType;
use sqlx::AnyPool;

/// Blockchain namespace groups all blockchain-related operations (blocks, coins, spends)
pub struct BlockchainNamespace {
    blocks: BlocksCrud,
    coins: CoinsCrud,
    spends: SpendsCrud,
}

impl BlockchainNamespace {
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            blocks: BlocksCrud::new(db_type.clone()),
            coins: CoinsCrud::new(db_type.clone()),
            spends: SpendsCrud::new(db_type),
        }
    }

    /// Access to blocks CRUD operations
    pub fn blocks(&self) -> &BlocksCrud {
        &self.blocks
    }

    /// Access to coins CRUD operations
    pub fn coins(&self) -> &CoinsCrud {
        &self.coins
    }

    /// Access to spends CRUD operations (with puzzle/solution abstraction)
    pub fn spends(&self) -> &SpendsCrud {
        &self.spends
    }
} 