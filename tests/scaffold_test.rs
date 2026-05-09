//! Integration tests for the scaffolder.

use gen_isa::scaffold::dry_run_paths;
use gen_isa::{InMemoryFs, ScaffoldRequest, scaffold};
use std::path::{Path, PathBuf};

fn req(slug: &str) -> ScaffoldRequest {
    ScaffoldRequest {
        slug: slug.to_string(),
        display_name: "Test ISA".to_string(),
        type_name: "TestIsa".to_string(),
        out_dir: PathBuf::from("/out"),
        framework_path: "../../sw-langtools".to_string(),
        force: false,
        spec: None,
    }
}

#[test]
fn scaffold_writes_quintet() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    let r = req("ibm1130");
    let written = scaffold(&r, &mut fs).expect("scaffold ok");
    let crates = ["isa", "target", "codegen", "asm", "emulator"];
    for role in crates {
        let crate_dir = format!("/out/sw-ibm1130-{role}");
        let cargo = format!("{crate_dir}/Cargo.toml");
        let lib = format!("{crate_dir}/src/lib.rs");
        let smoke = format!("{crate_dir}/tests/smoke.rs");
        let license = format!("{crate_dir}/LICENSE");
        let readme = format!("{crate_dir}/README.md");
        assert!(
            fs.read(Path::new(&cargo)).is_some(),
            "missing {cargo}; got {written:?}"
        );
        assert!(fs.read(Path::new(&lib)).is_some(), "missing {lib}");
        assert!(fs.read(Path::new(&smoke)).is_some(), "missing {smoke}");
        assert!(fs.read(Path::new(&license)).is_some(), "missing {license}");
        assert!(fs.read(Path::new(&readme)).is_some(), "missing {readme}");
    }
    assert_eq!(written.len(), expected_file_count());
}

#[test]
fn scaffold_target_has_abi_doc() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    let r = req("ibm1130");
    scaffold(&r, &mut fs).unwrap();
    assert!(
        fs.read(Path::new("/out/sw-ibm1130-target/docs/abi.md"))
            .is_some()
    );
}

#[test]
fn scaffold_isa_lib_contains_marker_type() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    scaffold(&req("ibm1130"), &mut fs).unwrap();
    let lib = fs
        .read(Path::new("/out/sw-ibm1130-isa/src/lib.rs"))
        .unwrap();
    assert!(lib.contains("pub struct TestIsa;"), "lib was: {lib}");
    assert!(lib.contains("pub mod opcode;"));
    assert!(lib.contains("pub mod encode;"));
}

#[test]
fn scaffold_codegen_cargo_has_three_framework_deps() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    scaffold(&req("ibm1130"), &mut fs).unwrap();
    let cargo = fs
        .read(Path::new("/out/sw-ibm1130-codegen/Cargo.toml"))
        .unwrap();
    assert!(cargo.contains("sw-isa-core"));
    assert!(cargo.contains("sw-target-core"));
    assert!(cargo.contains("sw-tir"));
    assert!(cargo.contains("sw-codegen-core"));
    assert!(cargo.contains("sw-ibm1130-isa"));
    assert!(cargo.contains("sw-ibm1130-target"));
}

#[test]
fn scaffold_rejects_invalid_slug() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    let bad_slugs = ["IBM1130", "ibm 1130", "-leading", "trailing-", ""];
    for slug in bad_slugs {
        let r = ScaffoldRequest {
            slug: slug.to_string(),
            display_name: "X".into(),
            type_name: "X".into(),
            out_dir: PathBuf::from("/out"),
            framework_path: "../sw-langtools".into(),
            force: false,
            spec: None,
        };
        assert!(
            scaffold(&r, &mut fs).is_err(),
            "should reject slug {slug:?}"
        );
    }
}

#[test]
fn scaffold_rejects_already_existing_without_force() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    scaffold(&req("ibm1130"), &mut fs).unwrap();
    let result = scaffold(&req("ibm1130"), &mut fs);
    assert!(
        result.is_err(),
        "second scaffold without --force should fail"
    );
}

#[test]
fn scaffold_force_overwrites() {
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    scaffold(&req("ibm1130"), &mut fs).unwrap();
    let mut r = req("ibm1130");
    r.force = true;
    scaffold(&r, &mut fs).expect("force should succeed");
}

#[test]
fn scaffold_with_spec_overrides_isa_modules() {
    use gen_isa::Spec;
    let spec_text =
        std::fs::read_to_string("docs/spec-examples/ibm1130.toml").expect("ibm1130 spec example");
    let spec = Spec::parse(&spec_text).expect("parse ibm1130");
    let mut fs = InMemoryFs::new();
    fs.create_dir_all_for_test("/out");
    let mut r = req("ibm1130");
    r.spec = Some(spec);
    scaffold(&r, &mut fs).unwrap();

    // Generated opcode.rs has spec-driven content (not the empty stub).
    let opcode = fs
        .read(Path::new("/out/sw-ibm1130-isa/src/opcode.rs"))
        .unwrap();
    assert!(
        opcode.contains("pub enum Opcode"),
        "opcode.rs missing enum: {opcode}"
    );
    assert!(opcode.contains("Load"));
    assert!(opcode.contains("Add"));

    // lib.rs has Architecture impl
    let lib = fs
        .read(Path::new("/out/sw-ibm1130-isa/src/lib.rs"))
        .unwrap();
    assert!(lib.contains("impl sw_isa_core::Architecture for TestIsa"));
    assert!(lib.contains("pub enum Instruction"));

    // roundtrip test exists for spec mode
    let rt = fs
        .read(Path::new("/out/sw-ibm1130-isa/tests/roundtrip.rs"))
        .unwrap();
    assert!(rt.contains("roundtrip_short") || rt.contains("roundtrip_long"));
}

#[test]
fn spec_parse_validates_examples() {
    use gen_isa::Spec;
    for path in [
        "docs/spec-examples/cor24.toml",
        "docs/spec-examples/ibm1130.toml",
    ] {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        Spec::parse(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}

#[test]
fn spec_validation_rejects_dangling_format() {
    use gen_isa::Spec;
    let text = r#"
[arch]
display_name = "X"
type_name = "X"
crate_slug = "x"

[memory]
address_unit = "Byte"
endian = "Big"
word_bits = 8
min_instr_bytes = 1
max_instr_bytes = 1

[branch]
short_offset_min = -128
short_offset_max = 127
max_short_branch_instructions = 31
pipeline_delay_bytes = 0

[encoding]
style = "rom_table"
length_dispatch = "per_opcode"

[[format]]
name = "f1"
size_bytes = 1

[[opcode]]
name = "Bad"
mnemonic = "bad"
value = 0
format = "does_not_exist"
"#;
    assert!(Spec::parse(text).is_err());
}

#[test]
fn dry_run_lists_paths() {
    let r = req("ibm1130");
    let paths = dry_run_paths(&r).unwrap();
    assert_eq!(paths.len(), expected_file_count());
    assert!(
        paths
            .iter()
            .any(|p| p.ends_with("sw-ibm1130-isa/Cargo.toml"))
    );
}

fn expected_file_count() -> usize {
    // 5 crates, each with: Cargo.toml, LICENSE, COPYRIGHT, .gitignore,
    // README.md, src/lib.rs, tests/smoke.rs = 7 universal files.
    // Plus per-role module stubs: isa=5, target=3+1(abi.md), codegen=4,
    // asm=3, emulator=3.
    let universal = 7;
    let per_role = [5, 3 + 1, 4, 3, 3];
    5 * universal + per_role.iter().sum::<usize>()
}

// Test-only helper to bypass the create_dir_all-needed-first contract
// when we only care about the simulated /out root.
trait TestHelpers {
    fn create_dir_all_for_test(&mut self, path: &str);
}

impl TestHelpers for InMemoryFs {
    fn create_dir_all_for_test(&mut self, path: &str) {
        use gen_isa::Filesystem;
        self.create_dir_all(Path::new(path)).unwrap();
    }
}
