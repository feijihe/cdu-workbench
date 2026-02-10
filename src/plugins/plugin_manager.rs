
use plugin::Plugin;
use std::collections::HashMap;
use serde_json::Value;

pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    config: Value,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut plugins = HashMap::new();
        Self {
            plugins: plugins,
        };
        Self
    }
    pub fn init_plugin() {

    }

}
