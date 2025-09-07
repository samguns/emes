use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Track {
    pub path: PathBuf,
    pub name: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<std::time::Duration>,
}

impl Track {
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            path,
            name,
            artist: None,
            album: None,
            duration: None,
        }
    }
}

pub struct Playlist {
    tracks: Vec<Track>,
    current_index: Option<usize>,
    selected_index: usize,
    repeat: bool,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            selected_index: 0,
            repeat: false,
        }
    }

    pub fn add_file(&mut self, path: PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
        }

        // Check if it's an audio file
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "mp3" | "flac" | "ogg" | "wav" | "m4a" | "aac") {
                self.tracks.push(Track::from_path(path));
                if self.current_index.is_none() && !self.tracks.is_empty() {
                    self.current_index = Some(0);
                }
            }
        }

        Ok(())
    }

    pub fn load_directory(&mut self, dir: PathBuf) -> Result<()> {
        if !dir.is_dir() {
            return Err(anyhow::anyhow!("Not a directory: {}", dir.display()));
        }

        let mut files = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_str().unwrap_or("").to_lowercase();
                    if matches!(ext.as_str(), "mp3" | "flac" | "ogg" | "wav" | "m4a" | "aac") {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        // Sort files alphabetically
        files.sort();

        for file in files {
            self.tracks.push(Track::from_path(file));
        }

        if self.current_index.is_none() && !self.tracks.is_empty() {
            self.current_index = Some(0);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.selected_index = 0;
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        self.tracks.shuffle(&mut rng);

        if !self.tracks.is_empty() {
            self.current_index = Some(0);
            self.selected_index = 0;
        }
    }

    pub fn next(&mut self) -> bool {
        if self.tracks.is_empty() {
            return false;
        }

        if let Some(index) = self.current_index {
            if index + 1 < self.tracks.len() {
                self.current_index = Some(index + 1);
                self.selected_index = index + 1;
                true
            } else if self.repeat {
                self.current_index = Some(0);
                self.selected_index = 0;
                true
            } else {
                false
            }
        } else if !self.tracks.is_empty() {
            self.current_index = Some(0);
            self.selected_index = 0;
            true
        } else {
            false
        }
    }

    pub fn previous(&mut self) -> bool {
        if self.tracks.is_empty() {
            return false;
        }

        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
                self.selected_index = index - 1;
                true
            } else if self.repeat {
                let last = self.tracks.len() - 1;
                self.current_index = Some(last);
                self.selected_index = last;
                true
            } else {
                false
            }
        } else if !self.tracks.is_empty() {
            self.current_index = Some(0);
            self.selected_index = 0;
            true
        } else {
            false
        }
    }

    pub fn move_selection_up(&mut self) {
        if !self.tracks.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.tracks.is_empty() && self.selected_index < self.tracks.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn play_selected(&mut self) {
        if !self.tracks.is_empty() {
            self.current_index = Some(self.selected_index);
        }
    }

    pub fn current(&self) -> Option<&Track> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn get_tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }

    pub fn is_repeat(&self) -> bool {
        self.repeat
    }
}
