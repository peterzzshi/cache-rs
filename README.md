# Cache-RS

A generic, thread-safe cache implementation in Rust with automatic expiration support and async loading.

## Overview

Cache-RS provides a flexible caching solution that:
- Automatically loads values when not present or expired
- Supports any key and value types (with proper trait bounds)
- Handles expiration with customizable durations
- Provides thread-safe concurrent access
- Supports async loading functions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cache-rs = "0.1.0"
tokio = { version = "1.0", features = ["rt", "macros"] }
```

## Basic Usage

### Simple String Cache

```rust
use cache_rs::{Cache, Expiring};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache = Cache::new(
        // Loader function - called when key is not cached or expired
        |key: i32| {
            Box::pin(async move {
                let value = format!("loaded_{}", key);
                Ok(Expiring::with_duration(value, Duration::from_secs(60)))
            })
        },
        // Key mapper - converts key to string for internal storage
        |key: &i32| key.to_string(),
    );

    // First call will load the value
    let value = cache.get(42).await?;
    println!("Value: {}", value); // "loaded_42"

    // Second call will return cached value (no loading)
    let cached_value = cache.get(42).await?;
    println!("Cached: {}", cached_value); // "loaded_42"

    Ok(())
}
```

### Custom Struct Cache

```rust
use cache_rs::{Cache, Expiring};
use std::time::Duration;

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache = Cache::new(
        |user_id: u32| {
            Box::pin(async move {
                // Simulate database lookup
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                let user = User {
                    id: user_id,
                    name: format!("User{}", user_id),
                    email: format!("user{}@example.com", user_id),
                };
                
                Ok(Expiring::with_duration(user, Duration::from_secs(300)))
            })
        },
        |key: &u32| key.to_string(),
    );

    let user = cache.get(123).await?;
    println!("User: {} ({})", user.name, user.email);

    Ok(())
}
```

### Complex Key Types

```rust
use cache_rs::{Cache, Expiring};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache = Cache::new(
        |key: (String, u32)| {
            Box::pin(async move {
                let (category, page) = key;
                let result = format!("{}:page{}", category, page);
                Ok(Expiring::with_duration(result, Duration::from_secs(120)))
            })
        },
        |key: &(String, u32)| format!("{}:{}", key.0, key.1),
    );

    let result = cache.get(("products".to_string(), 2)).await?;
    println!("Result: {}", result); // "products:page2"

    Ok(())
}
```

## API Reference

### Cache

The main cache struct with the following methods:

- `new(load, get_key_for_map)` - Creates a new cache instance
- `get(key)` - Gets a value, loading if necessary
- `get_with_expiry(key)` - Gets a value with expiration info
- `delete(key)` - Removes a key from the cache
- `delete_all()` - Clears the entire cache
- `size()` - Returns the number of cached items

### Expiring

A wrapper for values with expiration:

- `new(value, expires_at)` - Creates with specific expiration time
- `with_duration(value, duration)` - Creates with duration from now
- `is_expired()` - Checks if the value has expired

## Cache Management

### Manual Cache Operations

```rust
// Delete specific key
cache.delete(key).await;

// Clear entire cache
cache.delete_all().await;

// Check cache size
println!("Cache contains {} items", cache.size());
```

### Handling Expiration

```rust
use cache_rs::{Cache, Expiring};
use std::time::Duration;

let cache = Cache::new(
    |key: String| {
        Box::pin(async move {
            let value = format!("data_{}", key);
            // Cache for 30 seconds
            Ok(Expiring::with_duration(value, Duration::from_secs(30)))
        })
    },
    |key: &String| key.clone(),
);

// Values automatically expire and reload when accessed after expiration
```

### Error Handling

```rust
use cache_rs::{Cache, Expiring};
use std::time::Duration;

let cache = Cache::new(
    |key: String| {
        Box::pin(async move {
            if key == "error" {
                Err("Load failed".into())
            } else {
                Ok(Expiring::with_duration(
                    format!("success_{}", key),
                    Duration::from_secs(60)
                ))
            }
        })
    },
    |key: &String| key.clone(),
);

// Errors are propagated to the caller
match cache.get("error".to_string()).await {
    Ok(value) => println!("Got: {}", value),
    Err(e) => println!("Error: {}", e),
}
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test basic_tests
cargo test --test data_types_tests
cargo test --test error_handling_tests

# Run with output
cargo test -- --nocapture
```

### Test Categories

1. **Basic Tests** (`tests/basic_tests.rs`)
   - Basic functionality and cache hits
   - Expiration behavior
   - Delete operations

2. **Data Types Tests** (`tests/data_types_tests.rs`)
   - String caching
   - Custom struct caching
   - Collections (Vec, HashMap)
   - Tuple keys
   - Option and Result values

3. **Error Handling Tests** (`tests/error_handling_tests.rs`)
   - Load error propagation
   - Concurrent access
   - Different error types

### Example Test

```rust
#[tokio::test]
async fn test_cache_expiration() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                Ok(Expiring::with_duration(
                    format!("value_{}", key),
                    Duration::from_millis(50) // Short expiry
                ))
            })
        },
        |key: &i32| key.to_string(),
    );

    let value1 = cache.get(1).await.unwrap();
    assert_eq!(value1, "value_1");

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should reload after expiration
    let value2 = cache.get(1).await.unwrap();
    assert_eq!(value2, "value_1");
}
```

## Performance Considerations

- Uses `RwLock` for thread-safe access with concurrent reads
- Keys are converted to strings for internal storage
- Expired items are not automatically cleaned up (lazy removal on access)
- Concurrent requests for the same key may result in multiple loads

## License

Licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.