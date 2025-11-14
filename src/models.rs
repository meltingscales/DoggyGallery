use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<DirectoryEntry>,
    pub page: usize,
    pub per_page: usize,
    pub total_items: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Directory,
    Image,
    Video,
    Audio,
    Archive,
}

impl DirectoryEntry {
    pub fn formatted_size(&self) -> String {
        format_bytes(self.size)
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }

    pub fn is_image(&self) -> bool {
        matches!(self.entry_type, EntryType::Image)
    }

    pub fn is_video(&self) -> bool {
        matches!(self.entry_type, EntryType::Video)
    }

    pub fn is_audio(&self) -> bool {
        matches!(self.entry_type, EntryType::Audio)
    }

    pub fn is_archive(&self) -> bool {
        matches!(self.entry_type, EntryType::Archive)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let i = (bytes_f.log10() / 1024_f64.log10()).floor() as usize;
    let i = i.min(UNITS.len() - 1);

    let size = bytes_f / 1024_f64.powi(i as i32);

    format!("{:.2} {}", size, UNITS[i])
}
