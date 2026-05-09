//! gen-isa: ISA-specific Rust crate scaffolder.
//!
//! Given an ISA's slug, display name, and Rust type name, emits the
//! five-crate quintet skeleton (`sw-{slug}-{isa,target,codegen,asm,emulator}`)
//! ready for `git init` + `gh repo create --source --push`.
//!
//! Step 4 (`scaffolder-mvp`): templates only, no spec parsing. Step 5
//! (`scaffolder-codegen`) adds spec-driven emission for the mechanical
//! bits (opcode tables, register tables, encode/decode bit-field math).

pub mod fs;
pub mod scaffold;
pub mod templates;

pub use fs::{Filesystem, InMemoryFs, RealFs};
pub use scaffold::{ScaffoldError, ScaffoldRequest, scaffold};
pub use templates::{Context, CrateRole, crate_name};
