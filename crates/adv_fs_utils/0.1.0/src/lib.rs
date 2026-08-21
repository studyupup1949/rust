// src/lib.rs
use anyhow::Result;
use bytesize::ByteSize;
use glob::Pattern;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    Started { total_bytes: u64, total_files: usize },
    Progress { bytes_processed: u64 },
    Completed,
    Error(String),
}

pub struct FileSystemUtils;

impl FileSystemUtils {
    /// Recursively find files matching patterns
    pub fn find_files(
        root_path: &Path,
        pattern: Option<&str>,
        recursive: bool,
        case_sensitive: bool,
    ) -> Result<Vec<PathBuf>> {
        let mut matches = Vec::new();
        let pattern = pattern.map(|p| {
            if case_sensitive {
                Pattern::new(p).expect("Invalid glob pattern")
            } else {
                Pattern::new(&p.to_lowercase()).expect("Invalid glob pattern")
            }
        });

        let walker = if recursive {
            WalkDir::new(root_path)
        } else {
            WalkDir::new(root_path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            if let Some(ref pat) = pattern {
                let file_name = if case_sensitive {
                    entry.file_name().to_string_lossy().into_owned()
                } else {
                    entry.file_name().to_string_lossy().to_lowercase()
                };

                if !pat.matches(&file_name) {
                    continue;
                }
            }

            matches.push(entry.into_path());
        }

        Ok(matches)
    }

    /// Find files using regex pattern
    pub fn find_files_regex(root_path: &Path, regex_pattern: &str) -> Result<Vec<PathBuf>> {
        let re = Regex::new(regex_pattern)?;
        let mut matches = Vec::new();

        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy();
            if re.is_match(&file_name) {
                matches.push(entry.into_path());
            }
        }

        Ok(matches)
    }

    /// Calculate directory size (synchronous)
    pub fn get_directory_size(path: &Path) -> Result<u64> {
        let mut total_size = 0;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }

    /// Calculate directory size with human-readable format
    pub fn get_directory_size_human(path: &Path) -> Result<String> {
        let size = Self::get_directory_size(path)?;
        Ok(ByteSize::b(size).to_string())
    }

    /// Async file copy with progress reporting
    pub async fn copy_file_with_progress(
        src: &Path,
        dst: &Path,
        progress_sender: Option<tokio::sync::mpsc::Sender<ProgressUpdate>>,
    ) -> Result<()> {
        let start_time = Instant::now();
        let mut src_file = File::open(src).await?;
        let file_size = src_file.metadata().await?.len();

        if let Some(sender) = &progress_sender {
            sender
                .send(ProgressUpdate::Started {
                    total_bytes: file_size,
                    total_files: 1,
                })
                .await?;
        }

        let mut dst_file = File::create(dst).await?;
        let mut buffer = vec![0u8; 1024 * 64]; // 64KB buffer
        let mut bytes_copied = 0;

        loop {
            let bytes_read = src_file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            dst_file.write_all(&buffer[..bytes_read]).await?;
            bytes_copied += bytes_read as u64;

            if let Some(sender) = &progress_sender {
                if sender
                    .send(ProgressUpdate::Progress {
                        bytes_processed: bytes_copied,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }

        if let Some(sender) = progress_sender {
            sender.send(ProgressUpdate::Completed).await?;
        }

        let duration = start_time.elapsed();
        log::info!(
            "Copied {} in {:.2?} ({}/s)",
            ByteSize::b(bytes_copied),
            duration,
            ByteSize::b((bytes_copied as f64 / duration.as_secs_f64()) as u64)
        );

        Ok(())
    }

    /// Recursive directory copy with progress reporting
    pub async fn copy_directory_with_progress(
        src: &Path,
        dst: &Path,
        progress_sender: Option<tokio::sync::mpsc::Sender<ProgressUpdate>>,
    ) -> Result<()> {
        // Create destination directory if it doesn't exist
        fs::create_dir_all(dst).await?;

        let mut total_bytes = 0;
        let mut file_count = 0;
        let mut file_paths = Vec::new();

        // First pass: calculate total size and collect files
        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                total_bytes += entry.metadata()?.len();
                file_count += 1;
                file_paths.push(entry.path().to_path_buf());
            }
        }

        if let Some(sender) = &progress_sender {
            sender
                .send(ProgressUpdate::Started {
                    total_bytes,
                    total_files: file_count,
                })
                .await?;
        }

        // Second pass: copy files
        let mut bytes_processed = 0;
        for src_path in file_paths {
            let relative_path = src_path.strip_prefix(src)?;
            let dst_path = dst.join(relative_path);

            // Create parent directories if needed
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let file_size = src_path.metadata()?.len();
            Self::copy_file_with_progress(&src_path, &dst_path, None).await?;

            bytes_processed += file_size;
            if let Some(sender) = &progress_sender {
                if sender
                    .send(ProgressUpdate::Progress {
                        bytes_processed,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }

        if let Some(sender) = progress_sender {
            sender.send(ProgressUpdate::Completed).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_copy_file_with_progress() {
        let temp_dir = tempdir().unwrap();
        let src_path = temp_dir.path().join("test.txt");
        let dst_path = temp_dir.path().join("test_copy.txt");

        tokio::fs::write(&src_path, "Hello, world!").await.unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        FileSystemUtils::copy_file_with_progress(&src_path, &dst_path, Some(tx))
            .await
            .unwrap();

        let mut progress_updates = 0;
        while let Some(update) = rx.recv().await {
            match update {
                ProgressUpdate::Started { .. } => progress_updates += 1,
                ProgressUpdate::Progress { .. } => progress_updates += 1,
                ProgressUpdate::Completed => progress_updates += 1,
                _ => (),
            }
        }

        assert!(progress_updates >= 2); // At least Started and Completed
        assert_eq!(
            tokio::fs::read_to_string(&src_path).await.unwrap(),
            tokio::fs::read_to_string(&dst_path).await.unwrap()
        );
    }

    #[test]
    fn test_find_files() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.log");
        std::fs::write(file1, "").unwrap();
        std::fs::write(file2, "").unwrap();

        let txt_files = FileSystemUtils::find_files(temp_dir.path(), Some("*.txt"), false, false)
            .unwrap();
        assert_eq!(txt_files.len(), 1);
        assert!(txt_files[0].file_name().unwrap().to_string_lossy().ends_with(".txt"));
    }

    #[test]
    fn test_directory_size() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1");
        let file2 = temp_dir.path().join("file2");
        std::fs::write(file1, "12345").unwrap(); // 5 bytes
        std::fs::write(file2, "1234567890").unwrap(); // 10 bytes

        let size = FileSystemUtils::get_directory_size(temp_dir.path()).unwrap();
        assert_eq!(size, 15);

        let human_size = FileSystemUtils::get_directory_size_human(temp_dir.path()).unwrap();
        assert!(human_size.contains("15 B"));
    }
}