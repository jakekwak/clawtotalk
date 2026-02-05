use crate::error::{AppError, ApiError, RecoveryAction, ErrorSeverity};
use crate::api::RetryPolicy;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// User notification for errors
#[derive(Debug, Clone)]
pub struct UserNotification {
    pub id: uuid::Uuid,
    pub message: String,
    pub severity: ErrorSeverity,
    pub recovery_actions: Vec<RecoveryAction>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub auto_dismiss: bool,
}

impl UserNotification {
    pub fn new(error: &AppError) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            message: error.user_message(),
            severity: error.severity(),
            recovery_actions: error.recovery_actions(),
            timestamp: chrono::Utc::now(),
            auto_dismiss: matches!(error.severity(), ErrorSeverity::Info | ErrorSeverity::Warning),
        }
    }
    
    pub fn with_message(message: String, severity: ErrorSeverity) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            message,
            severity,
            recovery_actions: vec![],
            timestamp: chrono::Utc::now(),
            auto_dismiss: matches!(severity, ErrorSeverity::Info | ErrorSeverity::Warning),
        }
    }
}

/// Error type for retry policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    Network,
    Audio,
    Api,
    Configuration,
}

/// Global error handler
pub struct ErrorHandler {
    retry_policies: HashMap<ErrorType, RetryPolicy>,
    notifications: Arc<Mutex<Vec<UserNotification>>>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        let mut retry_policies = HashMap::new();
        
        // Network errors: aggressive retry
        retry_policies.insert(ErrorType::Network, RetryPolicy {
            max_retries: 5,
            initial_delay: std::time::Duration::from_secs(1),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
        });
        
        // API errors: moderate retry
        retry_policies.insert(ErrorType::Api, RetryPolicy {
            max_retries: 3,
            initial_delay: std::time::Duration::from_secs(2),
            max_delay: std::time::Duration::from_secs(60),
            backoff_multiplier: 2.0,
        });
        
        // Audio errors: quick retry
        retry_policies.insert(ErrorType::Audio, RetryPolicy {
            max_retries: 2,
            initial_delay: std::time::Duration::from_millis(500),
            max_delay: std::time::Duration::from_secs(5),
            backoff_multiplier: 2.0,
        });
        
        // Configuration errors: no retry
        retry_policies.insert(ErrorType::Configuration, RetryPolicy {
            max_retries: 0,
            initial_delay: std::time::Duration::from_secs(0),
            max_delay: std::time::Duration::from_secs(0),
            backoff_multiplier: 1.0,
        });
        
        Self {
            retry_policies,
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Handle an error with logging and user notification
    pub async fn handle_error(&self, error: AppError) -> Result<(), AppError> {
        // Log the error
        match error.severity() {
            ErrorSeverity::Info => log::info!("Application info: {:?}", error),
            ErrorSeverity::Warning => log::warn!("Application warning: {:?}", error),
            ErrorSeverity::Error => log::error!("Application error: {:?}", error),
            ErrorSeverity::Critical => log::error!("Critical application error: {:?}", error),
        }
        
        // Create user notification
        let notification = UserNotification::new(&error);
        self.add_notification(notification).await;
        
        // Check if error is retryable
        if error.is_retryable() {
            log::info!("Error is retryable, scheduling retry");
            Ok(())
        } else {
            Err(error)
        }
    }
    
    /// Add a notification to the queue
    pub async fn add_notification(&self, notification: UserNotification) {
        let mut notifications = self.notifications.lock().await;
        notifications.push(notification);
        
        // Keep only last 50 notifications
        let len = notifications.len();
        if len > 50 {
            notifications.drain(0..len - 50);
        }
    }
    
    /// Get all notifications
    pub async fn get_notifications(&self) -> Vec<UserNotification> {
        self.notifications.lock().await.clone()
    }
    
    /// Clear all notifications
    pub async fn clear_notifications(&self) {
        self.notifications.lock().await.clear();
    }
    
    /// Remove a specific notification
    pub async fn remove_notification(&self, id: uuid::Uuid) {
        let mut notifications = self.notifications.lock().await;
        notifications.retain(|n| n.id != id);
    }
    
    /// Get retry policy for an error type
    pub fn get_retry_policy(&self, error_type: ErrorType) -> RetryPolicy {
        self.retry_policies
            .get(&error_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Update retry policy for an error type
    pub fn set_retry_policy(&mut self, error_type: ErrorType, policy: RetryPolicy) {
        self.retry_policies.insert(error_type, policy);
    }
    
    /// Schedule a retry for an operation
    pub async fn schedule_retry<F, Fut, T>(
        &self,
        error: AppError,
        operation: F,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        if !error.is_retryable() {
            return Err(error);
        }
        
        let error_type = match &error {
            AppError::Api(ApiError::NetworkError(_)) => ErrorType::Network,
            AppError::Api(_) => ErrorType::Api,
            AppError::Audio(_) => ErrorType::Audio,
            AppError::Configuration(_) => ErrorType::Configuration,
            AppError::Unknown(_) => ErrorType::Network,
        };
        
        let policy = self.get_retry_policy(error_type);
        let delay = policy.delay_for_attempt(0);
        
        log::info!("Scheduling retry in {:?}", delay);
        tokio::time::sleep(delay).await;
        
        operation().await
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ErrorHandler {
    fn clone(&self) -> Self {
        Self {
            retry_policies: self.retry_policies.clone(),
            notifications: Arc::clone(&self.notifications),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AudioError, ApiError};
    
    #[tokio::test]
    async fn test_error_handler_creation() {
        let handler = ErrorHandler::new();
        assert_eq!(handler.retry_policies.len(), 4);
    }
    
    #[tokio::test]
    async fn test_handle_error() {
        let handler = ErrorHandler::new();
        let error = AppError::Audio(AudioError::DeviceNotFound);
        
        let result = handler.handle_error(error).await;
        assert!(result.is_err());
        
        let notifications = handler.get_notifications().await;
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].severity, ErrorSeverity::Critical);
    }
    
    #[tokio::test]
    async fn test_retryable_error() {
        let handler = ErrorHandler::new();
        let error = AppError::Api(ApiError::NetworkError("test".to_string()));
        
        let result = handler.handle_error(error).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_notification_management() {
        let handler = ErrorHandler::new();
        
        let notification = UserNotification::with_message(
            "Test notification".to_string(),
            ErrorSeverity::Info,
        );
        let id = notification.id;
        
        handler.add_notification(notification).await;
        
        let notifications = handler.get_notifications().await;
        assert_eq!(notifications.len(), 1);
        
        handler.remove_notification(id).await;
        
        let notifications = handler.get_notifications().await;
        assert_eq!(notifications.len(), 0);
    }
    
    #[tokio::test]
    async fn test_notification_limit() {
        let handler = ErrorHandler::new();
        
        // Add 60 notifications
        for i in 0..60 {
            let notification = UserNotification::with_message(
                format!("Notification {}", i),
                ErrorSeverity::Info,
            );
            handler.add_notification(notification).await;
        }
        
        let notifications = handler.get_notifications().await;
        assert_eq!(notifications.len(), 50); // Should be capped at 50
    }
    
    #[test]
    fn test_retry_policy() {
        let handler = ErrorHandler::new();
        
        let network_policy = handler.get_retry_policy(ErrorType::Network);
        assert_eq!(network_policy.max_retries, 5);
        
        let audio_policy = handler.get_retry_policy(ErrorType::Audio);
        assert_eq!(audio_policy.max_retries, 2);
    }
}
