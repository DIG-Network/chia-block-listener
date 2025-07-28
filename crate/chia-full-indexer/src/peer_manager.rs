use crate::types::*;
use crate::error::{IndexerError, Result};
use dns_discovery::{DnsDiscovery, PeerAddress};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use chrono::{DateTime, Utc};

/// Manages peer discovery and connection state
pub struct PeerManager {
    config: IndexerConfig,
    dns_discovery: DnsDiscovery,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    discovered_peers: Arc<RwLock<Vec<PeerAddress>>>,
}

impl PeerManager {
    /// Create a new peer manager
    pub async fn new(config: IndexerConfig) -> Result<Self> {
        let dns_discovery = DnsDiscovery::new().await
            .map_err(|e| IndexerError::DnsDiscovery(e))?;

        Ok(Self {
            config,
            dns_discovery,
            peers: Arc::new(RwLock::new(HashMap::new())),
            discovered_peers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Discover peers using DNS introducers
    pub async fn discover_peers(&self) -> Result<Vec<PeerAddress>> {
        info!("Discovering peers using DNS introducers...");

        let peers = match self.config.blockchain.network_id.as_str() {
            "mainnet" => {
                self.dns_discovery.discover_mainnet_peers().await
                    .map_err(|e| IndexerError::DnsDiscovery(e))?
            }
            "testnet11" => {
                self.dns_discovery.discover_testnet11_peers().await
                    .map_err(|e| IndexerError::DnsDiscovery(e))?
            }
            _ => {
                // Use custom introducers
                let introducer_refs: Vec<&str> = self.config.network.introducers
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                
                self.dns_discovery.discover_peers(&introducer_refs, self.config.network.default_port).await
                    .map_err(|e| IndexerError::DnsDiscovery(e))?
            }
        };

        let total_peers = peers.ipv4_peers.len() + peers.ipv6_peers.len();
        info!("Discovered {} IPv4 peers and {} IPv6 peers (total: {})", 
              peers.ipv4_peers.len(), peers.ipv6_peers.len(), total_peers);

        // Combine IPv4 and IPv6 peers
        let mut all_peers = peers.ipv4_peers;
        all_peers.extend(peers.ipv6_peers);

        // Store discovered peers
        {
            let mut discovered = self.discovered_peers.write().await;
            *discovered = all_peers.clone();
        }

        // Initialize peer info entries
        {
            let mut peer_map = self.peers.write().await;
            for peer in &all_peers {
                let peer_id = format!("{}:{}", peer.host, peer.port);
                peer_map.entry(peer_id.clone()).or_insert_with(|| PeerInfo {
                    host: peer.host.clone(),
                    port: peer.port,
                    is_connected: false,
                    last_activity: None,
                    connection_attempts: 0,
                    last_error: None,
                });
            }
        }

        Ok(all_peers)
    }

    /// Get the best peers for connection based on connection history
    pub async fn get_best_peers(&self, count: usize) -> Result<Vec<PeerAddress>> {
        let peer_map = self.peers.read().await;
        let discovered = self.discovered_peers.read().await;

        // Sort peers by reliability (fewer connection attempts and more recent activity)
        let mut peer_scores: Vec<(f64, &PeerAddress)> = Vec::new();

        for peer in discovered.iter() {
            let peer_id = format!("{}:{}", peer.host, peer.port);
            if let Some(info) = peer_map.get(&peer_id) {
                let mut score = 100.0;

                // Penalize failed connection attempts
                score -= info.connection_attempts as f64 * 10.0;

                // Bonus for recent activity
                if let Some(last_activity) = info.last_activity {
                    let hours_ago = (Utc::now() - last_activity).num_hours() as f64;
                    score += (24.0 - hours_ago.min(24.0)) * 2.0;
                }

                // Bonus for being currently connected
                if info.is_connected {
                    score += 50.0;
                }

                // Penalize if there's a recent error
                if info.last_error.is_some() {
                    score -= 25.0;
                }

                peer_scores.push((score, peer));
            } else {
                // New peer, give it a neutral score
                peer_scores.push((50.0, peer));
            }
        }

        // Sort by score (descending) and take the best ones
        peer_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        let best_peers = peer_scores
            .into_iter()
            .take(count)
            .map(|(_, peer)| peer.clone())
            .collect();

        debug!("Selected {} best peers out of {} discovered", best_peers.len(), discovered.len());
        Ok(best_peers)
    }

    /// Mark a peer as connected
    pub async fn mark_peer_connected(&self, host: &str, port: u16) -> Result<()> {
        let peer_id = format!("{}:{}", host, port);
        let mut peer_map = self.peers.write().await;
        
        if let Some(info) = peer_map.get_mut(&peer_id) {
            info.is_connected = true;
            info.last_activity = Some(Utc::now());
            info.last_error = None;
            debug!("Marked peer {} as connected", peer_id);
        } else {
            warn!("Attempted to mark unknown peer {} as connected", peer_id);
        }

        Ok(())
    }

    /// Mark a peer as disconnected
    pub async fn mark_peer_disconnected(&self, host: &str, port: u16, error: Option<String>) -> Result<()> {
        let peer_id = format!("{}:{}", host, port);
        let mut peer_map = self.peers.write().await;
        
        if let Some(info) = peer_map.get_mut(&peer_id) {
            info.is_connected = false;
            info.last_error = error;
            debug!("Marked peer {} as disconnected", peer_id);
        } else {
            warn!("Attempted to mark unknown peer {} as disconnected", peer_id);
        }

        Ok(())
    }

    /// Record a connection attempt
    pub async fn record_connection_attempt(&self, host: &str, port: u16) -> Result<()> {
        let peer_id = format!("{}:{}", host, port);
        let mut peer_map = self.peers.write().await;
        
        if let Some(info) = peer_map.get_mut(&peer_id) {
            info.connection_attempts += 1;
            debug!("Recorded connection attempt #{} for peer {}", info.connection_attempts, peer_id);
        } else {
            // Create new peer info
            peer_map.insert(peer_id.clone(), PeerInfo {
                host: host.to_string(),
                port,
                is_connected: false,
                last_activity: None,
                connection_attempts: 1,
                last_error: None,
            });
            debug!("Created new peer info for {} with first connection attempt", peer_id);
        }

        Ok(())
    }

    /// Update peer activity
    pub async fn update_peer_activity(&self, host: &str, port: u16) -> Result<()> {
        let peer_id = format!("{}:{}", host, port);
        let mut peer_map = self.peers.write().await;
        
        if let Some(info) = peer_map.get_mut(&peer_id) {
            info.last_activity = Some(Utc::now());
        }

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let peer_map = self.peers.read().await;
        let discovered = self.discovered_peers.read().await;

        let total_peers = discovered.len();
        let connected_peers = peer_map.values().filter(|info| info.is_connected).count();
        let failed_peers = peer_map.values().filter(|info| info.last_error.is_some()).count();
        let avg_attempts = if !peer_map.is_empty() {
            peer_map.values().map(|info| info.connection_attempts).sum::<u32>() as f64 / peer_map.len() as f64
        } else {
            0.0
        };

        ConnectionStats {
            total_discovered: total_peers,
            currently_connected: connected_peers,
            connection_failures: failed_peers,
            average_connection_attempts: avg_attempts,
        }
    }

    /// Get detailed peer information
    pub async fn get_peer_info(&self) -> Vec<PeerInfo> {
        let peer_map = self.peers.read().await;
        peer_map.values().cloned().collect()
    }

    /// Resolve additional peers for a specific hostname
    pub async fn resolve_hostname(&self, hostname: &str, port: u16) -> Result<Vec<PeerAddress>> {
        debug!("Resolving additional peers for {}:{}", hostname, port);

        let result = self.dns_discovery.resolve_both(hostname, port).await
            .map_err(|e| IndexerError::DnsDiscovery(e))?;

        let mut all_peers = result.ipv4_peers;
        all_peers.extend(result.ipv6_peers);

        debug!("Resolved {} additional peers for {}", all_peers.len(), hostname);
        Ok(all_peers)
    }

    /// Clean up old peer information
    pub async fn cleanup_old_peers(&self, max_age_hours: i64) -> Result<usize> {
        let mut peer_map = self.peers.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours);
        
        let initial_count = peer_map.len();
        peer_map.retain(|_, info| {
            // Keep connected peers
            if info.is_connected {
                return true;
            }
            
            // Keep peers with recent activity
            if let Some(last_activity) = info.last_activity {
                last_activity > cutoff_time
            } else {
                // Keep peers with few connection attempts (might be worth retrying)
                info.connection_attempts < 5
            }
        });
        
        let removed_count = initial_count - peer_map.len();
        if removed_count > 0 {
            info!("Cleaned up {} old peer entries", removed_count);
        }
        
        Ok(removed_count)
    }

    /// Get recommended peers for specific use cases
    pub async fn get_peers_for_historical_sync(&self) -> Result<Vec<PeerAddress>> {
        self.get_best_peers(self.config.blockchain.max_peers_historical).await
    }

    pub async fn get_peers_for_realtime_sync(&self) -> Result<Vec<PeerAddress>> {
        self.get_best_peers(self.config.blockchain.max_peers_realtime).await
    }

    /// Force refresh peer discovery
    pub async fn refresh_peer_discovery(&self) -> Result<Vec<PeerAddress>> {
        info!("Forcing refresh of peer discovery...");
        self.discover_peers().await
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_discovered: usize,
    pub currently_connected: usize,
    pub connection_failures: usize,
    pub average_connection_attempts: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_manager_creation() {
        let config = IndexerConfig::default();
        let peer_manager = PeerManager::new(config).await;
        assert!(peer_manager.is_ok());
    }

    #[tokio::test]
    async fn test_peer_state_management() {
        let config = IndexerConfig::default();
        let peer_manager = PeerManager::new(config).await.unwrap();

        // Test recording connection attempt
        peer_manager.record_connection_attempt("test.example.com", 8444).await.unwrap();
        
        // Test marking as connected
        peer_manager.mark_peer_connected("test.example.com", 8444).await.unwrap();
        
        // Test updating activity
        peer_manager.update_peer_activity("test.example.com", 8444).await.unwrap();
        
        // Test marking as disconnected
        peer_manager.mark_peer_disconnected("test.example.com", 8444, Some("Test error".to_string())).await.unwrap();
        
        let stats = peer_manager.get_connection_stats().await;
        assert_eq!(stats.total_discovered, 0); // No discovery happened
        assert_eq!(stats.currently_connected, 0);
    }
} 