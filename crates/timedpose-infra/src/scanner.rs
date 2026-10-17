use jwalk::WalkDir;
use std::path::Path;
use timedpose_core::{Orientation, PoseItem};

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp"];

/// Discovers and indexes image reference files.
pub struct ImageScanner;

impl ImageScanner {
    /// Returns true if the file extension matches a supported image format.
    pub fn is_supported_image<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                let ext_lower = ext.to_lowercase();
                SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
            })
            .unwrap_or(false)
    }

    /// Recursively scans a single directory for supported images, quickly inspecting
    /// image headers to determine orientation without decoding pixel payloads.
    pub fn scan_directory<P: AsRef<Path>>(dir_path: P) -> Vec<PoseItem> {
        let mut results = Vec::new();
        let mut id = 0;

        for entry in WalkDir::new(dir_path).follow_links(true) {
            let Ok(entry) = entry else { continue };
            if entry.file_type.is_file() && Self::is_supported_image(entry.path()) {
                let path = entry.path();
                let orientation = Self::probe_orientation(&path);

                let mut item = PoseItem::new(id, path);
                if let Some(orient) = orientation {
                    item = item.with_orientation(orient);
                }
                results.push(item);
                id += 1;
            }
        }

        results
    }

    /// Recursively scans multiple directories, combining all found poses.
    pub fn scan_directories<P: AsRef<Path>>(dirs: &[P]) -> Vec<PoseItem> {
        let mut combined = Vec::new();
        let mut id = 0;

        for dir in dirs {
            let poses = Self::scan_directory(dir);
            for mut pose in poses {
                pose.id = id;
                combined.push(pose);
                id += 1;
            }
        }

        combined
    }

    /// Quickly probes image width and height from file headers without full decode.
    pub fn probe_orientation(path: &Path) -> Option<Orientation> {
        image::image_dimensions(path)
            .ok()
            .map(|(w, h)| Orientation::from_dimensions(w, h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_supported_extensions() {
        assert!(ImageScanner::is_supported_image("test.jpg"));
        assert!(ImageScanner::is_supported_image("test.JPEG"));
        assert!(ImageScanner::is_supported_image("sub/folder/pose.png"));
        assert!(ImageScanner::is_supported_image("pose.webp"));
        assert!(ImageScanner::is_supported_image("pose.bmp"));
        assert!(!ImageScanner::is_supported_image("notes.txt"));
        assert!(!ImageScanner::is_supported_image("model.blend"));
        assert!(!ImageScanner::is_supported_image("video.mp4"));
    }

    #[test]
    fn test_scan_directory_discovers_images() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("timedpose_test_scanner_{nanos}"));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("subfolder");
        std::fs::create_dir_all(&sub_dir).unwrap();

        File::create(temp_dir.join("pose1.jpg")).unwrap().write_all(b"fake").unwrap();
        File::create(sub_dir.join("pose2.png")).unwrap().write_all(b"fake").unwrap();
        File::create(temp_dir.join("readme.md")).unwrap().write_all(b"text").unwrap();

        let results = ImageScanner::scan_directory(&temp_dir);
        assert_eq!(results.len(), 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
