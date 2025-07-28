use async_graphql::*;
use serde::{Deserialize, Serialize};

/// Puzzle information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PuzzleInfo {
    /// Puzzle ID
    pub id: i64,
    /// Puzzle hash hex
    pub puzzle_hash: String,
    /// Puzzle reveal hex (optional)
    pub puzzle_reveal: Option<String>,
    /// Reveal size in bytes
    pub reveal_size: Option<i64>,
    /// First seen block
    pub first_seen_block: Option<i64>,
}

/// Puzzle statistics
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PuzzleStats {
    /// Total puzzles
    pub total_puzzles: i64,
    /// Puzzles with reveals
    pub puzzles_with_reveals: i64,
    /// Puzzles without reveals
    pub puzzles_without_reveals: i64,
    /// Average reveal size
    pub avg_reveal_size: Option<f64>,
    /// Total reveal bytes
    pub total_reveal_bytes: Option<i64>,
}

/// Common puzzle information
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct CommonPuzzle {
    /// Puzzle hash hex
    pub puzzle_hash_hex: String,
    /// Reveal size
    pub reveal_size: Option<i64>,
    /// First seen block
    pub first_seen_block: Option<i64>,
    /// Usage count
    pub usage_count: i64,
}

/// Puzzle complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PuzzleComplexityAnalysis {
    /// Complexity category
    pub complexity_category: String,
    /// Puzzle count
    pub puzzle_count: i64,
    /// Average size
    pub avg_size: f64,
    /// Minimum size
    pub min_size: i64,
    /// Maximum size
    pub max_size: i64,
    /// Total usage
    pub total_usage: i64,
} 