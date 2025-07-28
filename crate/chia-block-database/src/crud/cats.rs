use crate::database::DatabaseType;

pub struct CatsCrud {
    db_type: DatabaseType,
}

impl CatsCrud {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }
} 