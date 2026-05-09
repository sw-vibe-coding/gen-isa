//! Orchestration: turn a [`ScaffoldRequest`] into files on a [`Filesystem`].

use crate::emit;
use crate::fs::Filesystem;
use crate::spec::Spec;
use crate::templates::{Context, CrateRole, crate_name, files};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Inputs to the scaffolder.
#[derive(Debug)]
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
    /// Optional ISA spec; when provided, the `-isa` crate's mechanical
    /// modules (`opcode.rs`, `register.rs`, `encode.rs`, `decode.rs`,
    /// `branch.rs`, `lib.rs`, `tests/roundtrip.rs`) are emitted from it.
    pub spec: Option<Spec>,
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
        let mut crate_files = files(&ctx);

        // Spec-driven overrides: when --spec is set and we're scaffolding
        // the -isa crate, replace the relevant module stubs with generated
        // code. Other crates are unchanged.
        if role == CrateRole::Isa
            && let Some(spec) = req.spec.as_ref()
        {
            apply_spec_overrides(&mut crate_files, spec, &req.type_name, &req.slug);
        }

        for (rel_path, content) in crate_files {
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

fn apply_spec_overrides(
    files: &mut Vec<(PathBuf, String)>,
    spec: &Spec,
    type_name: &str,
    slug: &str,
) {
    let overrides: Vec<(PathBuf, String)> = vec![
        (PathBuf::from("src/opcode.rs"), emit::opcode_rs(spec)),
        (PathBuf::from("src/register.rs"), emit::register_rs(spec)),
        (PathBuf::from("src/branch.rs"), emit::branch_rs(spec)),
        (PathBuf::from("src/encode.rs"), emit::encode_rs(spec)),
        (PathBuf::from("src/decode.rs"), emit::decode_rs(spec)),
        (
            PathBuf::from("src/lib.rs"),
            emit::lib_rs(spec, type_name, slug),
        ),
        (
            PathBuf::from("tests/roundtrip.rs"),
            emit::roundtrip_test(spec, type_name, slug),
        ),
    ];
    for (path, content) in overrides {
        if let Some(existing) = files.iter_mut().find(|(p, _)| *p == path) {
            existing.1 = content;
        } else {
            files.push((path, content));
        }
    }
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
        let mut crate_files = files(&ctx);
        if role == CrateRole::Isa && req.spec.is_some() {
            // Roundtrip test is only added in spec mode.
            if !crate_files
                .iter()
                .any(|(p, _)| p.as_os_str() == "tests/roundtrip.rs")
            {
                crate_files.push((PathBuf::from("tests/roundtrip.rs"), String::new()));
            }
        }
        for (rel_path, _content) in crate_files {
            out.push(crate_dir.join(&rel_path));
        }
    }
    Ok(out)
}
