use crate::api::{ServerClient, RetryPolicy};
use crate::error::ApiError;
use crate::models::ServerConfig;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

/// Connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Reconnecting { attempt: u32 },
    Error(String),
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub last_successful_connection: Option<Instant>,
    pub last_failed_connection: Option<Instant>,
    pub total_reconnect_attempts: u32,
    pub successful_reconnects: u32,
    pub average_response_time: Option<Duration>,
    pub last_response_time: Option<Duration>,
    pub network_quality: NetworkQuality,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            last_successful_connection: None,
            last_failed_connection: None,
            total_reconnect_attempts: 0,
            successful_reconnects: 0,
            average_response_time: None,
            last_response_time: None,
            network_quality: NetworkQuality::Unknown,
        }
    }
}

/// Network quality indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkQuality {
    Excellent,  // < 100ms
    Good,       // 100-300ms
    Fair,       // 300-1000ms
    Poor,       // > 1000ms
    Unknown,
}

impl NetworkQuality {
    pub fn from_response_time(duration: Duration) -> Self {
        let millis = duration.as_millis();
        if millis < 100 {
            NetworkQuality::Excellent
        } else if millis < 300 {
            NetworkQuality::Good
        } else if millis < 1000 {
            NetworkQuality::Fair
        } else {
            NetworkQuality::Poor
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            NetworkQuality::Excellent => "매우 좋음",
            NetworkQuality::Good => "좋음",
            NetworkQuality::Fair => "보통",
            NetworkQuality::Poor => "나쁨",
            NetworkQuality::Unknown => "알 수 없음",
        }
    }
}

/// Connection manager with auto-reconnect capability
pub struct ConnectionManager {
    client: Arc<RwLock<Option<ServerClient>>>,
    config: Arc<RwLock<ServerConfig>>,
    status: Arc<RwLock<ConnectionStatus>>,
    stats: Arc<Mutex<ConnectionStats>>,
    pub reconnect_policy: RetryPolicy,
    health_check_interval: Duration,
    is_monitoring: Arc<Mutex<bool>>,
}

impl ConnectionManager {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
            reconnect_policy: RetryPolicy {
                max_retries: 10,
                initial_delay: Duration::from_secs(2),
                max_delay: Duration::from_secs(120),
                backoff_multiplier: 1.5,
            },
            health_check_interval: Duration::from_secs(30),
            is_monitoring: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get current connection status
    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.read().await.clone()
    }
    
    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.lock().await.clone()
    }
    
    /// Get current network quality
    pub async fn get_network_quality(&self) -> NetworkQuality {
        self.stats.lock().await.network_quality
    }
    
    /// Get server response time
    pub async fn get_response_time(&self) -> Option<Duration> {
        self.stats.lock().await.last_response_time
    }
    
    /// Update server configuration
    pub async fn update_config(&self, config: ServerConfig) {
        *self.config.write().await = config.clone();
        
        // Reconnect with new config
        self.connect().await.ok();
    }
    
    /// Establish connection to server
    pub async fn connect(&self) -> Result<(), ApiError> {
        *self.status.write().await = ConnectionStatus::Connecting;
        
        let config = self.config.read().await.clone();
        
        match ServerClient::new(&config) {
            Ok(client) => {
                // Test connection with health check
                let start = Instant::now();
                match client.check_health().await {
                    Ok(_) => {
                        let response_time = start.elapsed();
                        
                        // Update client and status
                        *self.client.write().await = Some(client);
                        *self.status.write().await = ConnectionStatus::Connected;
                        
                        // Update stats
                        let mut stats = self.stats.lock().await;
                        stats.last_successful_connection = Some(Instant::now());
                        stats.last_response_time = Some(response_time);
                        stats.network_quality = NetworkQuality::from_response_time(response_time);
                        
                        // Update average response time
                        if let Some(avg) = stats.average_response_time {
                            stats.average_response_time = Some(Duration::from_millis(
                                (avg.as_millis() as u64 + response_time.as_millis() as u64) / 2
                            ));
                        } else {
                            stats.average_response_time = Some(response_time);
                        }
                        
                        log::info!("Connected to server successfully (response time: {:?})", response_time);
                        Ok(())
                    }
                    Err(e) => {
                        *self.status.write().await = ConnectionStatus::Error(e.to_string());
                        
                        let mut stats = self.stats.lock().await;
                        stats.last_failed_connection = Some(Instant::now());
                        
                        log::error!("Failed to connect to server: {}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                *self.status.write().await = ConnectionStatus::Error(e.to_string());
                
                let mut stats = self.stats.lock().await;
                stats.last_failed_connection = Some(Instant::now());
                
                log::error!("Failed to create server client: {}", e);
                Err(e)
            }
        }
    }
    
    /// Disconnect from server
    pub async fn disconnect(&self) {
        *self.client.write().await = None;
        *self.status.write().await = ConnectionStatus::Disconnected;
        log::info!("Disconnected from server");
    }
    
    /// Attempt to reconnect with exponential backoff
    pub async fn reconnect(&self) -> Result<(), ApiError> {
        let mut last_error = None;
        
        for attempt in 0..self.reconnect_policy.max_retries {
            *self.status.write().await = ConnectionStatus::Reconnecting { attempt: attempt + 1 };
            
            // Update stats
            {
                let mut stats = self.stats.lock().await;
                stats.total_reconnect_attempts += 1;
            }
            
            log::info!("Reconnection attempt {}/{}", attempt + 1, self.reconnect_policy.max_retries);
            
            match self.connect().await {
                Ok(_) => {
                    // Update stats
                    let mut stats = self.stats.lock().await;
                    stats.successful_reconnects += 1;
                    
                    log::info!("Reconnection successful");
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    if attempt < self.reconnect_policy.max_retries - 1 {
                        let delay = self.reconnect_policy.delay_for_attempt(attempt);
                        log::warn!("Reconnection failed: {}. Retrying in {:?}", e, delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        let error = last_error.unwrap_or_else(|| ApiError::NetworkError("Max reconnection attempts exceeded".to_string()));
        *self.status.write().await = ConnectionStatus::Error(error.to_string());
        Err(error)
    }
    
    /// Get the server client if connected
    pub async fn get_client(&self) -> Option<ServerClient> {
        self.client.read().await.clone()
    }
    
    /// Check if currently connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.status.read().await, ConnectionStatus::Connected)
    }
    
    /// Start monitoring connection health
    pub async fn start_monitoring(&self) {
        let mut is_monitoring = self.is_monitoring.lock().await;
        if *is_monitoring {
            log::warn!("Connection monitoring already running");
            return;
        }
        *is_monitoring = true;
        drop(is_monitoring);
        
        let status = Arc::clone(&self.status);
        let client = Arc::clone(&self.client);
        let stats = Arc::clone(&self.stats);
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let health_check_interval = self.health_check_interval;
        let reconnect_policy = self.reconnect_policy.clone();
        let config = Arc::clone(&self.config);
        
        tokio::spawn(async move {
            log::info!("Connection monitoring started");
            
            loop {
                // Check if monitoring should stop
                if !*is_monitoring.lock().await {
                    log::info!("Connection monitoring stopped");
                    break;
                }
                
                tokio::time::sleep(health_check_interval).await;
                
                // Check connection status
                let current_status = status.read().await.clone();
                if !matches!(current_status, ConnectionStatus::Connected) {
                    continue;
                }
                
                // Perform health check
                if let Some(client_instance) = client.read().await.as_ref() {
                    let start = Instant::now();
                    match client_instance.check_health().await {
                        Ok(_) => {
                            let response_time = start.elapsed();
                            
                            // Update stats
                            let mut stats_guard = stats.lock().await;
                            stats_guard.last_response_time = Some(response_time);
                            stats_guard.network_quality = NetworkQuality::from_response_time(response_time);
                            
                            if let Some(avg) = stats_guard.average_response_time {
                                stats_guard.average_response_time = Some(Duration::from_millis(
                                    (avg.as_millis() as u64 + response_time.as_millis() as u64) / 2
                                ));
                            } else {
                                stats_guard.average_response_time = Some(response_time);
                            }
                            
                            log::debug!("Health check passed (response time: {:?})", response_time);
                        }
                        Err(e) => {
                            log::warn!("Health check failed: {}. Attempting reconnection", e);
                            
                            // Mark as disconnected
                            *status.write().await = ConnectionStatus::Disconnected;
                            *client.write().await = None;
                            
                            // Attempt reconnection
                            for attempt in 0..reconnect_policy.max_retries {
                                *status.write().await = ConnectionStatus::Reconnecting { attempt: attempt + 1 };
                                
                                let config_guard = config.read().await.clone();
                                match ServerClient::new(&config_guard) {
                                    Ok(new_client) => {
                                        match new_client.check_health().await {
                                            Ok(_) => {
                                                *client.write().await = Some(new_client);
                                                *status.write().await = ConnectionStatus::Connected;
                                                
                                                let mut stats_guard = stats.lock().await;
                                                stats_guard.successful_reconnects += 1;
                                                stats_guard.last_successful_connection = Some(Instant::now());
                                                
                                                log::info!("Reconnection successful");
                                                break;
                                            }
                                            Err(e) => {
                                                log::warn!("Reconnection attempt {} failed: {}", attempt + 1, e);
                                                
                                                if attempt < reconnect_policy.max_retries - 1 {
                                                    let delay = reconnect_policy.delay_for_attempt(attempt);
                                                    tokio::time::sleep(delay).await;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to create client during reconnection: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    /// Stop monitoring connection health
    pub async fn stop_monitoring(&self) {
        *self.is_monitoring.lock().await = false;
        log::info!("Connection monitoring stop requested");
    }
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            config: Arc::clone(&self.config),
            status: Arc::clone(&self.status),
            stats: Arc::clone(&self.stats),
            reconnect_policy: self.reconnect_policy.clone(),
            health_check_interval: self.health_check_interval,
            is_monitoring: Arc::clone(&self.is_monitoring),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConnectionType;
    
    #[tokio::test]
    async fn test_connection_manager_creation() {
        let config = ServerConfig {
            server_url: "http://localhost:8080".to_string(),
            connection_type: ConnectionType::LocalNetwork,
            auth_token: None,
            timeout_seconds: 30,
        };
        
        let manager = ConnectionManager::new(config);
        let status = manager.get_status().await;
        assert_eq!(status, ConnectionStatus::Disconnected);
    }
    
    #[tokio::test]
    async fn test_connection_stats() {
        let config = ServerConfig::default();
        let manager = ConnectionManager::new(config);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_reconnect_attempts, 0);
        assert_eq!(stats.successful_reconnects, 0);
        assert!(stats.last_successful_connection.is_none());
    }
    
    #[tokio::test]
    async fn test_update_config() {
        let config = ServerConfig::default();
        let manager = ConnectionManager::new(config);
        
        let new_config = ServerConfig {
            server_url: "http://new-server:8080".to_string(),
            connection_type: ConnectionType::PublicUrl,
            auth_token: Some("token".to_string()),
            timeout_seconds: 60,
        };
        
        manager.update_config(new_config.clone()).await;
        
        let updated_config = manager.config.read().await.clone();
        assert_eq!(updated_config.server_url, "http://new-server:8080");
    }
    
    #[tokio::test]
    async fn test_disconnect() {
        let config = ServerConfig::default();
        let manager = ConnectionManager::new(config);
        
        manager.disconnect().await;
        
        let status = manager.get_status().await;
        assert_eq!(status, ConnectionStatus::Disconnected);
        
        let client = manager.get_client().await;
        assert!(client.is_none());
    }
    
    #[tokio::test]
    async fn test_is_connected() {
        let config = ServerConfig::default();
        let manager = ConnectionManager::new(config);
        
        assert!(!manager.is_connected().await);
        
        *manager.status.write().await = ConnectionStatus::Connected;
        assert!(manager.is_connected().await);
    }
}
