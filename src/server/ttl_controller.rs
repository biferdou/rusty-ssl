use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{Interval, interval};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub ip: IpAddr,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub ttl: Duration,
    pub request_count: u64,
}

impl ConnectionInfo {
    pub fn new(ip: IpAddr, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            ip,
            established_at: now,
            last_activity: now,
            ttl,
            request_count: 1,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.last_activity.elapsed() > self.ttl
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
        self.request_count += 1;
    }

    pub fn time_until_expiry(&self) -> Option<Duration> {
        let elapsed = self.last_activity.elapsed();
        if elapsed >= self.ttl {
            None
        } else {
            Some(self.ttl - elapsed)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtlStats {
    pub active_connections: usize,
    pub total_connections: u64,
    pub expired_connections: u64,
    pub average_ttl_secs: u64,
}

pub struct TtlController {
    connections: Arc<DashMap<IpAddr, ConnectionInfo>>,
    default_ttl: Duration,
    max_ttl: Duration,
    total_connections: u64,
    expired_connections: u64,
    cleanup_interval: Interval,
}

impl TtlController {
    pub fn new(default_ttl: Duration, max_ttl: Duration, cleanup_interval: Duration) -> Self {
        info!(
            "Initializing TTL controller with default TTL: {:?}, max TTL: {:?}",
            default_ttl, max_ttl
        );

        Self {
            connections: Arc::new(DashMap::new()),
            default_ttl,
            max_ttl,
            total_connections: 0,
            expired_connections: 0,
            cleanup_interval: interval(cleanup_interval),
        }
    }

    pub fn register_connection(&mut self, ip: IpAddr) -> Uuid {
        // Calculate adaptive TTL based on existing connection patterns
        let ttl = self.calculate_adaptive_ttl(ip);

        let connection = ConnectionInfo::new(ip, ttl);
        let connection_id = connection.id;

        // Update existing connection or insert new one
        match self.connections.entry(ip) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                entry.get_mut().update_activity();
                debug!(
                    "Updated existing connection for IP: {}, ID: {}",
                    ip, connection_id
                );
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(connection);
                self.total_connections += 1;
                info!(
                    "New connection registered for IP: {}, ID: {}, TTL: {:?}",
                    ip, connection_id, ttl
                );
            }
        }

        connection_id
    }

    fn calculate_adaptive_ttl(&self, ip: IpAddr) -> Duration {
        // Check if this IP has had recent connections
        if let Some(existing) = self.connections.get(&ip) {
            // If the connection is active and has high request count, extend TTL
            if existing.request_count > 10 && !existing.is_expired() {
                let extended_ttl = self.default_ttl.mul_f32(1.5);
                if extended_ttl <= self.max_ttl {
                    return extended_ttl;
                }
            }
        }

        self.default_ttl
    }

    pub fn update_connection_activity(&self, ip: IpAddr) -> bool {
        if let Some(mut connection) = self.connections.get_mut(&ip) {
            connection.update_activity();
            debug!("Updated activity for IP: {}", ip);
            true
        } else {
            warn!("Attempted to update non-existent connection for IP: {}", ip);
            false
        }
    }

    pub fn get_connection_info(&self, ip: IpAddr) -> Option<ConnectionInfo> {
        self.connections.get(&ip).map(|entry| entry.clone())
    }

    pub fn get_stats(&self) -> TtlStats {
        let active_connections = self.connections.len();
        let total_ttl_secs: u64 = self
            .connections
            .iter()
            .map(|entry| entry.ttl.as_secs())
            .sum();

        let average_ttl_secs = if active_connections > 0 {
            total_ttl_secs / active_connections as u64
        } else {
            self.default_ttl.as_secs()
        };

        TtlStats {
            active_connections,
            total_connections: self.total_connections,
            expired_connections: self.expired_connections,
            average_ttl_secs,
        }
    }

    pub async fn start_cleanup_task(&mut self) {
        info!("Starting TTL cleanup task");

        loop {
            self.cleanup_interval.tick().await;
            self.cleanup_expired_connections().await;
        }
    }

    async fn cleanup_expired_connections(&mut self) {
        let mut expired_ips = Vec::new();

        // Find expired connections
        for entry in self.connections.iter() {
            if entry.is_expired() {
                expired_ips.push(*entry.key());
            }
        }

        // Remove expired connections
        let mut cleaned_count = 0;
        for ip in expired_ips {
            if let Some((_, connection)) = self.connections.remove(&ip) {
                cleaned_count += 1;
                self.expired_connections += 1;
                debug!(
                    "Cleaned up expired connection for IP: {}, ID: {}, Duration: {:?}",
                    ip,
                    connection.id,
                    connection.established_at.elapsed()
                );
            }
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} expired connections", cleaned_count);
        }

        // Log periodic stats
        let stats = self.get_stats();
        debug!(
            "TTL Stats - Active: {}, Total: {}, Expired: {}, Avg TTL: {}s",
            stats.active_connections,
            stats.total_connections,
            stats.expired_connections,
            stats.average_ttl_secs
        );
    }

    pub fn get_connections_snapshot(&self) -> Vec<(IpAddr, ConnectionInfo)> {
        self.connections
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    pub fn force_cleanup_connection(&mut self, ip: IpAddr) -> bool {
        if let Some((_, connection)) = self.connections.remove(&ip) {
            self.expired_connections += 1;
            info!(
                "Force cleaned connection for IP: {}, ID: {}",
                ip, connection.id
            );
            true
        } else {
            false
        }
    }
}
