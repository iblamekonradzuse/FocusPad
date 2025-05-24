use crate::weather::WeatherWidget;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeatherSettings {
    city: Option<String>,
}

impl WeatherWidget {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = WeatherSettings {
            city: self.city.clone(),
        };

        let settings_dir = get_settings_dir()?;
        fs::create_dir_all(&settings_dir)?;

        let settings_path = settings_dir.join("weather_settings.json");
        let json = serde_json::to_string_pretty(&settings)?;
        fs::write(settings_path, json)?;

        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let settings_dir = get_settings_dir()?;
        let settings_path = settings_dir.join("weather_settings.json");

        if !settings_path.exists() {
            return Ok(Self::default());
        }

        let json = fs::read_to_string(settings_path)?;
        let settings: WeatherSettings = serde_json::from_str(&json)?;

        let mut widget = Self::default();
        widget.city = settings.city;

        // Fetch weather on load if city is set
        if widget.city.is_some() {
            widget.fetch_weather();
        }

        Ok(widget)
    }
}

fn get_settings_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = dirs::config_dir()
        .or_else(|| dirs::home_dir())
        .ok_or("Could not find config directory")?;

    path.push("study_timer");
    Ok(path)
}
