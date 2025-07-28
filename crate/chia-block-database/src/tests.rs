use crate::{DatabaseType};

#[test]
fn test_database_type_from_str() {
    // Test basic parsing functionality
    assert_eq!("sqlite".parse::<DatabaseType>().unwrap(), DatabaseType::Sqlite);
    assert_eq!("postgres".parse::<DatabaseType>().unwrap(), DatabaseType::Postgres);
    assert_eq!("postgresql".parse::<DatabaseType>().unwrap(), DatabaseType::Postgres);
    
    // Test error case
    assert!("invalid".parse::<DatabaseType>().is_err());
}

#[test]
fn test_database_type_clone_debug() {
    let db_type = DatabaseType::Sqlite;
    let cloned = db_type.clone();
    assert_eq!(db_type, cloned);
    
    // Test debug formatting
    let debug_str = format!("{:?}", db_type);
    assert!(debug_str.contains("Sqlite"));
}

// Note: Integration tests with actual database connections are commented out
// due to sqlx::Any requiring runtime driver installation. These would need
// to be run with specific database drivers or refactored to use concrete types.
//
// The following tests would work with actual database setup:
// - test_sqlite_database_creation
// - test_sqlite_database_migration  
// - test_database_type_detection
// - test_wal_mode_enabled 