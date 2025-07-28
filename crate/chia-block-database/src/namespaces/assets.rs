use crate::crud::{CatsCrud, NftsCrud};
use crate::database::DatabaseType;

/// Assets namespace groups all asset-related operations (CATs, NFTs)
pub struct AssetsNamespace {
    cats: CatsCrud,
    nfts: NftsCrud,
}

impl AssetsNamespace {
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            cats: CatsCrud::new(db_type.clone()),
            nfts: NftsCrud::new(db_type),
        }
    }

    /// Access to CATs CRUD operations
    pub fn cats(&self) -> &CatsCrud {
        &self.cats
    }

    /// Access to NFTs CRUD operations
    pub fn nfts(&self) -> &NftsCrud {
        &self.nfts
    }
} 