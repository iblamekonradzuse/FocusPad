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
