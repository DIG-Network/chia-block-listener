use crate::database::DatabaseType;
use crate::error::Result;
use sqlx::AnyPool;

pub struct NftsCrud {
    db_type: DatabaseType,
}

impl NftsCrud {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }
}

// TODO: Implement NFT CRUD operations 