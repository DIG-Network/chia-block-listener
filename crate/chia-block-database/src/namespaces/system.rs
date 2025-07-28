use crate::crud::{SyncStatusCrud, SystemPreferencesCrud};
use crate::database::DatabaseType;

/// System namespace groups all system-related operations (sync status, preferences)
pub struct SystemNamespace {
    sync_status: SyncStatusCrud,
    preferences: SystemPreferencesCrud,
}

impl SystemNamespace {
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            sync_status: SyncStatusCrud::new(db_type.clone()),
            preferences: SystemPreferencesCrud::new(db_type),
        }
    }

    /// Access to sync status CRUD operations
    pub fn sync_status(&self) -> &SyncStatusCrud {
        &self.sync_status
    }

    /// Access to system preferences CRUD operations
    pub fn preferences(&self) -> &SystemPreferencesCrud {
        &self.preferences
    }
} 