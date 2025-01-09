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
struct Config {
    encrypted_key: String,
    nonce: String,
}

pub struct ApiKeyManager {
    config_path: PathBuf,
    encryption_key: [u8; 32],
}

impl ApiKeyManager {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "todoclist", "Todoclist")
            .expect("Could not determine config directory");
        
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).expect("Failed to create config directory");
        
        Self {
            config_path: config_dir.join("config.json"),
            encryption_key: *b"0123456789abcdef0123456789abcdef", // TODO: Replace with proper key management
        }
    }

    pub fn save_api_key(&self, api_key: &str) -> Result<(), String> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;
        
        let nonce = Nonce::from_slice(b"unique nonce"); // TODO: Generate unique nonce per save
        let encrypted_data = cipher.encrypt(nonce, api_key.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let config = Config {
            encrypted_key: general_purpose::STANDARD.encode(encrypted_data),
            nonce: general_purpose::STANDARD.encode(nonce),
        };

        let json = serde_json::to_string(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn load_api_key(&self) -> Result<String, String> {
        let data = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        let config: Config = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let encrypted_data = general_purpose::STANDARD.decode(config.encrypted_key)
            .map_err(|e| format!("Failed to decode encrypted key: {}", e))?;

        let nonce = general_purpose::STANDARD.decode(config.nonce)
            .map_err(|e| format!("Failed to decode nonce: {}", e))?;

        let decrypted_data = cipher.decrypt(Nonce::from_slice(&nonce), &encrypted_data[..])
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(decrypted_data)
            .map_err(|e| format!("Failed to convert to string: {}", e))
    }
}
