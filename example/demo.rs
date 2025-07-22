use cache_rs::{Cache, Expiring};
use std::time::Duration;

const CACHE_DURATION_SECS: u64 = 5;
const LOAD_DELAY_MS: u64 = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache = Cache::new(
        |key: i32| {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(LOAD_DELAY_MS)).await;
                let value = format!("Value for key: {}", key);
                Ok(Expiring::with_duration(
                    value,
                    Duration::from_secs(CACHE_DURATION_SECS),
                ))
            })
        },
        |key: &i32| key.to_string(),
    );

    println!("Loading value for key 1...");
    let value1 = cache.get(1).await?;
    println!("Got: {}", value1);

    println!("Loading value for key 1 again (should be cached)...");
    let value2 = cache.get(1).await?;
    println!("Got: {}", value2);

    println!("Cache size: {}", cache.size());

    cache.delete(1);
    println!("Cache size after deletion: {}", cache.size());

    println!("Testing multiple keys:");
    let _val_a = cache.get(10).await?;
    let _val_b = cache.get(20).await?;
    let _val_c = cache.get(30).await?;

    println!("Cache size with multiple keys: {}", cache.size());

    cache.delete_all();
    println!("Cache size after clear: {}", cache.size());

    Ok(())
}
