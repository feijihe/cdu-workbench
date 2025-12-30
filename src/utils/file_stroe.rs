use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type ConfigCallback = Box<dyn Fn(&serde_json::Value, &serde_json::Value) + Send + Sync>;

#[derive(Clone)]
struct FileStore {
    path: PathBuf,
    default_path: Option<PathBuf>,
    config: Arc<RwLock<serde_json::Value>>,
    last_config: Arc<RwLock<serde_json::Value>>,
    config_change_callback: Arc<RwLock<HashMap<String, ConfigCallback>>>,
}

impl FileStore {
    fn new(path: impl AsRef<Path>, default_path: Option<impl AsRef<Path>>) -> Self {
        let path = path.as_ref().to_path_buf();
        let default_path = default_path.map(|p| p.as_ref().to_path_buf());

        let config: () = if path.exists() {
            Self::read_file(&config_path)?
        } else {
        };
        Self {
            path,
            default_path,
            config: Arc::new(RwLock::new(config)),
            last_config: Arc::new(RwLock::new(serde_json::Value::Null)),
            config_change_callback: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn save_config(&self) -> &str {
        self.config.save(self.path).unwrap();
        self.path
    }
    fn load_config(&self) -> &mut Config {
        self.config = load_config(self.path);
        self.config
    }
    fn read_file(path: &PathBuf) -> Result<()> {
        let config = serde_json::from_str::<Config>(&std::fs::read_to_string(path).unwrap());
        config.unwrap()
    }
}

/*
 * 加载配置文件
 * 根据文件后缀，判断是json还是yaml文件
 */
fn load_config(path: &str) -> &mut serde_json::Value {
    if path.starts_with(".json") {
        let config =
            serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(path).unwrap());
        Ok(config)
    } else if path.ends_with(".yaml") {
        let config =
            serde_yaml::from_str::<serde_json::Value>(&std::fs::read_to_string(path).unwrap());
        Ok(config)
    } else {
        panic!("不支持的配置文件格式");
    }
    config
}
