//! Per-crate-role templates.
//!
//! Each generated crate has a fixed set of files. Universal files
//! (LICENSE, COPYRIGHT, .gitignore) are role-independent. Role-dependent
//! files (Cargo.toml, src/lib.rs, README.md, per-role module stubs,
//! tests/smoke.rs) are generated from [`Context`].
//!
//! Step 4 (`scaffolder-mvp`) emits empty module stubs. Step 5 fills in
//! `opcode.rs` / `register.rs` / `encode.rs` / `decode.rs` / `branch.rs`
//! / `lib.rs` from a TOML spec.

use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CrateRole {
    Isa,
    Target,
    Codegen,
    Asm,
    Emulator,
}

impl CrateRole {
    pub const ALL: [CrateRole; 5] = [
        CrateRole::Isa,
        CrateRole::Target,
        CrateRole::Codegen,
        CrateRole::Asm,
        CrateRole::Emulator,
    ];

    pub fn suffix(self) -> &'static str {
        match self {
            CrateRole::Isa => "isa",
            CrateRole::Target => "target",
            CrateRole::Codegen => "codegen",
            CrateRole::Asm => "asm",
            CrateRole::Emulator => "emulator",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            CrateRole::Isa => "ISA description: opcodes, encoding, decoding, disassembly",
            CrateRole::Target => "Target description: ABI, calling convention, register classes",
            CrateRole::Codegen => "Codegen: lowering TIR to instructions",
            CrateRole::Asm => "Assembler: text source to instruction bytes",
            CrateRole::Emulator => "Emulator: instruction execution semantics",
        }
    }

    /// Per-role module stubs in `src/`.
    pub fn modules(self) -> &'static [&'static str] {
        match self {
            CrateRole::Isa => &["opcode", "register", "encode", "decode", "branch"],
            CrateRole::Target => &["abi", "classes", "types"],
            CrateRole::Codegen => &["select", "lower", "peephole", "intrinsics"],
            CrateRole::Asm => &["parser", "symtab", "encode"],
            CrateRole::Emulator => &["state", "memory", "exec"],
        }
    }
}

pub fn crate_name(slug: &str, role: CrateRole) -> String {
    format!("sw-{}-{}", slug, role.suffix())
}

pub struct Context<'a> {
    pub slug: &'a str,
    pub display_name: &'a str,
    pub type_name: &'a str,
    pub role: CrateRole,
    pub framework_path: &'a str,
}

pub const LICENSE: &str = r#"MIT License

Copyright (c) 2026 Michael A Wright

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

pub const COPYRIGHT: &str = "Copyright (c) 2026 Michael A Wright\n";

pub const GITIGNORE: &str = "/target\n";

/// Cargo.toml for a generated crate.
pub fn cargo_toml(ctx: &Context) -> String {
    let crate_name = crate_name(ctx.slug, ctx.role);
    let display_name = ctx.display_name;
    let role_desc = ctx.role.description();
    let deps = cargo_deps(ctx);
    format!(
        "[package]\n\
         name = \"{crate_name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\
         description = \"{display_name} {role_desc}\"\n\
         license = \"MIT\"\n\
         \n\
         [dependencies]\n\
         {deps}"
    )
}

fn cargo_deps(ctx: &Context) -> String {
    let fp = ctx.framework_path;
    let slug = ctx.slug;
    match ctx.role {
        CrateRole::Isa => {
            format!("sw-isa-core = {{ path = \"{fp}/sw-isa-core\" }}\n")
        }
        CrateRole::Target => format!(
            "sw-isa-core = {{ path = \"{fp}/sw-isa-core\" }}\n\
             sw-target-core = {{ path = \"{fp}/sw-target-core\" }}\n\
             sw-{slug}-isa = {{ path = \"../sw-{slug}-isa\" }}\n"
        ),
        CrateRole::Codegen => format!(
            "sw-isa-core = {{ path = \"{fp}/sw-isa-core\" }}\n\
             sw-target-core = {{ path = \"{fp}/sw-target-core\" }}\n\
             sw-tir = {{ path = \"{fp}/sw-tir\" }}\n\
             sw-codegen-core = {{ path = \"{fp}/sw-codegen-core\" }}\n\
             sw-{slug}-isa = {{ path = \"../sw-{slug}-isa\" }}\n\
             sw-{slug}-target = {{ path = \"../sw-{slug}-target\" }}\n"
        ),
        CrateRole::Asm => format!(
            "sw-isa-core = {{ path = \"{fp}/sw-isa-core\" }}\n\
             sw-{slug}-isa = {{ path = \"../sw-{slug}-isa\" }}\n"
        ),
        CrateRole::Emulator => format!(
            "sw-isa-core = {{ path = \"{fp}/sw-isa-core\" }}\n\
             sw-{slug}-isa = {{ path = \"../sw-{slug}-isa\" }}\n"
        ),
    }
}

/// `src/lib.rs` for a generated crate.
pub fn lib_rs(ctx: &Context) -> String {
    let display_name = ctx.display_name;
    let role_desc = ctx.role.description();
    let modules: String = ctx
        .role
        .modules()
        .iter()
        .map(|m| format!("pub mod {m};\n"))
        .collect();
    let marker = match ctx.role {
        CrateRole::Isa => format!(
            "/// Marker type that will implement `sw_isa_core::Architecture`.\n\
             #[derive(Copy, Clone, Debug, Eq, PartialEq)]\n\
             pub struct {0};\n",
            ctx.type_name
        ),
        CrateRole::Target => format!(
            "/// Marker type that will implement `sw_target_core::Target`.\n\
             #[derive(Copy, Clone, Debug, Eq, PartialEq)]\n\
             pub struct {0}Target;\n",
            ctx.type_name
        ),
        CrateRole::Codegen => format!(
            "/// Marker type that will implement `sw_codegen_core::Backend`.\n\
             #[derive(Copy, Clone, Debug, Eq, PartialEq)]\n\
             pub struct {0}Backend;\n",
            ctx.type_name
        ),
        CrateRole::Asm | CrateRole::Emulator => String::new(),
    };
    format!(
        "//! `sw-{}-{}`: {}\n\
         //!\n\
         //! Skeleton generated by `gen-isa`. Hand-fill the per-ISA logic\n\
         //! against the trait surfaces in `sw-langtools`.\n\
         \n\
         {}\n\
         {}",
        ctx.slug,
        ctx.role.suffix(),
        format_args!("{display_name} {role_desc}."),
        modules,
        marker,
    )
}

/// Module stub: `//!` doc with role-specific TODO.
pub fn module_stub(role: CrateRole, module: &str) -> String {
    let purpose = match (role, module) {
        (CrateRole::Isa, "opcode") => "Opcode enum + mnemonic + format-of-opcode.",
        (CrateRole::Isa, "register") => "Register identifier + name table + parser.",
        (CrateRole::Isa, "encode") => "Encoding helpers (operands -> bytes).",
        (CrateRole::Isa, "decode") => "Decoding helpers (bytes -> operands).",
        (CrateRole::Isa, "branch") => "Branch range constants and reachability helpers.",
        (CrateRole::Target, "abi") => "CallingConvention impl: arg regs, return reg, saved sets.",
        (CrateRole::Target, "classes") => "RegisterClasses impl: GPR / reserved / fixed pairs.",
        (CrateRole::Target, "types") => "Type widths and alignment.",
        (CrateRole::Codegen, "select") => "Top-level instruction-selection dispatch.",
        (CrateRole::Codegen, "lower") => "Per-IR-op lowering modules.",
        (CrateRole::Codegen, "peephole") => "Target-specific peepholes.",
        (CrateRole::Codegen, "intrinsics") => "Target-specific intrinsic lowering.",
        (CrateRole::Asm, "parser") => "Source text -> AST.",
        (CrateRole::Asm, "symtab") => "Symbol table for two-pass assembly.",
        (CrateRole::Asm, "encode") => "AST + symbol table -> instruction bytes (via `-isa`).",
        (CrateRole::Emulator, "state") => "CPU state struct (registers + flags).",
        (CrateRole::Emulator, "memory") => "Memory model.",
        (CrateRole::Emulator, "exec") => "Instruction execution dispatch.",
        _ => "TODO.",
    };
    format!("//! {purpose}\n//!\n//! Skeleton; hand-fill in saga steps 7-11.\n")
}

/// `tests/smoke.rs`: trivial test asserting the crate compiles and links.
pub fn smoke_test(ctx: &Context) -> String {
    let crate_underscore = crate_name(ctx.slug, ctx.role).replace('-', "_");
    match ctx.role {
        CrateRole::Isa => format!(
            "//! Smoke test: crate compiles and the marker type is constructible.\n\
             \n\
             #[test]\n\
             fn marker_constructs() {{\n\
             \x20   let _ = {crate_underscore}::{0};\n\
             }}\n",
            ctx.type_name
        ),
        CrateRole::Target => format!(
            "#[test]\n\
             fn marker_constructs() {{\n\
             \x20   let _ = {crate_underscore}::{0}Target;\n\
             }}\n",
            ctx.type_name
        ),
        CrateRole::Codegen => format!(
            "#[test]\n\
             fn marker_constructs() {{\n\
             \x20   let _ = {crate_underscore}::{0}Backend;\n\
             }}\n",
            ctx.type_name
        ),
        CrateRole::Asm | CrateRole::Emulator => "#[test]\n\
             fn crate_links() {\n\
             \x20   // Marker; ensures the test target compiles against the crate.\n\
             }\n"
        .to_string(),
    }
}

/// `README.md` for a generated crate.
pub fn readme(ctx: &Context) -> String {
    let crate_name = crate_name(ctx.slug, ctx.role);
    let display_name = ctx.display_name;
    let role_desc = ctx.role.description();
    let slug = ctx.slug;
    format!(
        "# {crate_name}\n\n\
         {display_name} {role_desc}.\n\n\
         ## Status\n\n\
         `0.1.0` skeleton, generated by [`gen-isa`](https://github.com/sw-vibe-coding/gen-isa).\n\
         The trait surface is in place; implementation is pending saga step 7-11.\n\n\
         ## Sibling layout\n\n\
         Cross-crate deps assume sibling clones at\n\
         `~/github/sw-langtools/<framework-crate>` and\n\
         `~/github/<host-org>/sw-{slug}-<role>`. See\n\
         [`gen-isa/docs/decisions.md`](https://github.com/sw-vibe-coding/gen-isa/blob/main/docs/decisions.md)\n\
         Sec 1 for the full org map.\n\n\
         ## License\n\n\
         MIT.\n"
    )
}

/// `docs/abi.md` placeholder for `-target` crates.
pub fn abi_md(ctx: &Context) -> String {
    format!(
        "# {} ABI\n\n\
         Status: stub. Author the calling convention, register saved-sets,\n\
         stack layout, and frame-pointer convention here as `sw-{}-target`\n\
         implementation lands.\n\n\
         ## Calling convention\n\n\
         TODO.\n\n\
         ## Stack\n\n\
         TODO.\n\n\
         ## Frame pointer\n\n\
         TODO.\n",
        ctx.display_name, ctx.slug,
    )
}

/// All files this crate emits, as `(relative-path, content)` tuples.
pub fn files(ctx: &Context) -> Vec<(PathBuf, String)> {
    let mut out: Vec<(PathBuf, String)> = vec![
        (PathBuf::from("Cargo.toml"), cargo_toml(ctx)),
        (PathBuf::from("LICENSE"), LICENSE.to_string()),
        (PathBuf::from("COPYRIGHT"), COPYRIGHT.to_string()),
        (PathBuf::from(".gitignore"), GITIGNORE.to_string()),
        (PathBuf::from("README.md"), readme(ctx)),
        (PathBuf::from("src/lib.rs"), lib_rs(ctx)),
        (PathBuf::from("tests/smoke.rs"), smoke_test(ctx)),
    ];
    for module in ctx.role.modules() {
        out.push((
            PathBuf::from(format!("src/{module}.rs")),
            module_stub(ctx.role, module),
        ));
    }
    if ctx.role == CrateRole::Target {
        out.push((PathBuf::from("docs/abi.md"), abi_md(ctx)));
    }
    out
}
