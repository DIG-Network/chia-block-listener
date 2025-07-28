use crate::database::DatabaseType;

pub struct SystemPreferencesCrud {
    db_type: DatabaseType,
}

impl SystemPreferencesCrud {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }
} 