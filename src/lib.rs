use reqwest;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use std::collections::HashMap;
use tokio::sync::broadcast;
use url::Url;

const API_BASE_URL: &str = "https://kvs.wireway.ch/v2";

pub struct WireKVSDatabase {
    id: String,
    access_key: String,
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    is_connected: bool,
    tx: broadcast::Sender<Value>,
}

impl WireKVSDatabase {
    /// Creates a new WireKVSDatabase instance
    /// 
    /// # Example
    /// ```
    /// let db = WireKVSDatabase::new("database-id".to_string(), "access-key".to_string()).await;
    /// ```
    pub async fn new(id: String, access_key: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        let mut db = WireKVSDatabase {
            id,
            access_key,
            ws: None,
            is_connected: false,
            tx,
        };
        db.setup_websocket().await;
        db
    }

    async fn setup_websocket(&mut self) {
        let ws_url = format!(
            "wss://kvs.wireway.ch/events/{}?accessKey={}",
            self.id,
            urlencoding::encode(&self.access_key)
        );
        
        let url = Url::parse(&ws_url).unwrap();
        let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect");
        self.ws = Some(ws_stream);
        self.is_connected = true;
    }

    /// Gets all entries from the database
    /// 
    /// # Example
    /// ```
    /// let entries = db.get_all_entries().await.unwrap();
    /// println!("Entries: {:?}", entries);
    /// ```
    pub async fn get_all_entries(&self) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/database/{}", API_BASE_URL, self.id))
            .header("Authorization", &self.access_key)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Gets a specific value by key
    /// 
    /// # Example
    /// ```
    /// let value = db.get("my-key").await.unwrap();
    /// println!("Value: {:?}", value);
    /// ```
    pub async fn get(&self, key: &str) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/database/{}/{}", API_BASE_URL, self.id, key))
            .header("Authorization", &self.access_key)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Sets a value for a specific key
    /// 
    /// # Example
    /// ```
    /// db.set("greeting", json!("Hello!")).await.unwrap();
    /// ```
    pub async fn set(&self, key: &str, value: Value) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        client
            .post(&format!("{}/database/{}/{}", API_BASE_URL, self.id, key))
            .header("Authorization", &self.access_key)
            .json(&value)
            .send()
            .await?;
        Ok(())
    }

    /// Deletes a value by key
    /// 
    /// # Example
    /// ```
    /// db.delete("my-key").await.unwrap();
    /// ```
    pub async fn delete(&self, key: &str) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        client
            .delete(&format!("{}/database/{}/{}", API_BASE_URL, self.id, key))
            .header("Authorization", &self.access_key)
            .send()
            .await?;
        Ok(())
    }

    /// Subscribe to real-time database events
    /// 
    /// # Example
    /// ```
    /// let mut rx = db.subscribe();
    /// tokio::spawn(async move {
    ///     while let Ok(event) = rx.recv().await {
    ///         println!("Event: {:?}", event);
    ///     }
    /// });
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.tx.subscribe()
    }
}

pub struct WireKVS {
    token: String,
}

impl WireKVS {
    /// Creates a new WireKVS client instance
    /// 
    /// # Example
    /// ```
    /// let client = WireKVS::new("auth-token".to_string());
    /// ```
    pub fn new(token: String) -> Self {
        WireKVS { token }
    }

    /// Lists all databases for the authenticated user
    /// 
    /// # Example
    /// ```
    /// let databases = client.list_databases().await.unwrap();
    /// println!("Databases: {:?}", databases);
    /// ```
    pub async fn list_databases(&self) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/databases", API_BASE_URL))
            .header("Authorization", &self.token)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Creates a new database with specified configuration
    /// 
    /// # Example
    /// ```
    /// let mut config = HashMap::new();
    /// config.insert("allowPublicReads".to_string(), true);
    /// let db = client.create_database("My Database", config).await.unwrap();
    /// ```
    pub async fn create_database(&self, name: &str, config: HashMap<String, bool>) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/database", API_BASE_URL))
            .header("Authorization", &self.token)
            .json(&json!({
                "name": name,
                "allowPublicWrites": config.get("allowPublicWrites").unwrap_or(&false),
                "allowPublicReads": config.get("allowPublicReads").unwrap_or(&false),
                "allowPublicModifications": config.get("allowPublicModifications").unwrap_or(&false),
                "allowSpecificPublicReads": config.get("allowSpecificPublicReads").unwrap_or(&false),
            }))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    /// Deletes a database by ID
    /// 
    /// # Example
    /// ```
    /// client.delete_database("database-id").await.unwrap();
    /// ```
    pub async fn delete_database(&self, id: &str) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        client
            .delete(&format!("{}/database/{}", API_BASE_URL, id))
            .header("Authorization", &self.token)
            .send()
            .await?;
        Ok(())
    }

    /// Gets a database instance for direct operations
    /// 
    /// # Example
    /// ```
    /// let db = client.database("database-id".to_string(), "access-key".to_string()).await;
    /// ```
    pub async fn database(&self, id: String, access_key: String) -> WireKVSDatabase {
        WireKVSDatabase::new(id, access_key).await
    }
} 