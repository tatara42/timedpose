use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Orientation classification for pose images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    Landscape,
    Square,
}

impl Orientation {
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        if height == 0 || width == 0 {
            return Orientation::Square;
        }
        let ratio = width as f32 / height as f32;
        if ratio < 0.95 {
            Orientation::Portrait
        } else if ratio > 1.05 {
            Orientation::Landscape
        } else {
            Orientation::Square
        }
    }
}

/// Filter criteria for pose selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeckFilter {
    #[default]
    All,
    PortraitOnly,
    LandscapeOnly,
    BookmarkedOnly,
}

/// Represents an individual pose reference image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoseItem {
    pub id: usize,
    pub path: PathBuf,
    pub orientation: Option<Orientation>,
    pub bookmarked: bool,
}

impl PoseItem {
    pub fn new(id: usize, path: PathBuf) -> Self {
        Self {
            id,
            path,
            orientation: None,
            bookmarked: false,
        }
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

/// Manages a collection of poses with Fisher-Yates deck shuffling (no repeats until exhausted),
/// historical navigation (Previous / Next), filtering, and bookmarking.
#[derive(Debug, Clone)]
pub struct Deck {
    items: Vec<PoseItem>,
    /// Active playlist of indices into `items` matching the active filter.
    playlist: Vec<usize>,
    /// Index inside `playlist` pointing to the current pose.
    playlist_index: usize,
    /// History of item indices visited in order during the session.
    history: Vec<usize>,
    /// Cursor inside `history` when the user navigates backward with "Previous".
    history_cursor: Option<usize>,
    filter: DeckFilter,
}

impl Deck {
    /// Creates a new Deck from a list of items and shuffles the initial playlist.
    pub fn new(items: Vec<PoseItem>) -> Self {
        let mut deck = Self {
            items,
            playlist: Vec::new(),
            playlist_index: 0,
            history: Vec::new(),
            history_cursor: None,
            filter: DeckFilter::All,
        };
        deck.rebuild_playlist();
        deck
    }

    /// Creates a Deck from file paths.
    pub fn from_paths<P: AsRef<Path>>(paths: &[P]) -> Self {
        let items: Vec<PoseItem> = paths
            .iter()
            .enumerate()
            .map(|(idx, p)| PoseItem::new(idx, p.as_ref().to_path_buf()))
            .collect();
        Self::new(items)
    }

    /// Sets the active filter and reshuffles matching poses.
    pub fn set_filter(&mut self, filter: DeckFilter) {
        if self.filter != filter {
            self.filter = filter;
            self.rebuild_playlist();
        }
    }

    /// Returns the active filter.
    pub fn filter(&self) -> DeckFilter {
        self.filter
    }

    /// Rebuilds the playlist based on the active filter and applies a Fisher-Yates shuffle.
    pub fn rebuild_playlist(&mut self) {
        let mut matching_indices: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| self.matches_filter(item))
            .map(|(idx, _)| idx)
            .collect();

        matching_indices.shuffle(&mut thread_rng());
        self.playlist = matching_indices;
        self.playlist_index = 0;
    }

    fn matches_filter(&self, item: &PoseItem) -> bool {
        match self.filter {
            DeckFilter::All => true,
            DeckFilter::PortraitOnly => item.orientation == Some(Orientation::Portrait),
            DeckFilter::LandscapeOnly => item.orientation == Some(Orientation::Landscape),
            DeckFilter::BookmarkedOnly => item.bookmarked,
        }
    }

    /// Advances to the next pose in the shuffled deck.
    /// If user was browsing backwards in history, moves forward in history first.
    pub fn next(&mut self) -> Option<&PoseItem> {
        if let Some(cursor) = self.history_cursor {
            if cursor + 1 < self.history.len() {
                self.history_cursor = Some(cursor + 1);
                let item_idx = self.history[cursor + 1];
                return self.items.get(item_idx);
            } else {
                // Reached the end of the past history stack; continue with new poses from the playlist
                self.history_cursor = None;
            }
        }

        if self.playlist.is_empty() {
            return None;
        }

        // If playlist is exhausted, re-shuffle for endless cycling
        if self.playlist_index >= self.playlist.len() {
            self.rebuild_playlist();
        }

        let item_idx = self.playlist[self.playlist_index];
        self.playlist_index += 1;
        self.history.push(item_idx);
        self.items.get(item_idx)
    }

    /// Navigates to the previously viewed pose in history.
    pub fn prev(&mut self) -> Option<&PoseItem> {
        if self.history.len() <= 1 {
            return self.current();
        }

        let current_pos = self.history_cursor.unwrap_or(self.history.len() - 1);
        if current_pos > 0 {
            let new_pos = current_pos - 1;
            self.history_cursor = Some(new_pos);
            let item_idx = self.history[new_pos];
            self.items.get(item_idx)
        } else {
            self.current()
        }
    }

    /// Returns a reference to the currently displayed pose.
    pub fn current(&self) -> Option<&PoseItem> {
        let item_idx = if let Some(cursor) = self.history_cursor {
            self.history.get(cursor).copied()
        } else {
            self.history.last().copied()
        };

        item_idx.and_then(|idx| self.items.get(idx))
    }

    /// Toggles the bookmark state for the currently displayed pose.
    /// Returns the new bookmark state, or false if no pose is active.
    pub fn toggle_bookmark_current(&mut self) -> bool {
        let item_idx = if let Some(cursor) = self.history_cursor {
            self.history.get(cursor).copied()
        } else {
            self.history.last().copied()
        };

        if let Some(idx) = item_idx {
            if let Some(item) = self.items.get_mut(idx) {
                item.bookmarked = !item.bookmarked;
                return item.bookmarked;
            }
        }
        false
    }

    /// Total number of items in the library.
    pub fn total_library_count(&self) -> usize {
        self.items.len()
    }

    /// Number of matching poses in the active filtered playlist.
    pub fn playlist_count(&self) -> usize {
        self.playlist.len()
    }

    /// Number of unique poses viewed so far in the current session.
    pub fn session_history_count(&self) -> usize {
        self.history.len()
    }

    /// Returns a list of references to all bookmarked poses.
    pub fn bookmarked_items(&self) -> Vec<&PoseItem> {
        self.items.iter().filter(|i| i.bookmarked).collect()
    }

    /// Returns the full list of visited poses in chronological order.
    pub fn history_items(&self) -> Vec<&PoseItem> {
        self.history
            .iter()
            .filter_map(|&idx| self.items.get(idx))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_shuffle_no_repeats_until_exhausted() {
        let paths: Vec<PathBuf> = (0..5).map(|i| PathBuf::from(format!("pose_{i}.jpg"))).collect();
        let mut deck = Deck::from_paths(&paths);
        assert_eq!(deck.total_library_count(), 5);

        let mut seen = Vec::new();
        for _ in 0..5 {
            let pose = deck.next().expect("should have pose");
            seen.push(pose.id);
        }

        // Must have drawn all 5 unique items
        assert_eq!(seen.len(), 5);
        for i in 0..5 {
            assert!(seen.contains(&i));
        }
    }

    #[test]
    fn test_deck_history_navigation() {
        let paths: Vec<PathBuf> = (0..3).map(|i| PathBuf::from(format!("pose_{i}.jpg"))).collect();
        let mut deck = Deck::from_paths(&paths);

        let p1 = deck.next().unwrap().id;
        let p2 = deck.next().unwrap().id;
        let p3 = deck.next().unwrap().id;

        assert_eq!(deck.current().unwrap().id, p3);

        // Prev -> p2
        assert_eq!(deck.prev().unwrap().id, p2);
        // Prev -> p1
        assert_eq!(deck.prev().unwrap().id, p1);
        // Prev at start stays at p1
        assert_eq!(deck.prev().unwrap().id, p1);

        // Next -> p2
        assert_eq!(deck.next().unwrap().id, p2);
        // Next -> p3
        assert_eq!(deck.next().unwrap().id, p3);
    }

    #[test]
    fn test_bookmark_toggle() {
        let paths = vec![PathBuf::from("pose_a.jpg"), PathBuf::from("pose_b.jpg")];
        let mut deck = Deck::from_paths(&paths);
        deck.next();

        assert!(!deck.current().unwrap().bookmarked);
        assert!(deck.toggle_bookmark_current());
        assert!(deck.current().unwrap().bookmarked);
        assert_eq!(deck.bookmarked_items().len(), 1);

        // Toggle off
        assert!(!deck.toggle_bookmark_current());
        assert!(!deck.current().unwrap().bookmarked);
        assert_eq!(deck.bookmarked_items().len(), 0);
    }
}
