use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio;
use wirekvs::{WireKVS, WireKVSDatabase};

const AUTH_TOKEN: &str = "your-token-from-cookies-here";

/// Demonstrates direct database connection and operations
/// 
/// Shows how to:
/// - Connect directly to a database
/// - Subscribe to events
/// - Set and get values
async fn demo_direct_connect() {
    println!("🚀 Starting WireKVS Direct Connect Demo");

    let db = WireKVSDatabase::new("your-database-id".to_string(), "your-access-key".to_string()).await;
    println!("✅ Connected directly to database");

    let mut rx = db.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("📝 Received event: {:?}", event);
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    db.set("greeting", json!("Hello from direct connect!")).await.unwrap();
    let value = db.get("greeting").await.unwrap();
    println!("Value: {:?}", value);

    println!("✅ Direct connect demo completed successfully!");
}

/// Demonstrates full client workflow
/// 
/// Shows how to:
/// - Initialize client
/// - Create database
/// - Perform CRUD operations
/// - Handle real-time events
/// - Clean up resources
async fn demo_with_client() {
    println!("🚀 Starting WireKVS Client Demo");

    let client = WireKVS::new(AUTH_TOKEN.to_string());
    println!("✅ Client initialized");

    println!("\n🆕 Creating new database...");
    let mut config = HashMap::new();
    config.insert("allowPublicReads".to_string(), true);
    config.insert("allowPublicWrites".to_string(), false);
    config.insert("allowPublicModifications".to_string(), false);
    config.insert("allowSpecificPublicReads".to_string(), true);

    let new_db = client.create_database("Demo Database", config).await.unwrap();
    println!("Created database: {:?}", new_db);

    let db = client.database(new_db["kvsId"].as_str().unwrap().to_string(), 
                           new_db["accessKey"].as_str().unwrap().to_string()).await;
    println!("\n🔌 Connected to database");

    let mut rx = db.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("📝 Received event: {:?}", event);
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\n💾 Setting values...");
    db.set("greeting", json!("Hello, World!")).await.unwrap();
    db.set("number", json!("42")).await.unwrap();
    db.set("json", json!({"foo": "bar"})).await.unwrap();

    println!("\n📖 Reading all entries...");
    let entries = db.get_all_entries().await.unwrap();
    println!("All entries: {:?}", entries);

    println!("\n🔍 Reading specific value...");
    let greeting = db.get("greeting").await.unwrap();
    println!("Greeting: {:?}", greeting);

    println!("\n🗑️ Deleting value...");
    db.delete("number").await.unwrap();

    println!("\n🧹 Cleaning up...");
    client.delete_database(&new_db["kvsId"].as_str().unwrap()).await.unwrap();

    println!("\n✅ Client demo completed successfully!");
}

/// Main entry point demonstrating both connection methods
#[tokio::main]
async fn main() {
    println!("Starting demos in 1 second...");
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    demo_direct_connect().await;
    println!("\n-------------------\n");
    demo_with_client().await;
}
