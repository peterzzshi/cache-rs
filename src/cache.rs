use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

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
    F: Fn(
        K,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>>>
                + Send,
        >,
    >,
    G: Fn(&K) -> String,
{
    map: std::sync::RwLock<HashMap<String, Expiring<V>>>,
    load: F,
    get_key_for_map: G,
    _phantom: std::marker::PhantomData<K>,
}

impl<K, V, F, G> Cache<K, V, F, G>
where
    K: Clone + Send + Sync,
    V: Clone + Send + Sync,
    F: Fn(
        K,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>>>
                + Send,
        >,
    >,
    G: Fn(&K) -> String + Send + Sync,
{
    /// Creates a new cache with the given loader and key mapper functions
    pub fn new(load: F, get_key_for_map: G) -> Self {
        Self {
            map: std::sync::RwLock::new(HashMap::new()),
            load,
            get_key_for_map,
            _phantom: std::marker::PhantomData,
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
    pub async fn get_with_expiry(
        &self,
        key: K,
    ) -> Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn load_and_cache_item(
        &self,
        key: K,
        identifier: String,
    ) -> Result<Expiring<V>, Box<dyn std::error::Error + Send + Sync>> {
        let item = (self.load)(key).await?;

        if let Ok(mut map) = self.map.write() {
            map.insert(identifier, item.clone());
        }

        Ok(item)
    }
}
