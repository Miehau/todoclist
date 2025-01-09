use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use base64::{engine::general_purpose, Engine as _};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct EncryptedKey {
    value: String,
    nonce: String,
    service: String,
    created_at: String, // ISO 8601 timestamp
    last_used: Option<String>, // ISO 8601 timestamp
}

#[derive(Serialize, Deserialize)]
struct Config {
    keys: Vec<EncryptedKey>,
    version: u8, // For future schema changes
    refresh_interval: Option<u64>, // Refresh interval in seconds
}

#[derive(Debug)]
pub struct ApiKeyManager {
    config_path: PathBuf,
    encryption_key: [u8; 32],
}

impl ApiKeyManager {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("dev", "mmlak", "todoclist")
            .expect("Could not determine config directory");
        
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).expect("Failed to create config directory");
        
        Self {
            config_path: config_dir.join("config.json"),
            encryption_key: *b"0123456789abcdef0123456789abcdef", // TODO: Replace with proper key management
        }
    }

    pub fn save_refresh_interval(&self, interval: u64) -> Result<(), String> {
        let mut config = self.load_config().unwrap_or_else(|_| Config {
            keys: Vec::new(),
            version: 1,
            refresh_interval: None,
        });

        config.refresh_interval = Some(interval);
        self.save_config(&config)
    }

    fn save_config(&self, config: &Config) -> Result<(), String> {
        let json = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn save_api_key(&self, service: &str, api_key: &str) -> Result<(), String> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;
        
        let nonce = Nonce::from_slice(b"unique nonce"); // TODO: Generate unique nonce per save
        let encrypted_data = cipher.encrypt(nonce, api_key.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let new_key = EncryptedKey {
            value: general_purpose::STANDARD.encode(encrypted_data),
            nonce: general_purpose::STANDARD.encode(nonce),
            service: service.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };

        // Try to load existing config
        let mut config = match fs::read_to_string(&self.config_path) {
            Ok(data) => serde_json::from_str::<Config>(&data)
                .map_err(|e| format!("Failed to parse config: {}", e))?,
            Err(_) => Config {
                keys: Vec::new(),
                version: 1,
                refresh_interval: Some(10),
            },
        };

        // Remove existing key for this service if it exists
        config.keys.retain(|k| k.service != service);
        config.keys.push(new_key);

        let json = serde_json::to_string(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn load_config(&self) -> Result<Config, String> {
        let data = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn load_api_key(&self, service: &str) -> Result<String, String> {
        let data = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        let config: Config = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let key = config.keys.iter()
            .find(|k| k.service == service)
            .ok_or_else(|| format!("No API key found for service: {}", service))?;

        let encrypted_data = general_purpose::STANDARD.decode(&key.value)
            .map_err(|e| format!("Failed to decode encrypted key: {}", e))?;

        let nonce = general_purpose::STANDARD.decode(&key.nonce)
            .map_err(|e| format!("Failed to decode nonce: {}", e))?;

        let decrypted_data = cipher.decrypt(Nonce::from_slice(&nonce), &encrypted_data[..])
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(decrypted_data)
            .map_err(|e| format!("Failed to convert to string: {}", e))
    }
}
