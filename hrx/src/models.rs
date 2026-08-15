//! Data models for HRX archives.

use std::collections::BTreeMap;
use std::fmt;
use tracing::{debug, trace};

/// The maximum line length allowed by the HRX spec (1MB).
pub const MAX_LINE_LENGTH: usize = 1_048_576;

/// The boundary marker that starts each file entry.
pub const BOUNDARY_MARKER: &str = "<===>";

/// The comment marker used for directory boundaries.
pub const DIR_BOUNDARY_COMMENT: &str = "================================================================================";

/// A single entry in an HRX archive - either a file or a subdirectory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A file with its path and contents.
    File(FileEntry),
    /// A directory with its path and child entries.
    Dir(DirEntry),
}

impl Entry {
    /// Returns the path of this entry.
    pub fn path(&self) -> &str {
        match self {
            Entry::File(f) => &f.path,
            Entry::Dir(d) => &d.path,
        }
    }

    /// Returns true if this is a file entry.
    pub fn is_file(&self) -> bool {
        matches!(self, Entry::File(_))
    }

    /// Returns true if this is a directory entry.
    pub fn is_dir(&self) -> bool {
        matches!(self, Entry::Dir(_))
    }
}

/// A file entry in an HRX archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// The path of this file relative to the archive root.
    pub path: String,
    /// The contents of this file.
    pub contents: String,
}

impl FileEntry {
    /// Creates a new file entry.
    pub fn new(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
        }
    }
}

/// A directory entry in an HRX archive.
///
/// Directories in HRX are implicit - they exist when files have nested paths.
/// Directory boundaries (shown as HRX comments with `=` characters) indicate
/// where one set of files ends and another begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// The path of this directory relative to the archive root.
    pub path: String,
    /// Child entries (files and subdirectories) in this directory.
    pub children: Vec<Entry>,
}

impl DirEntry {
    /// Creates a new directory entry.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            children: Vec::new(),
        }
    }
}

/// A parsed HRX archive representing a virtual filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    /// The root entries of the archive.
    entries: Vec<Entry>,
    /// Fast lookup from path to file entry index (entry_index, None for dirs).
    file_index: BTreeMap<String, usize>,
}

impl Archive {
    /// Creates an empty archive.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            file_index: BTreeMap::new(),
        }
    }

    /// Builds an archive (with file index) from a list of entries.
    pub fn from_entries(entries: Vec<Entry>) -> Self {
        let mut file_index = BTreeMap::new();
        Self::build_index(&entries, "", &mut file_index);
        Self { entries, file_index }
    }

    fn build_index(
        entries: &[Entry],
        prefix: &str,
        index: &mut BTreeMap<String, usize>,
    ) {
        for entry in entries {
            let full_path = if prefix.is_empty() {
                entry.path().to_string()
            } else {
                format!("{}/{}", prefix, entry.path())
            };
            match entry {
                Entry::File(_) => {
                    index.insert(full_path, index.len());
                }
                Entry::Dir(d) => {
                    Self::build_index(&d.children, &full_path, index);
                }
            }
        }
    }

    /// Returns all top-level entries.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Returns true if the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of top-level entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Gets a file entry by its path.
    pub fn get_file(&self, path: &str) -> Option<&FileEntry> {
        trace!(path, "looking up file");
        // Find in entries recursively
        Self::find_file(&self.entries, path)
    }

    fn find_file<'a>(entries: &'a [Entry], path: &str) -> Option<&'a FileEntry> {
        for entry in entries {
            match entry {
                Entry::File(f) if f.path == path => return Some(f),
                Entry::File(f) => {
                    // Check if it's a nested path match
                    if path.starts_with(&f.path) {
                        // continue
                    }
                }
                Entry::Dir(d) => {
                    let prefix = format!("{}/", d.path);
                    if path.starts_with(&prefix) {
                        let subpath = &path[prefix.len()..];
                        if let found @ Some(_) = Self::find_file(&d.children, subpath) {
                            return found;
                        }
                    } else if d.path == path {
                        // It's a directory, not a file
                        return None;
                    }
                }
            }
        }
        None
    }

    /// Gets a directory entry by its path.
    pub fn get_dir(&self, path: &str) -> Option<&DirEntry> {
        trace!(path, "looking up directory");
        if path.is_empty() {
            return None;
        }
        Self::find_dir(&self.entries, path)
    }

    fn find_dir<'a>(entries: &'a [Entry], path: &str) -> Option<&'a DirEntry> {
        for entry in entries {
            if let Entry::Dir(d) = entry {
                if d.path == path {
                    return Some(d);
                }
                let prefix = format!("{}/", d.path);
                if path.starts_with(&prefix) {
                    let subpath = &path[prefix.len()..];
                    if let found @ Some(_) = Self::find_dir(&d.children, subpath) {
                        return found;
                    }
                }
            }
        }
        None
    }

    /// Returns all file entries in the archive as a flat map.
    pub fn files(&self) -> &BTreeMap<String, usize> {
        &self.file_index
    }

    /// Adds a file entry to the archive.
    pub fn add_file(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        let path = path.into();
        debug!(path = %path, "added file to archive");
        let entry = Entry::File(FileEntry::new(path.clone(), contents));
        self.entries.push(entry);
        self.file_index.insert(path, self.file_index.len());
    }

    /// Adds a directory entry to the archive.
    pub fn add_dir(&mut self, dir: DirEntry) {
        let path = dir.path.clone();
        self.entries.push(Entry::Dir(dir));
        debug!(path, "added directory to archive");
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Archive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Archive {{ entries: {}, files: {} }}", self.entries.len(), self.file_index.len())
    }
}

/// Convenience function to create a file entry.
pub fn file(path: impl Into<String>, contents: impl Into<String>) -> Entry {
    Entry::File(FileEntry::new(path, contents))
}

/// Convenience function to create a directory entry.
pub fn dir(path: impl Into<String>, children: Vec<Entry>) -> Entry {
    let mut d = DirEntry::new(path);
    d.children = children;
    Entry::Dir(d)
}
