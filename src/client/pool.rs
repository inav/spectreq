//! Connection pooling for Spectre client
//!
//! This module provides connection pooling for improved performance
//! by reusing connections across requests to the same host.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_rustls::client::TlsStream;

/// Connection wrapper for pooled connections
#[derive(Debug)]
pub enum PooledConnection {
    Tls(Box<TlsStream<TcpStream>>),
    Plain(TcpStream),
}

impl PooledConnection {
    /// Check if the connection is still valid
    pub fn is_valid(&self) -> bool {
        // For now, we'll assume connections are valid
        // A more sophisticated implementation could check
        // if the connection is still alive
        true
    }

    /// Get the age of the connection
    pub fn age(&self) -> Option<Duration> {
        // Track connection age for eviction
        None
    }
}

/// Connection pool entry with metadata
struct PoolEntry {
    connection: PooledConnection,
    created_at: Instant,
    last_used: Instant,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections per host
    pub max_connections_per_host: usize,
    /// Maximum number of idle connections to keep
    pub max_idle_connections: usize,
    /// Idle timeout before closing an unused connection
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Option<Duration>,
    /// Enable connection pooling
    pub enabled: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 100,
            max_idle_connections: 10,
            idle_timeout: Duration::from_secs(90),
            max_lifetime: Some(Duration::from_secs(300)),
            enabled: true,
        }
    }
}

impl PoolConfig {
    /// Create a new pool config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of connections per host
    pub fn max_connections_per_host(mut self, max: usize) -> Self {
        self.max_connections_per_host = max;
        self
    }

    /// Set the maximum number of idle connections
    pub fn max_idle_connections(mut self, max: usize) -> Self {
        self.max_idle_connections = max;
        self
    }

    /// Set the idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set the maximum connection lifetime
    pub fn max_lifetime(mut self, lifetime: Option<Duration>) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Enable or disable connection pooling
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Host pool for a specific host
struct HostPool {
    connections: Vec<PoolEntry>,
    active_connections: usize,
}

impl HostPool {
    fn new() -> Self {
        Self {
            connections: Vec::new(),
            active_connections: 0,
        }
    }

    /// Get an idle connection from the pool
    fn get_idle_connection(&mut self, config: &PoolConfig) -> Option<PooledConnection> {
        let now = Instant::now();
        let mut index = None;

        for (i, entry) in self.connections.iter().enumerate() {
            // Check if connection is still valid
            if !entry.connection.is_valid() {
                continue;
            }

            // Check idle timeout
            if now.duration_since(entry.last_used) > config.idle_timeout {
                continue;
            }

            // Check max lifetime
            if let Some(max_lifetime) = config.max_lifetime {
                if now.duration_since(entry.created_at) > max_lifetime {
                    continue;
                }
            }

            index = Some(i);
            break;
        }

        if let Some(i) = index {
            let entry = self.connections.remove(i);
            Some(entry.connection)
        } else {
            None
        }
    }

    /// Add a connection to the pool
    fn add_connection(&mut self, connection: PooledConnection, config: &PoolConfig) {
        // Don't add if we have too many idle connections
        if self.connections.len() >= config.max_idle_connections {
            return;
        }

        let entry = PoolEntry {
            connection,
            created_at: Instant::now(),
            last_used: Instant::now(),
        };

        self.connections.push(entry);
    }

    /// Clean up expired connections
    fn cleanup(&mut self, config: &PoolConfig) {
        let now = Instant::now();
        self.connections.retain(|entry| {
            // Keep valid, non-expired connections
            if !entry.connection.is_valid() {
                return false;
            }

            let idle = now.duration_since(entry.last_used);
            let lifetime = now.duration_since(entry.created_at);

            idle < config.idle_timeout && config.max_lifetime.is_none_or(|max| lifetime < max)
        });
    }

    /// Get the number of connections in the pool
    fn len(&self) -> usize {
        self.connections.len()
    }

    /// Get the number of active connections
    fn active_count(&self) -> usize {
        self.active_connections
    }
}

/// Connection pool
pub struct ConnectionPool {
    pools: Arc<Mutex<HashMap<String, HostPool>>>,
    config: PoolConfig,
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration
    pub fn new(config: PoolConfig) -> Self {
        Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Create a connection pool with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PoolConfig::default())
    }

    /// Get a connection for the given host
    pub async fn get(&self, host: &str) -> Option<PooledConnection> {
        if !self.config.enabled {
            return None;
        }

        let mut pools = self.pools.lock().await;
        let pool = pools.entry(host.to_string()).or_insert_with(HostPool::new);

        // Try to get an idle connection
        if let Some(conn) = pool.get_idle_connection(&self.config) {
            // Update last used time
            for entry in &mut pool.connections {
                if entry.connection.age().is_some() {
                    entry.last_used = Instant::now();
                }
            }
            Some(conn)
        } else {
            None
        }
    }

    /// Put a connection back into the pool
    pub async fn put(&self, host: &str, connection: PooledConnection) {
        if !self.config.enabled {
            return;
        }

        let mut pools = self.pools.lock().await;
        let pool = pools.entry(host.to_string()).or_insert_with(HostPool::new);

        // Only add if the connection is valid
        if connection.is_valid() {
            pool.add_connection(connection, &self.config);
        }
    }

    /// Clean up expired connections
    pub async fn cleanup(&self) {
        if !self.config.enabled {
            return;
        }

        let mut pools = self.pools.lock().await;
        for pool in pools.values_mut() {
            pool.cleanup(&self.config);
        }
    }

    /// Get pool statistics
    pub async fn stats(&self, host: &str) -> PoolStats {
        let pools = self.pools.lock().await;
        if let Some(pool) = pools.get(host) {
            PoolStats {
                idle_connections: pool.len(),
                active_connections: pool.active_count(),
            }
        } else {
            PoolStats {
                idle_connections: 0,
                active_connections: 0,
            }
        }
    }

    /// Get the pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Remove all connections for a host
    pub async fn remove_host(&self, host: &str) {
        let mut pools = self.pools.lock().await;
        pools.remove(host);
    }

    /// Clear the entire pool
    pub async fn clear(&self) {
        let mut pools = self.pools.lock().await;
        pools.clear();
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Number of idle connections
    pub idle_connections: usize,
    /// Number of active connections
    pub active_connections: usize,
}

/// Background cleanup task for the connection pool
pub async fn cleanup_task(pool: ConnectionPool, interval: Duration) {
    let mut interval = tokio::time::interval(interval);
    loop {
        interval.tick().await;
        pool.cleanup().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections_per_host, 100);
        assert_eq!(config.max_idle_connections, 10);
        assert!(config.enabled);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new()
            .max_connections_per_host(50)
            .idle_timeout(Duration::from_secs(60));

        assert_eq!(config.max_connections_per_host, 50);
        assert_eq!(config.idle_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_pool_disabled() {
        let config = PoolConfig::new().enabled(false);
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_connection_pool_stats() {
        let pool = ConnectionPool::with_defaults();
        let stats = pool.stats("example.com").await;
        assert_eq!(stats.idle_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_clear() {
        let pool = ConnectionPool::with_defaults();
        pool.clear().await;
        // Should not panic
    }
}
