use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Audio buffer pool for reusing buffers and reducing allocations
/// Requirement 11.3: Memory usage optimization
pub struct AudioBufferPool {
    pool: Arc<Mutex<VecDeque<Vec<f32>>>>,
    max_pool_size: usize,
    buffer_capacity: usize,
}

impl AudioBufferPool {
    pub fn new(max_pool_size: usize, buffer_capacity: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_pool_size))),
            max_pool_size,
            buffer_capacity,
        }
    }
    
    /// Acquire a buffer from the pool or create a new one
    pub fn acquire(&self) -> Vec<f32> {
        let mut pool = self.pool.lock().unwrap();
        
        if let Some(mut buffer) = pool.pop_front() {
            buffer.clear();
            buffer
        } else {
            Vec::with_capacity(self.buffer_capacity)
        }
    }
    
    /// Return a buffer to the pool for reuse
    pub fn release(&self, buffer: Vec<f32>) {
        let mut pool = self.pool.lock().unwrap();
        
        if pool.len() < self.max_pool_size {
            pool.push_back(buffer);
        }
        // If pool is full, buffer is dropped
    }
    
    /// Get current pool size
    pub fn pool_size(&self) -> usize {
        self.pool.lock().unwrap().len()
    }
    
    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
    }
}

impl Default for AudioBufferPool {
    fn default() -> Self {
        // Default: pool of 10 buffers, each with capacity for 1 second at 16kHz
        Self::new(10, 16000)
    }
}

/// Paginated conversation history for memory efficiency
/// Requirement 11.3: Conversation history pagination
#[derive(Clone, Debug)]
pub struct PaginatedHistory<T> {
    items: Vec<T>,
    page_size: usize,
    max_items: usize,
}

impl<T: Clone> PaginatedHistory<T> {
    pub fn new(page_size: usize, max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            page_size,
            max_items,
        }
    }
    
    /// Add an item to the history
    pub fn push(&mut self, item: T) {
        self.items.push(item);
        
        // Trim old items if we exceed max_items
        if self.items.len() > self.max_items {
            let excess = self.items.len() - self.max_items;
            self.items.drain(0..excess);
        }
    }
    
    /// Get a page of items
    pub fn get_page(&self, page: usize) -> Vec<T> {
        let start = page * self.page_size;
        let end = (start + self.page_size).min(self.items.len());
        
        if start >= self.items.len() {
            Vec::new()
        } else {
            self.items[start..end].to_vec()
        }
    }
    
    /// Get the most recent items (last page)
    pub fn get_recent(&self, count: usize) -> Vec<T> {
        let start = self.items.len().saturating_sub(count);
        self.items[start..].to_vec()
    }
    
    /// Get all items
    pub fn get_all(&self) -> &[T] {
        &self.items
    }
    
    /// Get total number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    /// Get number of pages
    pub fn page_count(&self) -> usize {
        (self.items.len() + self.page_size - 1) / self.page_size
    }
    
    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }
    
    /// Remove old items beyond a certain count
    pub fn trim_to(&mut self, count: usize) {
        if self.items.len() > count {
            let excess = self.items.len() - count;
            self.items.drain(0..excess);
        }
    }
}

impl<T: Clone> Default for PaginatedHistory<T> {
    fn default() -> Self {
        // Default: 50 items per page, max 500 items
        Self::new(50, 500)
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    audio_buffer_count: Arc<Mutex<usize>>,
    conversation_item_count: Arc<Mutex<usize>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            audio_buffer_count: Arc::new(Mutex::new(0)),
            conversation_item_count: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn track_audio_buffer(&self, size: usize) {
        let mut count = self.audio_buffer_count.lock().unwrap();
        *count += size;
    }
    
    pub fn release_audio_buffer(&self, size: usize) {
        let mut count = self.audio_buffer_count.lock().unwrap();
        *count = count.saturating_sub(size);
    }
    
    pub fn track_conversation_item(&self) {
        let mut count = self.conversation_item_count.lock().unwrap();
        *count += 1;
    }
    
    pub fn get_audio_buffer_count(&self) -> usize {
        *self.audio_buffer_count.lock().unwrap()
    }
    
    pub fn get_conversation_item_count(&self) -> usize {
        *self.conversation_item_count.lock().unwrap()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_buffer_pool() {
        let pool = AudioBufferPool::new(5, 1000);
        
        // Acquire buffers
        let buf1 = pool.acquire();
        let buf2 = pool.acquire();
        
        assert_eq!(buf1.capacity(), 1000);
        assert_eq!(buf2.capacity(), 1000);
        assert_eq!(pool.pool_size(), 0);
        
        // Release buffers
        pool.release(buf1);
        pool.release(buf2);
        
        assert_eq!(pool.pool_size(), 2);
        
        // Reuse buffer
        let buf3 = pool.acquire();
        assert_eq!(pool.pool_size(), 1);
        assert_eq!(buf3.capacity(), 1000);
    }
    
    #[test]
    fn test_audio_buffer_pool_max_size() {
        let pool = AudioBufferPool::new(2, 1000);
        
        // Release more buffers than max size
        for _ in 0..5 {
            pool.release(Vec::with_capacity(1000));
        }
        
        // Pool should not exceed max size
        assert_eq!(pool.pool_size(), 2);
    }
    
    #[test]
    fn test_paginated_history() {
        let mut history = PaginatedHistory::new(10, 100);
        
        // Add items
        for i in 0..25 {
            history.push(i);
        }
        
        assert_eq!(history.len(), 25);
        assert_eq!(history.page_count(), 3);
        
        // Get first page
        let page0 = history.get_page(0);
        assert_eq!(page0.len(), 10);
        assert_eq!(page0[0], 0);
        assert_eq!(page0[9], 9);
        
        // Get second page
        let page1 = history.get_page(1);
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0], 10);
        
        // Get last page
        let page2 = history.get_page(2);
        assert_eq!(page2.len(), 5);
        assert_eq!(page2[0], 20);
    }
    
    #[test]
    fn test_paginated_history_max_items() {
        let mut history = PaginatedHistory::new(10, 20);
        
        // Add more items than max
        for i in 0..30 {
            history.push(i);
        }
        
        // Should only keep last 20 items
        assert_eq!(history.len(), 20);
        assert_eq!(history.get_all()[0], 10); // First item should be 10
    }
    
    #[test]
    fn test_paginated_history_recent() {
        let mut history = PaginatedHistory::new(10, 100);
        
        for i in 0..25 {
            history.push(i);
        }
        
        let recent = history.get_recent(5);
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0], 20);
        assert_eq!(recent[4], 24);
    }
    
    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();
        
        tracker.track_audio_buffer(1000);
        tracker.track_audio_buffer(2000);
        assert_eq!(tracker.get_audio_buffer_count(), 3000);
        
        tracker.release_audio_buffer(1000);
        assert_eq!(tracker.get_audio_buffer_count(), 2000);
        
        tracker.track_conversation_item();
        tracker.track_conversation_item();
        assert_eq!(tracker.get_conversation_item_count(), 2);
    }
}
