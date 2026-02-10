use serde::Serialize;
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

// 错误类型
#[derive(Debug, Error)]
pub enum FileStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("not found file: {0}")]
    ConfigNotFound(String),

    #[error("unsupported file extension: {0}")]
    UnsupportedExtension(String),
}

pub type Result<T> = std::result::Result<T, FileStoreError>;

#[allow(dead_code)]
pub const EVENT_NAME: &str = "change";

pub trait ConfigChangeCallback: Send + Sync {
    fn on_config_change(&self, new_config: &Config, old_config: &Config);
}

pub type Config = serde_json::Value;

pub struct FileStore {
    path: PathBuf,
    default_path: Option<PathBuf>,
    pub config: Config,
    last_config: Config,
    callbacks: Arc<Mutex<HashMap<String, Vec<Arc<dyn ConfigChangeCallback>>>>>,
}

#[allow(dead_code)]
impl FileStore {
    pub fn new<P: AsRef<Path>>(path: P, default_path: Option<P>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let default_path = default_path.map(|p| p.as_ref().to_path_buf());

        // 确保配置目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = if path.exists() {
            Self::load_config_from_file(&path)?
        } else if let Some(ref dp) = default_path {
            Self::load_config_from_file(dp).unwrap_or_default()
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        Ok(Self {
            path,
            default_path,
            config: config.clone(),
            last_config: config,
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    fn load_config_from_file(path: &Path) -> Result<Config> {
        if !path.exists() {
            return Err(FileStoreError::ConfigNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                let config: Config = serde_json::from_str(&content)?;
                Ok(config)
            }
            Some("yaml") | Some("yml") => {
                let config: Config = serde_yaml::from_str(&content)?;
                Ok(config)
            }
            Some(ext) => Err(FileStoreError::UnsupportedExtension(ext.to_string())),
            None => Err(FileStoreError::UnsupportedExtension(
                "no extension".to_string(),
            )),
        }
    }

    pub fn get_config(&self) -> Config {
        let config = self.config.clone();
        config
    }

    pub fn update_config<F>(&self, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.clone();
        let old_config = config.clone();
        updater(&mut config);
        if &old_config == &config {
            return Ok(false);
        }
        self.trigger_callbacks(&config, &old_config);
        Ok(true)
    }
    pub fn set_config(&self, new_config: Config) -> Result<bool> {
        let mut config = self.config.clone();
        let old_config = config.clone();

        config = new_config;

        self.save_config_internal(&config)?;

        self.trigger_callbacks(&config, &old_config);
        Ok(true)
    }

    pub fn save_config(&self) -> Result<bool> {
        let config = self.config.clone();
        let old_config = self.last_config.clone();

        self.save_config_internal(&config)?;
        self.trigger_callbacks(&config, &old_config);
        Ok(true)
    }
    fn save_config_internal(&self, config: &Config) -> Result<bool> {
        let content = match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::to_string_pretty(config)?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(config)?,
            Some(ext) => return Err(FileStoreError::UnsupportedExtension(ext.to_string())),
            None => {
                return Err(FileStoreError::UnsupportedExtension(
                    "no extension".to_string(),
                ))
            }
        };
        fs::write(&self.path, content)?;
        // let mut last_config = self.last_config.lock().unwrap();
        // *last_config = config.clone();

        Ok(true)
    }

    /// 重置为默认配置
    pub fn reset_config(&self) -> Result<bool> {
        let default_config = if let Some(ref default_path) = self.default_path {
            Self::load_config_from_file(default_path).unwrap_or_default()
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        self.set_config(default_config)
    }

    /// 获取配置值
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let config = self.config.clone();
        config
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// 设置配置值
    pub fn set<T: Serialize>(&self, key: &str, value: T) -> Result<bool> {
        let value = serde_json::to_value(value)?;
        self.update_config(|config| {
            config.as_object_mut().unwrap().insert(key.to_string(), value);
        })
    }

    /// 删除配置值
    pub fn remove(&self, key: &str) -> Result<bool> {
        self.update_config(|config| {
            config.as_object_mut().unwrap().remove(key);
        })
    }

    /// 注册配置变更回调（使用 async-trait）
    pub fn register_callback<C>(&self, event: String, callback: C)
    where
        C: ConfigChangeCallback + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks
            .entry(event)
            .or_insert_with(Vec::new)
            .push(Arc::new(callback));
    }

    /// 注册 change 事件回调的便捷方法
    pub fn on_change<C>(&self, callback: C)
    where
        C: ConfigChangeCallback + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks
            .entry(EVENT_NAME.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(callback));
    }

    /// 触发回调（异步）
    fn trigger_callbacks(&self, new_config: &Config, old_config: &Config) {
        let callbacks = self.callbacks.lock().unwrap();

        if let Some(callbacks_list) = callbacks.get(EVENT_NAME) {
            // 同步执行回调
            for callback in callbacks_list {
                callback.on_config_change(new_config, old_config);
            }
        }
    }
}

impl std::fmt::Debug for FileStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStore")
            .field("path", &self.path)
            .field("default_path", &self.default_path)
            .field("config", &self.config)
            .field("last_config", &self.last_config)
            .field("callbacks_count", &self.callbacks.lock().unwrap().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::tempdir;

    #[test]
    fn test_filestore_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        assert!(!config_path.exists());
        assert!(filestore.get_config().as_object_mut().unwrap().is_empty());
    }

    #[test]
    fn test_filestore_with_default() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let default_path = temp_dir.path().join("default.json");

        let default_config = serde_json::json!({
            "key1": "value1",
            "key2": 42
        });

        fs::write(
            &default_path,
            serde_json::to_string_pretty(&default_config).unwrap(),
        )
        .unwrap();

        let filestore = FileStore::new(&config_path, Some(&default_path)).unwrap();

        assert!(!config_path.exists());
        let config = filestore.get_config();
        assert_eq!(config.as_object().unwrap().len(), 2);
        assert_eq!(
            config.get("key1").unwrap(),
            &serde_json::Value::String("value1".to_string())
        );
        assert_eq!(
            config.get("key2").unwrap(),
            &serde_json::Value::Number(42.into())
        );
    }

    #[tokio::test]
    async fn test_set_and_get_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        let mut new_config = serde_json::Value::Object(serde_json::Map::new());
        new_config.as_object_mut().unwrap().insert(
            "test_key".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );

        filestore.set_config(new_config.clone()).unwrap();

        let retrieved_config = filestore.get_config();
        assert_eq!(retrieved_config.as_object().unwrap(), new_config.as_object_mut().unwrap());
    }

    #[tokio::test]
    async fn test_save_and_load_json() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        filestore.set("name", "test_user").unwrap();
        filestore.set("age", 25).unwrap();

        filestore.save_config().unwrap();

        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = serde_json::from_str(&content).unwrap();

        assert_eq!(
            loaded_config.get("name").unwrap(),
            &serde_json::Value::String("test_user".to_string())
        );
        assert_eq!(
            loaded_config.get("age").unwrap(),
            &serde_json::Value::Number(25.into())
        );
    }

    #[tokio::test]
    async fn test_save_and_load_yaml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let filestore = FileStore::new(&config_path, None).unwrap();

        filestore.set("name", "test_user").unwrap();
        filestore.set("age", 25).unwrap();

        filestore.save_config().unwrap();

        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = serde_yaml::from_str(&content).unwrap();

        assert_eq!(
            loaded_config.get("name").unwrap(),
            &serde_json::Value::String("test_user".to_string())
        );
        assert_eq!(
            loaded_config.get("age").unwrap(),
            &serde_json::Value::Number(25.into())
        );
    }

    #[test]
    fn test_get_typed_value() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        filestore.set("string_value", "hello").unwrap();
        filestore.set("number_value", 42).unwrap();
        filestore.set("bool_value", true).unwrap();

        let string_val: String = filestore.get("string_value").unwrap();
        let number_val: i32 = filestore.get("number_value").unwrap();
        let bool_val: bool = filestore.get("bool_value").unwrap();

        assert_eq!(string_val, "hello");
        assert_eq!(number_val, 42);
        assert_eq!(bool_val, true);
    }

    #[tokio::test]
    async fn test_update_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        filestore.set("initial", "value").unwrap();

        let updated = filestore
            .update_config(|config| {
                config.as_object_mut().unwrap().insert(
                    "updated".to_string(),
                    serde_json::Value::String("new_value".to_string()),
                );
            })
            .unwrap();

        assert!(updated);

        let config = filestore.get_config();
        assert_eq!(config.as_object().unwrap().len(), 2);
        assert_eq!(
            config.get("initial").unwrap(),
            &serde_json::Value::String("value".to_string())
        );
        assert_eq!(
            config.get("updated").unwrap(),
            &serde_json::Value::String("new_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_remove_key() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        filestore.set("key1", "value1").unwrap();
        filestore.set("key2", "value2").unwrap();

        let config_before = filestore.get_config();
        assert_eq!(config_before.as_object().unwrap().len(), 2);

        filestore.remove("key1").unwrap();

        let config_after = filestore.get_config();
        assert_eq!(config_after.as_object().unwrap().len(), 1);
        assert!(config_after.get("key1").is_none());
        assert!(config_after.get("key2").is_some());
    }

    #[tokio::test]
    async fn test_reset_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let default_path = temp_dir.path().join("default.json");

        let default_config = serde_json::json!({
            "default_key": "default_value"
        });

        fs::write(
            &default_path,
            serde_json::to_string_pretty(&default_config).unwrap(),
        )
        .unwrap();

        let filestore = FileStore::new(&config_path, Some(&default_path)).unwrap();

        filestore.set("custom_key", "custom_value").unwrap();

        let config_before = filestore.get_config();
        assert_eq!(config_before.as_object().unwrap().len(), 2);

        filestore.reset_config().unwrap();

        let config_after = filestore.get_config();
        assert_eq!(config_after.as_object().unwrap().len(), 1);
        assert_eq!(
            config_after.get("default_key").unwrap(),
            &serde_json::Value::String("default_value".to_string())
        );
        assert!(config_after.get("custom_key").is_none());
    }

    #[tokio::test]
    async fn test_config_change_callback() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let filestore = FileStore::new(&config_path, None).unwrap();

        let callback_called = Arc::new(AtomicBool::new(false));
        let callback_called_clone = callback_called.clone();

        struct TestCallback {
            called: Arc<AtomicBool>,
        }

        impl ConfigChangeCallback for TestCallback {
            fn on_config_change(&self, new_config: &Config, old_config: &Config) {
                self.called.store(true, Ordering::SeqCst);
                assert_eq!(old_config.as_object().unwrap().len(), 0);
                assert_eq!(new_config.as_object().unwrap().len(), 1);
                assert_eq!(
                    new_config.get("test").unwrap(),
                    &serde_json::Value::String("value".to_string())
                );
            }
        }

        let callback = TestCallback {
            called: callback_called_clone,
        };

        filestore.on_change(callback);

        filestore.set("test", "value").unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(callback_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_unsupported_extension() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.txt");

        let result = FileStore::new(&config_path, None);

        assert!(result.is_ok());

        let filestore = result.unwrap();
        filestore.set("test", "value").unwrap();

        let save_result = filestore.save_config();
        assert!(save_result.is_err());

        if let Err(FileStoreError::UnsupportedExtension(ext)) = save_result {
            assert_eq!(ext, "txt");
        } else {
            panic!("Expected UnsupportedExtension error");
        }
    }

    #[test]
    fn test_file_not_found_error() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");

        let result = FileStore::load_config_from_file(&config_path);

        assert!(result.is_err());

        if let Err(FileStoreError::ConfigNotFound(path)) = result {
            assert!(path.contains("nonexistent.json"));
        } else {
            panic!("Expected ConfigNotFound error");
        }
    }
}
