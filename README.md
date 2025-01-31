# WireKVS

A Rust client for the WireKVS database service. This client provides a simple interface to interact with WireKVS databases, including real-time updates via WebSocket connections.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wirekvs-rs = "0.1.0"
```

## Usage

There are two ways to use the WireKVS client:

### 1. Direct Database Connection

```rust
use wirekvs_rs::WireKVSDatabase;

#[tokio::main]
async fn main() {
    // Connect directly to a database
    let db = WireKVSDatabase::new("your-database-id".to_string(), "your-access-key".to_string()).await;

    // Use the database
    db.set("key", json!("value")).await.unwrap();
    let value = db.get("key").await.unwrap();
}
```

### 2. Using the Client for Database Management

```rust
use wirekvs_rs::WireKVS;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Initialize with your auth token
    let client = WireKVS::new("your-auth-token".to_string());

    // Create a new database
    let mut config = HashMap::new();
    config.insert("allowPublicWrites".to_string(), false);
    config.insert("allowPublicReads".to_string(), true);
    config.insert("allowPublicModifications".to_string(), false);
    config.insert("allowSpecificPublicReads".to_string(), false);

    let new_db = client.create_database("My Database", config).await.unwrap();

    // Connect to the new database
    let db = client.database(new_db["kvsId"].to_string(), new_db["accessKey"].to_string()).await;

    // Use the database
    db.set("key", json!("value")).await.unwrap();
}
```

## Database Operations

### Basic Operations

```rust
// Get all entries
let entries = db.get_all_entries().await.unwrap();

// Get a specific value
let value = db.get("key").await.unwrap();

// Set a value
db.set("key", json!("value")).await.unwrap();

// Delete a value
db.delete("key").await.unwrap();
```

### Database Management

```rust
// List all databases
let databases = client.list_databases().await.unwrap();

// Delete a database
client.delete_database("database-id").await.unwrap();
```

## Real-time Updates

The database instance includes a WebSocket connection for real-time updates. Events are broadcast through a channel that you can subscribe to.

## Error Handling

All async methods return `Result` types that should be handled appropriately:

```rust
match db.set("key", json!("value")).await {
    Ok(_) => println!("Value set successfully"),
    Err(e) => eprintln!("Error setting value: {}", e),
}
```

## Automatic Reconnection

The WebSocket connection is automatically established when creating a database instance.

## License

MIT
