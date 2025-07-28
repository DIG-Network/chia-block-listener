use crate::database::DatabaseType;

pub struct SyncStatusCrud {
    db_type: DatabaseType,
}

impl SyncStatusCrud {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }
} 