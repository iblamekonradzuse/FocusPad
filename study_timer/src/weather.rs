use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherWidget {
    pub city: Option<String>,
    pub current_weather: String,
    #[serde(skip)] // Skip serialization for Instant
    pub last_update: Option<Instant>,
    #[serde(skip)] // Skip serialization for UI state
    pub show_city_input: bool,
    #[serde(skip)] // Skip serialization for UI state
    pub city_input_buffer: String,
    #[serde(skip)] // Skip serialization for Duration
    pub update_interval: Duration,
}

impl Default for WeatherWidget {
    fn default() -> Self {
        Self {
            city: None,
            current_weather: "☀️".to_string(),
            last_update: None,
            show_city_input: false,
            city_input_buffer: String::new(),
            update_interval: Duration::from_secs(600), // 10 minutes
        }
    }
}

impl WeatherWidget {
    pub fn should_update(&self) -> bool {
        match self.last_update {
            Some(last) => last.elapsed() >= self.update_interval,
            None => self.city.is_some(),
        }
    }

    pub fn fetch_weather(&mut self) {
        if let Some(city) = &self.city {
            match self.get_weather_data(city) {
                Ok(weather) => {
                    self.current_weather = weather;
                    self.last_update = Some(Instant::now());
                }
                Err(_) => {
                    // Keep the previous weather data on error
                    self.last_update = Some(Instant::now());
                }
            }
        }
    }

    fn get_weather_data(&self, city: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("wttr.in/{}?format=3", city);

        let output = Command::new("curl").arg("-s").arg(&url).output()?;

        if output.status.success() {
            let weather_text = String::from_utf8(output.stdout)?;
            let weather_text = weather_text.trim();

            // Remove city name part (everything up to and including the colon)
            let weather_part = if let Some(colon_pos) = weather_text.find(':') {
                &weather_text[colon_pos + 1..].trim()
            } else {
                weather_text
            };

            // Clean up the weather text - remove problematic Unicode and extra whitespace
            let cleaned = weather_part
                .chars()
                .filter(|c| {
                    // Keep basic ASCII, common weather emojis, and temperature symbols
                    c.is_ascii()
                        || matches!(
                            *c,
                            '☀' | '☁'
                                | '⛅'
                                | '⛈'
                                | '🌧'
                                | '🌦'
                                | '🌨'
                                | '❄'
                                | '🌩'
                                | '🌤'
                                | '°'
                                | '℃'
                                | '℉'
                        )
                })
                .collect::<String>()
                .trim()
                .to_string();

            // If we have a result, return it, otherwise fallback to a simple weather emoji
            if cleaned.is_empty() {
                Ok("☀️".to_string())
            } else {
                Ok(cleaned)
            }
        } else {
            Err("Failed to fetch weather data".into())
        }
    }

    pub fn set_city(&mut self, city: String) {
        if city.trim().is_empty() {
            self.city = None;
            self.current_weather = "☀️".to_string();
        } else {
            self.city = Some(city.trim().to_string());
            self.fetch_weather();
        }
        self.show_city_input = false;
        self.city_input_buffer.clear();
    }

    pub fn show_city_input(&mut self) {
        self.show_city_input = true;
        self.city_input_buffer = self.city.clone().unwrap_or_default();
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut city_changed = false;

        if self.show_city_input {
            ui.horizontal(|ui| {
                ui.label("City:");
                let response = ui.text_edit_singleline(&mut self.city_input_buffer);

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.set_city(self.city_input_buffer.clone());
                    city_changed = true;
                }

                if ui.button("✓").clicked() {
                    self.set_city(self.city_input_buffer.clone());
                    city_changed = true;
                }

                if ui.button("✗").clicked() {
                    self.show_city_input = false;
                    self.city_input_buffer.clear();
                }
            });
        } else {
            let weather_button = ui.button(&self.current_weather);
            if weather_button.clicked() {
                self.show_city_input();
            }

            if weather_button.hovered() {
                let tooltip_text = if let Some(city) = &self.city {
                    format!("Weather for {}\nClick to change city", city)
                } else {
                    "Click to set your city".to_string()
                };
                egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("weather_tooltip"), |ui| {
                    ui.label(tooltip_text);
                });
            }
        }

        city_changed
    }

    pub fn update(&mut self) {
        if self.should_update() {
            self.fetch_weather();
        }
    }

    // Save and load methods
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let settings_dir = self.get_settings_dir()?;
        fs::create_dir_all(&settings_dir)?;

        let settings_path = settings_dir.join("weather_settings.json");
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(settings_path, json)?;

        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;

        let settings_dir = Self::get_settings_dir_static()?;
        let settings_path = settings_dir.join("weather_settings.json");

        if !settings_path.exists() {
            return Ok(Self::default());
        }

        let json = fs::read_to_string(settings_path)?;
        let mut widget: WeatherWidget = serde_json::from_str(&json)?;

        // Initialize skipped fields with defaults
        widget.last_update = None;
        widget.show_city_input = false;
        widget.city_input_buffer = String::new();
        widget.update_interval = Duration::from_secs(600);

        // Fetch weather on load if city is set
        if widget.city.is_some() {
            widget.fetch_weather();
        }

        Ok(widget)
    }

    fn get_settings_dir(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        Self::get_settings_dir_static()
    }

    fn get_settings_dir_static() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let mut path = dirs::config_dir()
            .or_else(|| dirs::home_dir())
            .ok_or("Could not find config directory")?;

        path.push("study_timer");
        Ok(path)
    }
}

