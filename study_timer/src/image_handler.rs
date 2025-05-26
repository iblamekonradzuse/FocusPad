use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardImage {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub data: String, // Base64 encoded image data
    pub size: usize,
}

impl CardImage {
    pub fn new(filename: String, data: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let mime_type = Self::get_mime_type(&filename)?;
        let base64_data = general_purpose::STANDARD.encode(&data);
        let id = Self::generate_id(&filename);

        Ok(CardImage {
            id,
            filename,
            mime_type,
            data: base64_data,
            size: data.len(),
        })
    }

    pub fn from_file(file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read(file_path)?;
        let filename = file_path
            .file_name()
            .ok_or("Invalid filename")?
            .to_string_lossy()
            .to_string();

        Self::new(filename, data)
    }

    pub fn get_data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.data)
    }

    pub fn save_to_disk(&self, images_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(images_dir)?;

        let file_path = images_dir.join(&self.filename);
        let data = general_purpose::STANDARD.decode(&self.data)?;

        let mut file = fs::File::create(&file_path)?;
        file.write_all(&data)?;

        Ok(file_path)
    }

    fn get_mime_type(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or("No file extension found")?
            .to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
            "png" => Ok("image/png".to_string()),
            "gif" => Ok("image/gif".to_string()),
            "webp" => Ok("image/webp".to_string()),
            "bmp" => Ok("image/bmp".to_string()),
            "svg" => Ok("image/svg+xml".to_string()),
            _ => Err(format!("Unsupported image format: {}", extension).into()),
        }
    }

    fn generate_id(filename: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}_{}", timestamp, filename.replace(' ', "_"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageManager {
    images_dir: PathBuf,
}

impl ImageManager {
    pub fn new() -> Self {
        let images_dir = PathBuf::from("flashcard_images");
        Self { images_dir }
    }

    pub fn add_image_from_file(
        &self,
        file_path: &Path,
    ) -> Result<CardImage, Box<dyn std::error::Error>> {
        let image = CardImage::from_file(file_path)?;
        // Optionally save to disk for backup
        image.save_to_disk(&self.images_dir)?;
        Ok(image)
    }

    pub fn add_image_from_data(
        &self,
        filename: String,
        data: Vec<u8>,
    ) -> Result<CardImage, Box<dyn std::error::Error>> {
        let image = CardImage::new(filename, data)?;
        // Optionally save to disk for backup
        image.save_to_disk(&self.images_dir)?;
        Ok(image)
    }

    pub fn delete_image(&self, image: &CardImage) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.images_dir.join(&image.filename);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn cleanup_unused_images(
        &self,
        used_image_ids: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.images_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.images_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();

            // Check if this image is still being used
            let is_used = used_image_ids
                .iter()
                .any(|id| id.ends_with(&filename) || filename.contains(id));

            if !is_used {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }
}

pub fn open_file_dialog() -> Option<PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter(
            "Images",
            &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"],
        )
        .set_title("Select Image for Flashcard")
        .pick_file()
}

// Helper function to validate image file
pub fn is_valid_image_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg"
        )
    } else {
        false
    }
}

// Helper function to format file size
pub fn format_file_size(size: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as usize, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

