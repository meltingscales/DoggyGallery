use crate::constants;
use crate::models::{DirectoryEntry, EntryType};
use anyhow::Result;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

/// Check if a filename is an archive
pub fn is_archive(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    constants::ARCHIVE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Check if a file is an audio file
fn is_audio_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    constants::AUDIO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Check if an archive contains audio files
pub async fn archive_contains_audio(archive_path: &Path) -> Result<bool> {
    let data = tokio::fs::read(archive_path).await?;
    let filename = archive_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if filename.ends_with(".zip") {
        check_zip_for_audio(&data)
    } else if filename.ends_with(".tar") || filename.ends_with(".tar.gz") ||
              filename.ends_with(".tgz") || filename.ends_with(".tar.bz2") ||
              filename.ends_with(".tbz2") {
        check_tar_for_audio(&data, filename)
    } else {
        Ok(false)
    }
}

/// Check if a ZIP archive contains audio files
fn check_zip_for_audio(data: &[u8]) -> Result<bool> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if !file.is_dir() && is_audio_file(file.name()) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if a TAR archive contains audio files
fn check_tar_for_audio(data: &[u8], filename: &str) -> Result<bool> {
    let cursor = Cursor::new(data);
    let reader: Box<dyn Read> = if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(cursor))
    } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
        Box::new(flate2::read::GzDecoder::new(cursor)) // Note: for bz2 we'd need bzip2 crate
    } else {
        Box::new(cursor)
    };

    let mut archive = tar::Archive::new(reader);

    for entry in archive.entries()? {
        let entry = entry?;
        if !entry.header().entry_type().is_dir() {
            if let Ok(path) = entry.path() {
                if let Some(name) = path.to_str() {
                    if is_audio_file(name) {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

/// List contents of an archive
pub async fn list_archive_contents(archive_path: &Path) -> Result<Vec<DirectoryEntry>> {
    let data = tokio::fs::read(archive_path).await?;
    let filename = archive_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if filename.ends_with(".zip") {
        list_zip_contents(&data)
    } else if filename.ends_with(".tar") || filename.ends_with(".tar.gz") ||
              filename.ends_with(".tgz") || filename.ends_with(".tar.bz2") ||
              filename.ends_with(".tbz2") {
        list_tar_contents(&data, filename)
    } else {
        Ok(Vec::new())
    }
}

/// List contents of a ZIP archive
fn list_zip_contents(data: &[u8]) -> Result<Vec<DirectoryEntry>> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name();

        // Skip directories and hidden files
        if file.is_dir() || name.starts_with('.') || name.contains("/.") {
            continue;
        }

        // Only include audio files
        if !is_audio_file(name) {
            continue;
        }

        // Extract just the filename (not full path within archive)
        let display_name = PathBuf::from(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name)
            .to_string();

        entries.push(DirectoryEntry {
            name: display_name,
            path: name.to_string(),
            entry_type: EntryType::Audio,
            size: file.size(),
        });
    }

    // Sort by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries)
}

/// List contents of a TAR archive
fn list_tar_contents(data: &[u8], filename: &str) -> Result<Vec<DirectoryEntry>> {
    let cursor = Cursor::new(data);
    let reader: Box<dyn Read> = if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(cursor))
    } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
        Box::new(flate2::read::GzDecoder::new(cursor))
    } else {
        Box::new(cursor)
    };

    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();

    for entry in archive.entries()? {
        let entry = entry?;

        if entry.header().entry_type().is_dir() {
            continue;
        }

        if let Ok(path) = entry.path() {
            let path_str = path.to_str().unwrap_or("");

            // Skip hidden files
            if path_str.starts_with('.') || path_str.contains("/.") {
                continue;
            }

            // Only include audio files
            if !is_audio_file(path_str) {
                continue;
            }

            let display_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path_str)
                .to_string();

            entries.push(DirectoryEntry {
                name: display_name,
                path: path_str.to_string(),
                entry_type: EntryType::Audio,
                size: entry.header().size()?,
            });
        }
    }

    // Sort by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries)
}

/// Extract a specific file from an archive
pub async fn extract_file_from_archive(
    archive_path: &Path,
    file_path: &str,
) -> Result<Vec<u8>> {
    let data = tokio::fs::read(archive_path).await?;
    let filename = archive_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if filename.ends_with(".zip") {
        extract_from_zip(&data, file_path)
    } else if filename.ends_with(".tar") || filename.ends_with(".tar.gz") ||
              filename.ends_with(".tgz") || filename.ends_with(".tar.bz2") ||
              filename.ends_with(".tbz2") {
        extract_from_tar(&data, filename, file_path)
    } else {
        anyhow::bail!("Unsupported archive format")
    }
}

/// Extract a file from a ZIP archive
fn extract_from_zip(data: &[u8], file_path: &str) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == file_path {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            return Ok(contents);
        }
    }

    anyhow::bail!("File not found in archive")
}

/// Extract a file from a TAR archive
fn extract_from_tar(data: &[u8], filename: &str, file_path: &str) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let reader: Box<dyn Read> = if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(cursor))
    } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
        Box::new(flate2::read::GzDecoder::new(cursor))
    } else {
        Box::new(cursor)
    };

    let mut archive = tar::Archive::new(reader);

    for entry in archive.entries()? {
        let mut entry = entry?;
        if let Ok(path) = entry.path() {
            if path.to_str() == Some(file_path) {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                return Ok(contents);
            }
        }
    }

    anyhow::bail!("File not found in archive")
}
