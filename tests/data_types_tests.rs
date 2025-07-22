use cache_rs::{Cache, Expiring};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: String,
    name: String,
    price: f64,
    in_stock: bool,
}

#[tokio::test]
async fn test_string_cache() {
    let cache = Cache::new(
        |key: String| {
            Box::pin(async move {
                let value = format!("processed_{}", key.to_uppercase());
                Ok(Expiring::with_duration(value, Duration::from_secs(1)))
            })
        },
        |key: &String| key.clone(),
    );

    let result = cache.get("hello".to_string()).await.unwrap();
    assert_eq!(result, "processed_HELLO");
}

#[tokio::test]
async fn test_struct_cache() {
    let cache = Cache::new(
        |user_id: u32| {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                let user = User {
                    id: user_id,
                    name: format!("User{}", user_id),
                    email: format!("user{}@example.com", user_id),
                };
                Ok(Expiring::with_duration(user, Duration::from_secs(1)))
            })
        },
        |key: &u32| key.to_string(),
    );

    let user = cache.get(123).await.unwrap();
    assert_eq!(user.id, 123);
    assert_eq!(user.name, "User123");
    assert_eq!(user.email, "user123@example.com");
}

#[tokio::test]
async fn test_vec_cache() {
    let cache = Cache::new(
        |count: usize| {
            Box::pin(async move {
                let numbers: Vec<i32> = (0..count as i32).collect();
                Ok(Expiring::with_duration(numbers, Duration::from_secs(1)))
            })
        },
        |key: &usize| key.to_string(),
    );

    let numbers = cache.get(5).await.unwrap();
    assert_eq!(numbers, vec![0, 1, 2, 3, 4]);
}

#[tokio::test]
async fn test_hashmap_cache() {
    let cache = Cache::new(
        |category: String| {
            Box::pin(async move {
                let mut products = HashMap::new();
                products.insert(
                    "prod1".to_string(),
                    Product {
                        id: "prod1".to_string(),
                        name: format!("{} Product 1", category),
                        price: 99.99,
                        in_stock: true,
                    },
                );
                products.insert(
                    "prod2".to_string(),
                    Product {
                        id: "prod2".to_string(),
                        name: format!("{} Product 2", category),
                        price: 149.99,
                        in_stock: false,
                    },
                );
                Ok(Expiring::with_duration(products, Duration::from_secs(1)))
            })
        },
        |key: &String| key.clone(),
    );

    let products = cache.get("Electronics".to_string()).await.unwrap();
    assert_eq!(products.len(), 2);
    assert!(products.contains_key("prod1"));
    assert_eq!(products["prod1"].name, "Electronics Product 1");
}

#[tokio::test]
async fn test_tuple_keys() {
    let cache = Cache::new(
        |key: (String, u32)| {
            Box::pin(async move {
                let (category, page) = key;
                let result = format!("{}:page{}", category, page);
                Ok(Expiring::with_duration(result, Duration::from_secs(1)))
            })
        },
        |key: &(String, u32)| format!("{}:{}", key.0, key.1),
    );

    let result = cache.get(("products".to_string(), 2)).await.unwrap();
    assert_eq!(result, "products:page2");
}

#[tokio::test]
async fn test_option_values() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                let value = if key % 2 == 0 {
                    Some(format!("even_{}", key))
                } else {
                    None
                };
                Ok(Expiring::with_duration(value, Duration::from_secs(1)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let even_result = cache.get(4).await.unwrap();
    assert_eq!(even_result, Some("even_4".to_string()));

    let odd_result = cache.get(3).await.unwrap();
    assert_eq!(odd_result, None);
}

#[tokio::test]
async fn test_result_values() {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                let value: Result<String, String> = if key > 0 {
                    Ok(format!("positive_{}", key))
                } else {
                    Err("negative_number".to_string())
                };
                Ok(Expiring::with_duration(value, Duration::from_secs(1)))
            })
        },
        |key: &i32| key.to_string(),
    );

    let positive_result = cache.get(5).await.unwrap();
    assert_eq!(positive_result, Ok("positive_5".to_string()));

    let negative_result = cache.get(-1).await.unwrap();
    assert_eq!(negative_result, Err("negative_number".to_string()));
}
