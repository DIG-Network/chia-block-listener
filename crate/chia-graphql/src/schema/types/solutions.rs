use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Solution information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SolutionInfo {
    /// Solution ID
    pub id: i64,
    /// Solution hash hex
    pub solution_hash: String,
    /// Solution hex
    pub solution: String,
    /// Size in bytes
    pub size: i64,
    /// First used block
    pub first_used_block: i64,
}

/// Solution statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SolutionStats {
    /// Total solutions
    pub total_solutions: i64,
    /// Average size
    pub avg_size: Option<f64>,
    /// Minimum size
    pub min_size: Option<i64>,
    /// Maximum size
    pub max_size: Option<i64>,
    /// Total bytes
    pub total_bytes: Option<i64>,
}

/// Common solution information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CommonSolution {
    /// Solution hash hex
    pub solution_hash_hex: String,
    /// Size
    pub size: i64,
    /// First used block
    pub first_used_block: i64,
    /// Usage count
    pub usage_count: i64,
}

/// Solution complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SolutionComplexityAnalysis {
    /// Complexity category
    pub complexity_category: String,
    /// Solution count
    pub solution_count: i64,
    /// Average size
    pub avg_size: f64,
    /// Minimum size
    pub min_size: i64,
    /// Maximum size
    pub max_size: i64,
    /// Total usage
    pub total_usage: i64,
} 