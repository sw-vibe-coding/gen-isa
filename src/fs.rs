//! Filesystem abstraction.
//!
//! [`RealFs`] writes to disk; [`InMemoryFs`] is a test fixture that records
//! writes in a `BTreeMap` for inspection.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

pub trait Filesystem {
    fn write(&mut self, path: &Path, content: &str) -> io::Result<()>;
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
}

#[derive(Default)]
pub struct RealFs;

impl RealFs {
    pub fn new() -> Self {
        Self
    }
}

impl Filesystem for RealFs {
    fn write(&mut self, path: &Path, content: &str) -> io::Result<()> {
        std::fs::write(path, content)
    }
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

#[derive(Default)]
pub struct InMemoryFs {
    files: BTreeMap<PathBuf, String>,
    dirs: BTreeSet<PathBuf>,
}

impl InMemoryFs {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn files(&self) -> &BTreeMap<PathBuf, String> {
        &self.files
    }
    pub fn dirs(&self) -> &BTreeSet<PathBuf> {
        &self.dirs
    }
    pub fn read(&self, path: &Path) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }
}

impl Filesystem for InMemoryFs {
    fn write(&mut self, path: &Path, content: &str) -> io::Result<()> {
        self.files.insert(path.to_path_buf(), content.to_string());
        Ok(())
    }
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        self.dirs.insert(path.to_path_buf());
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.dirs.contains(path)
    }
}
