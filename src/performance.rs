use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, OnceLock};
use log::{info, warn};

/// Global performance monitor instance
static GLOBAL_PERF_MONITOR: OnceLock<PerformanceMonitor> = OnceLock::new();

/// Get or initialize the global performance monitor
pub fn get_performance_monitor() -> &'static PerformanceMonitor {
    GLOBAL_PERF_MONITOR.get_or_init(|| PerformanceMonitor::new())
}

/// Performance metrics for the application
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub startup_time: Option<Duration>,
    pub recording_start_latency: Option<Duration>,
    pub last_api_call_duration: Option<Duration>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            startup_time: None,
            recording_start_latency: None,
            last_api_call_duration: None,
        }
    }
}

/// Performance monitor for tracking app performance
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    startup_instant: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            startup_instant: Instant::now(),
        }
    }
    
    /// Mark app startup as complete and record the time
    pub fn mark_startup_complete(&self) {
        let elapsed = self.startup_instant.elapsed();
        
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.startup_time = Some(elapsed);
        }
        
        info!("App startup completed in {:?}", elapsed);
        
        // Requirement 11.1: App startup time should be < 3 seconds
        if elapsed > Duration::from_secs(3) {
            warn!("Startup time exceeded 3 seconds: {:?}", elapsed);
        }
    }
    
    /// Record recording start latency
    pub fn record_recording_latency(&self, latency: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.recording_start_latency = Some(latency);
        }
        
        info!("Recording start latency: {:?}", latency);
        
        // Requirement 11.2: Recording start latency should be < 100ms
        if latency > Duration::from_millis(100) {
            warn!("Recording latency exceeded 100ms: {:?}", latency);
        }
    }
    
    /// Record API call duration
    pub fn record_api_call(&self, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.last_api_call_duration = Some(duration);
        }
        
        info!("API call completed in {:?}", duration);
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Measure execution time of a function
    pub fn measure<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        
        info!("{} took {:?}", name, elapsed);
        result
    }
    
    /// Measure async execution time
    pub async fn measure_async<F, Fut, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let elapsed = start.elapsed();
        
        info!("{} took {:?}", name, elapsed);
        result
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Lazy initialization helper for deferred loading
pub struct LazyInit<T> {
    value: Arc<Mutex<Option<T>>>,
    initializer: Arc<Mutex<Option<Box<dyn FnOnce() -> T + Send>>>>,
}

impl<T> LazyInit<T> {
    pub fn new<F>(initializer: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            value: Arc::new(Mutex::new(None)),
            initializer: Arc::new(Mutex::new(Some(Box::new(initializer)))),
        }
    }
    
    /// Get or initialize the value
    pub fn get_or_init(&self) -> T
    where
        T: Clone,
    {
        let mut value = self.value.lock().unwrap();
        
        if value.is_none() {
            let mut init = self.initializer.lock().unwrap();
            if let Some(initializer) = init.take() {
                *value = Some(initializer());
            }
        }
        
        value.as_ref().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        // Simulate startup
        thread::sleep(Duration::from_millis(10));
        monitor.mark_startup_complete();
        
        let metrics = monitor.get_metrics();
        assert!(metrics.startup_time.is_some());
        assert!(metrics.startup_time.unwrap() >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_recording_latency() {
        let monitor = PerformanceMonitor::new();
        
        let latency = Duration::from_millis(50);
        monitor.record_recording_latency(latency);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.recording_start_latency, Some(latency));
    }
    
    #[test]
    fn test_measure() {
        let monitor = PerformanceMonitor::new();
        
        let result = monitor.measure("test_operation", || {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_lazy_init() {
        let lazy = LazyInit::new(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        let start = Instant::now();
        let value = lazy.get_or_init();
        let first_call = start.elapsed();
        
        assert_eq!(value, 42);
        assert!(first_call >= Duration::from_millis(10));
        
        // Second call should be instant
        let start = Instant::now();
        let value = lazy.get_or_init();
        let second_call = start.elapsed();
        
        assert_eq!(value, 42);
        assert!(second_call < Duration::from_millis(5));
    }
}
