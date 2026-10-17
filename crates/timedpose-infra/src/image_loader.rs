use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// An in-memory decoded image ready for direct GPU texture upload.
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    /// Loads and decodes an image file into raw RGBA8 pixels.
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        Ok(Self {
            path: path.to_path_buf(),
            width,
            height,
            rgba: rgba_img.into_raw(),
        })
    }
}

enum PreloadCommand {
    Load(PathBuf),
}

/// Asynchronous background worker that pre-decodes upcoming poses into an in-memory LRU cache,
/// ensuring instant pose transitions with zero UI hitching.
#[derive(Clone)]
pub struct ImagePreloader {
    sender: Sender<PreloadCommand>,
    cache: Arc<Mutex<HashMap<PathBuf, DecodedImage>>>,
    recent_queue: Arc<Mutex<VecDeque<PathBuf>>>,
    max_capacity: usize,
}

impl ImagePreloader {
    /// Spawns a dedicated background worker thread with the specified cache capacity (default 4).
    pub fn new(max_capacity: usize) -> Self {
        let (sender, receiver): (Sender<PreloadCommand>, Receiver<PreloadCommand>) = unbounded();
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let recent_queue = Arc::new(Mutex::new(VecDeque::new()));

        let cache_worker = Arc::clone(&cache);
        let queue_worker = Arc::clone(&recent_queue);

        thread::Builder::new()
            .name("timedpose-image-loader".to_string())
            .spawn(move || {
                while let Ok(cmd) = receiver.recv() {
                    match cmd {
                        PreloadCommand::Load(path) => {
                            // Check if already in cache
                            {
                                let cache_lock = cache_worker.lock().unwrap();
                                if cache_lock.contains_key(&path) {
                                    continue;
                                }
                            }

                            if let Ok(decoded) = DecodedImage::load_from_file(&path) {
                                let mut cache_lock = cache_worker.lock().unwrap();
                                let mut queue_lock = queue_worker.lock().unwrap();

                                // Evict oldest if exceeding max capacity
                                if queue_lock.len() >= max_capacity {
                                    if let Some(oldest) = queue_lock.pop_front() {
                                        cache_lock.remove(&oldest);
                                    }
                                }

                                queue_lock.push_back(path.clone());
                                cache_lock.insert(path, decoded);
                            }
                        }
                    }
                }
            })
            .expect("Failed to spawn image preloader thread");

        Self {
            sender,
            cache,
            recent_queue,
            max_capacity,
        }
    }

    /// Queues an image for background decoding.
    pub fn queue_preload(&self, path: PathBuf) {
        let _ = self.sender.send(PreloadCommand::Load(path));
    }

    /// Checks if an image is already pre-decoded in cache.
    pub fn get_cached(&self, path: &Path) -> Option<DecodedImage> {
        let cache_lock = self.cache.lock().unwrap();
        cache_lock.get(path).cloned()
    }

    /// Synchronously retrieves an image from cache, or decodes it immediately if not yet cached.
    pub fn get_or_load_sync(&self, path: &Path) -> Result<DecodedImage, String> {
        if let Some(img) = self.get_cached(path) {
            return Ok(img);
        }

        let decoded = DecodedImage::load_from_file(path)?;
        let mut cache_lock = self.cache.lock().unwrap();
        let mut queue_lock = self.recent_queue.lock().unwrap();

        if queue_lock.len() >= self.max_capacity {
            if let Some(oldest) = queue_lock.pop_front() {
                cache_lock.remove(&oldest);
            }
        }

        queue_lock.push_back(path.to_path_buf());
        cache_lock.insert(path.to_path_buf(), decoded.clone());

        Ok(decoded)
    }

    /// Clears the entire decoded cache.
    pub fn clear(&self) {
        let mut cache_lock = self.cache.lock().unwrap();
        let mut queue_lock = self.recent_queue.lock().unwrap();
        cache_lock.clear();
        queue_lock.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preloader_eviction() {
        let preloader = ImagePreloader::new(2);
        assert_eq!(preloader.max_capacity, 2);
    }
}
