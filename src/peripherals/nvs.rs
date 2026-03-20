//! NVS (Non-Volatile Storage) emulation
//!
//! Simple key-value storage backed by a JSON file.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use parking_lot::RwLock;

/// NVS storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NvsValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    String(String),
    Blob(Vec<u8>),
}

/// NVS storage manager
pub struct NvsStorage {
    path: PathBuf,
    data: RwLock<HashMap<String, NvsValue>>,
    auto_save: bool,
}

impl NvsStorage {
    /// Create or open NVS storage
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let data = if path.exists() {
            let json = fs::read_to_string(&path)
                .context("Failed to read NVS file")?;
            serde_json::from_str(&json)
                .context("Failed to parse NVS JSON")?
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            data: RwLock::new(data),
            auto_save: true,
        })
    }

    /// Set a value in NVS
    pub fn set(&self, key: impl Into<String>, value: NvsValue) -> Result<()> {
        let key = key.into();
        self.data.write().insert(key, value);

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Get a value from NVS
    pub fn get(&self, key: &str) -> Option<NvsValue> {
        self.data.read().get(key).cloned()
    }

    /// Get a u32 value
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        match self.get(key)? {
            NvsValue::U32(v) => Some(v),
            _ => None,
        }
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key)? {
            NvsValue::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get a blob value
    pub fn get_blob(&self, key: &str) -> Option<Vec<u8>> {
        match self.get(key)? {
            NvsValue::Blob(v) => Some(v),
            _ => None,
        }
    }

    /// Remove a key from NVS
    pub fn erase(&self, key: &str) -> Result<bool> {
        let removed = self.data.write().remove(key).is_some();

        if removed && self.auto_save {
            self.save()?;
        }

        Ok(removed)
    }

    /// Check if a key exists
    pub fn exists(&self, key: &str) -> bool {
        self.data.read().contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.data.read().keys().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&self) -> Result<()> {
        self.data.write().clear();

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&*self.data.read())?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    /// Enable/disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_nvs_basic() {
        let path = "test_nvs.json";
        fs::remove_file(path).ok();

        let nvs = NvsStorage::new(path).unwrap();

        nvs.set("test_u32", NvsValue::U32(42)).unwrap();
        nvs.set("test_str", NvsValue::String("hello".to_string())).unwrap();

        assert_eq!(nvs.get_u32("test_u32"), Some(42));
        assert_eq!(nvs.get_string("test_str"), Some("hello".to_string()));

        drop(nvs);

        // Reload and verify persistence
        let nvs2 = NvsStorage::new(path).unwrap();
        assert_eq!(nvs2.get_u32("test_u32"), Some(42));
        assert_eq!(nvs2.get_string("test_str"), Some("hello".to_string()));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_nvs_erase() {
        let path = "test_nvs_erase.json";
        fs::remove_file(path).ok();

        let nvs = NvsStorage::new(path).unwrap();

        nvs.set("key1", NvsValue::U32(1)).unwrap();
        nvs.set("key2", NvsValue::U32(2)).unwrap();

        assert_eq!(nvs.len(), 2);
        assert!(nvs.erase("key1").unwrap());
        assert_eq!(nvs.len(), 1);
        assert!(!nvs.exists("key1"));
        assert!(nvs.exists("key2"));

        fs::remove_file(path).ok();
    }
}
