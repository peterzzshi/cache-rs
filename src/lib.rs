//! # Cache-RS
//! 
//! A generic cache implementation with expiration support for Rust applications.
//! 
//! ## Features
//! 
//! - Generic key-value caching with custom types
//! - Automatic expiration handling
//! - Async support with configurable loaders
//! - Thread-safe operations
//! - Customizable key mapping
//! 
//! ## Quick Start
//! 
//! ```rust
//! use cache_rs::{Cache, Expiring};
//! use std::time::Duration;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let cache = Cache::new(
//!     |key: i32| {
//!         Box::pin(async move {
//!             let value = format!("loaded_{}", key);
//!             Ok(Expiring::with_duration(value, Duration::from_secs(60)))
//!         })
//!     },
//!     |key: &i32| key.to_string(),
//! );
//! 
//! let value = cache.get(42).await?;
//! println!("Cached value: {}", value);
//! # Ok(())
//! # }
//! ```

pub mod cache;

pub use cache::{Cache, CacheConfig, Expiring};