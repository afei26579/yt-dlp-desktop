use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use crate::database::models::AppSettings;

pub struct ConfigManager {
    app_dir: PathBuf,
    settings: RwLock<AppSettings>,
}

impl ConfigManager {
    pub fn new(app_dir: PathBuf) -> Self {
        let config_path = app_dir.join("settings.json");
        let settings = if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppSettings::default()
        };

        Self {
            app_dir,
            settings: RwLock::new(settings),
        }
    }

    pub fn app_dir(&self) -> PathBuf {
        self.app_dir.clone()
    }

    pub fn load(&self) -> AppSettings {
        self.settings.read().unwrap().clone()
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = self.app_dir.join("settings.json");
        fs::create_dir_all(&self.app_dir)?;
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(config_path, json)?;
        *self.settings.write().unwrap() = settings.clone();
        Ok(())
    }
}