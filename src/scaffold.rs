//! Orchestration: turn a [`ScaffoldRequest`] into files on a [`Filesystem`].

use crate::fs::Filesystem;
use crate::templates::{Context, CrateRole, crate_name, files};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Inputs to the scaffolder.
#[derive(Clone, Debug)]
pub struct ScaffoldRequest {
    /// Lowercase slug used in crate names: `sw-{slug}-isa`, etc.
    pub slug: String,
    /// Human-readable name, e.g. `"IBM 1130"`.
    pub display_name: String,
    /// Rust type name (PascalCase), e.g. `"Ibm1130"`.
    pub type_name: String,
    /// Output directory; the five crate dirs are created as children.
    pub out_dir: PathBuf,
    /// Path prefix for `sw-langtools` framework deps. Default
    /// `"../../sw-langtools"` assumes `--out` is a sibling of
    /// `~/github/sw-langtools`.
    pub framework_path: String,
    /// Overwrite existing files if any.
    pub force: bool,
}

#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("output directory does not exist: {0}")]
    OutDirMissing(PathBuf),
    #[error("file already exists (use --force to overwrite): {0}")]
    AlreadyExists(PathBuf),
    #[error("invalid slug: {0} (must be lowercase ASCII letters, digits, and '-')")]
    InvalidSlug(String),
    #[error("invalid type name: {0} (must be a Rust identifier)")]
    InvalidTypeName(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

fn validate_slug(s: &str) -> Result<(), ScaffoldError> {
    if s.is_empty()
        || !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || s.starts_with('-')
        || s.ends_with('-')
    {
        return Err(ScaffoldError::InvalidSlug(s.to_string()));
    }
    Ok(())
}

fn validate_type_name(s: &str) -> Result<(), ScaffoldError> {
    let mut chars = s.chars();
    let first = chars.next();
    let valid_first = matches!(first, Some(c) if c.is_ascii_alphabetic() || c == '_');
    let valid_rest = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid_first || !valid_rest {
        return Err(ScaffoldError::InvalidTypeName(s.to_string()));
    }
    Ok(())
}

/// Scaffold the five-crate quintet for an ISA.
///
/// Returns the list of files written, in deterministic order (sorted by
/// crate then by file).
pub fn scaffold(
    req: &ScaffoldRequest,
    fs: &mut dyn Filesystem,
) -> Result<Vec<PathBuf>, ScaffoldError> {
    validate_slug(&req.slug)?;
    validate_type_name(&req.type_name)?;

    if !fs.exists(&req.out_dir) {
        return Err(ScaffoldError::OutDirMissing(req.out_dir.clone()));
    }

    let mut written = Vec::new();
    for role in CrateRole::ALL {
        let crate_dir = req.out_dir.join(crate_name(&req.slug, role));
        let ctx = Context {
            slug: &req.slug,
            display_name: &req.display_name,
            type_name: &req.type_name,
            role,
            framework_path: &req.framework_path,
        };
        for (rel_path, content) in files(&ctx) {
            let full_path = crate_dir.join(&rel_path);
            if !req.force && fs.exists(&full_path) {
                return Err(ScaffoldError::AlreadyExists(full_path));
            }
            if let Some(parent) = full_path.parent() {
                fs.create_dir_all(parent)?;
            }
            fs.write(&full_path, &content)?;
            written.push(full_path);
        }
    }
    Ok(written)
}

/// List the files that would be written, without touching the filesystem.
/// Used by `--dry-run`.
pub fn dry_run_paths(req: &ScaffoldRequest) -> Result<Vec<PathBuf>, ScaffoldError> {
    validate_slug(&req.slug)?;
    validate_type_name(&req.type_name)?;
    let mut out = Vec::new();
    for role in CrateRole::ALL {
        let crate_dir: &Path = &req.out_dir.join(crate_name(&req.slug, role));
        let ctx = Context {
            slug: &req.slug,
            display_name: &req.display_name,
            type_name: &req.type_name,
            role,
            framework_path: &req.framework_path,
        };
        for (rel_path, _content) in files(&ctx) {
            out.push(crate_dir.join(&rel_path));
        }
    }
    Ok(out)
}
