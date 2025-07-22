use cache_rs::{Cache, Expiring};
use std::time::Duration;

#[derive(Debug)]
struct CustomError {
    message: String,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomError: {}", self.message)
    }
}

impl std::error::Error for CustomError {}

#[tokio::test]
async fn test_load_error_handling() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                if key == 404 {
                    Err(Box::new(CustomError {
                        message: "Not found".to_string(),
                    })
                        as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Ok(Expiring::with_duration(
                        format!("loaded_{}", key),
                        Duration::from_secs(1),
                    ))
                }
            })
        },
        |key: &i32| key.to_string(),
    );

    let result = cache.get(200).await.unwrap();
    assert_eq!(result, "loaded_200");

    let error_result = cache.get(404).await;
    assert!(error_result.is_err());
    assert!(error_result.unwrap_err().to_string().contains("Not found"));
}

#[tokio::test]
async fn test_concurrent_access() {
    let cache = std::sync::Arc::new(Cache::new(
        |key: i32| {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(Expiring::with_duration(
                    format!("loaded_{}", key),
                    Duration::from_secs(1),
                ))
            })
        },
        |key: &i32| key.to_string(),
    ));

    let mut handles = Vec::new();
    for i in 0..5 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move { cache_clone.get(i).await.unwrap() });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    assert_eq!(results.len(), 5);
    assert_eq!(cache.size(), 5);

    for result in &results {
        assert!(result.starts_with("loaded_"));
    }
}

#[tokio::test]
async fn test_concurrent_same_key() {
    let load_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = load_counter.clone();

    let cache = std::sync::Arc::new(Cache::new(
        move |key: i32| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                tokio::time::sleep(Duration::from_millis(50)).await;

                Ok(Expiring::with_duration(
                    format!("loaded_{}", key),
                    Duration::from_secs(10),
                ))
            })
        },
        |key: &i32| key.to_string(),
    ));

    let mut handles = Vec::new();
    for _ in 0..3 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move { cache_clone.get(42).await.unwrap() });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    assert_eq!(results.len(), 3);
    for result in &results {
        assert_eq!(*result, "loaded_42");
    }

    let load_count = load_counter.load(std::sync::atomic::Ordering::SeqCst);
    assert!(load_count >= 1, "Loader should be called at least once");

    assert_eq!(cache.size(), 1);
}

#[tokio::test]
async fn test_cache_with_different_error_types() {
    let cache = Cache::new(
        |key: String| {
            Box::pin(async move {
                match key.as_str() {
                    "io_error" => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
                        as Box<dyn std::error::Error + Send + Sync>),
                    "parse_error" => Err(Box::new("123abc".parse::<i32>().unwrap_err())
                        as Box<dyn std::error::Error + Send + Sync>),
                    "custom_error" => Err(Box::new(CustomError {
                        message: "Something went wrong".to_string(),
                    })
                        as Box<dyn std::error::Error + Send + Sync>),
                    _ => Ok(Expiring::with_duration(
                        format!("success_{}", key),
                        Duration::from_secs(1),
                    )),
                }
            })
        },
        |key: &String| key.clone(),
    );

    let io_result = cache.get("io_error".to_string()).await;
    assert!(io_result.is_err());
    assert!(
        io_result
            .unwrap_err()
            .to_string()
            .contains("File not found")
    );

    let parse_result = cache.get("parse_error".to_string()).await;
    assert!(parse_result.is_err());

    let custom_result = cache.get("custom_error".to_string()).await;
    assert!(custom_result.is_err());
    assert!(
        custom_result
            .unwrap_err()
            .to_string()
            .contains("Something went wrong")
    );

    let success_result = cache.get("valid_key".to_string()).await;
    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), "success_valid_key");
}

#[tokio::test]
async fn test_expiry_with_errors() {
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let cache = Cache::new(
        move |key: i32| {
            let counter = count_clone.clone();
            Box::pin(async move {
                let count = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if count == 0 {
                    Ok(Expiring::with_duration(
                        format!("success_{}", key),
                        Duration::from_millis(50),
                    ))
                } else {
                    Err(Box::new(CustomError {
                        message: "Subsequent call failed".to_string(),
                    })
                        as Box<dyn std::error::Error + Send + Sync>)
                }
            })
        },
        |key: &i32| key.to_string(),
    );

    let first_result = cache.get(1).await.unwrap();
    assert_eq!(first_result, "success_1");
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

    tokio::time::sleep(Duration::from_millis(100)).await;

    let second_result = cache.get(1).await;
    assert!(second_result.is_err());
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
}
