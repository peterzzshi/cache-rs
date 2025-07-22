use cache_rs::{Cache, Expiring};
use std::time::Duration;

#[tokio::test]
async fn test_basic_functionality() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                let value = format!("loaded_{}", key);
                Ok(Expiring::with_duration(value, Duration::from_secs(1)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let result = cache.get(42).await.unwrap();
    assert_eq!(result, "loaded_42");
    assert_eq!(cache.size(), 1);
}

#[tokio::test]
async fn test_cache_hit() {
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let cache = Cache::new(
        move |key: i32| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let value = format!("loaded_{}", key);
                Ok(Expiring::with_duration(value, Duration::from_secs(10)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let result1 = cache.get(1).await.unwrap();
    assert_eq!(result1, "loaded_1");
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    let result2 = cache.get(1).await.unwrap();
    assert_eq!(result2, "loaded_1");
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_expiration() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                let value = format!("loaded_{}", key);
                Ok(Expiring::with_duration(value, Duration::from_millis(50)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let _result1 = cache.get(42).await.unwrap();
    assert_eq!(cache.size(), 1);

    tokio::time::sleep(Duration::from_millis(100)).await;

    let _result2 = cache.get(42).await.unwrap();
    assert_eq!(cache.size(), 1);
}

#[tokio::test]
async fn test_delete_operations() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                let value = format!("loaded_{}", key);
                Ok(Expiring::with_duration(value, Duration::from_secs(1)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let _val1 = cache.get(1).await.unwrap();
    let _val2 = cache.get(2).await.unwrap();
    assert_eq!(cache.size(), 2);

    cache.delete(1);
    assert_eq!(cache.size(), 1);

    cache.delete_all();
    assert_eq!(cache.size(), 0);
}
