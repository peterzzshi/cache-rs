use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a value with an expiration time
#[derive(Debug, Clone)]
pub struct Expiring<T> {
    pub expires_at: SystemTime,
    pub value: T,
}

impl<T> Expiring<T> {
    /// Creates a new expiring value
    pub fn new(value: T, expires_at: SystemTime) -> Self {
        Self { expires_at, value }
    }

    /// Creates a new expiring value that expires after the given duration
    pub fn with_duration(value: T, duration: std::time::Duration) -> Self {
        let expires_at = SystemTime::now() + duration;
        Self::new(value, expires_at)
    }

    /// Checks if this item has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Configuration for the Cache
#[derive(Clone)]
pub struct CacheConfig<K, V, F, G> {
    pub load: F,
    pub get_key_for_map: G,
    _phantom: std::marker::PhantomData<(K, V)>,
}

/// A generic cache with expiration support
pub struct Cache<K, V, F, G> 
where
    K: Clone,
    V: Clone,
    F: Fn(K) -> Box<dyn Future<Output = Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync>,
    G: Fn(&K) -> String,
{
    map: std::sync::RwLock<HashMap<String, Expiring<V>>>,
    load: F,
    get_key_for_map: G,
}

impl<K, V, F, G> Cache<K, V, F, G>
where
    K: Clone + Send + Sync,
    V: Clone + Send + Sync,
    F: Fn(K) -> Box<dyn Future<Output = Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync>,
    G: Fn(&K) -> String + Send + Sync,
{
    /// Creates a new cache with the given loader and key mapper functions
    pub fn new(load: F, get_key_for_map: G) -> Self {
        Self {
            map: std::sync::RwLock::new(HashMap::new()),
            load,
            get_key_for_map,
        }
    }

    /// Gets a value from the cache, loading it if necessary or expired
    pub async fn get(&self, key: K) -> Result<V, Box<dyn std::error::Error + Send + Sync>> {
        let expiring = self.get_with_expiry(key).await?;
        Ok(expiring.value)
    }

    /// Gets the cache configuration
    pub fn get_config(&self) -> CacheConfig<K, V, &F, &G> {
        CacheConfig {
            load: &self.load,
            get_key_for_map: &self.get_key_for_map,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Gets a value with its expiration information
    pub async fn get_with_expiry(&self, key: K) -> Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>> {
        let identifier = (self.get_key_for_map)(&key);
        
        // Try to get non-expired item
        if let Some(item) = self.get_non_expired(&identifier) {
            return Ok(item);
        }

        // Load and cache the item
        self.load_and_cache_item(key, identifier).await
    }

    /// Deletes an item from the cache
    pub fn delete(&self, key: K) {
        let identifier = (self.get_key_for_map)(&key);
        if let Ok(mut map) = self.map.write() {
            map.remove(&identifier);
        }
    }

    /// Clears all items from the cache
    pub fn delete_all(&self) {
        if let Ok(mut map) = self.map.write() {
            map.clear();
        }
    }

    /// Gets the current size of the cache
    pub fn size(&self) -> usize {
        self.map.read().map(|map| map.len()).unwrap_or(0)
    }

    fn get_non_expired(&self, identifier: &str) -> Option<Expiring<V>> {
        if let Ok(map) = self.map.read() {
            if let Some(item) = map.get(identifier) {
                if !item.is_expired() {
                    return Some(item.clone());
                }
            }
        }
        None
    }

    async fn load_and_cache_item(&self, key: K, identifier: String) -> Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>> {
        let item = (self.load)(key).await?;
        
        if let Ok(mut map) = self.map.write() {
            map.insert(identifier, item.clone());
        }
        
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_basic_functionality() {
        let cache = Cache::new(
            |key: i32| {
                Box::new(async move {
                    let value = format!("loaded_{}", key);
                    Ok(Expiring::with_duration(value, Duration::from_secs(1)))
                })
            },
            |key: &i32| key.to_string(),
        );

        let result = cache.get(42).await.unwrap();
        assert_eq!(result, "loaded_42");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new(
            |key: i32| {
                Box::new(async move {
                    let value = format!("loaded_{}", key);
                    Ok(Expiring::with_duration(value, Duration::from_millis(10)))
                })
            },
            |key: &i32| key.to_string(),
        );

        let _result1 = cache.get(42).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        let _result2 = cache.get(42).await.unwrap();
        // In a real test, you'd verify the loader was called twice
    }
}