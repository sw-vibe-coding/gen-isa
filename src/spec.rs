//! ISA TOML spec parsing and validation.
//!
//! See `gen-isa/docs/spec-format.md` for the format definition. The structs
//! here mirror the documented shape one-to-one; their `Deserialize` impls
//! are derived. [`Spec::parse`] runs validation as part of parsing.

use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub arch: Arch,
    pub memory: Memory,
    pub branch: Branch,
    pub encoding: Encoding,
    #[serde(rename = "register", default)]
    pub registers: Vec<Register>,
    #[serde(rename = "register_bank", default)]
    pub register_banks: Vec<RegisterBank>,
    #[serde(rename = "fixed_pair", default)]
    pub fixed_pairs: Vec<FixedPair>,
    #[serde(rename = "format")]
    pub formats: Vec<Format>,
    #[serde(rename = "opcode")]
    pub opcodes: Vec<Opcode>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arch {
    pub display_name: String,
    pub type_name: String,
    pub crate_slug: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub address_unit: AddressUnit,
    pub endian: Endian,
    pub word_bits: u32,
    pub min_instr_bytes: usize,
    pub max_instr_bytes: usize,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum AddressUnit {
    Byte,
    Word16,
    Word24,
    Word32,
}

impl AddressUnit {
    pub fn rust_variant(self) -> &'static str {
        match self {
            AddressUnit::Byte => "Byte",
            AddressUnit::Word16 => "Word16",
            AddressUnit::Word24 => "Word24",
            AddressUnit::Word32 => "Word32",
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    pub fn rust_variant(self) -> &'static str {
        match self {
            Endian::Big => "Big",
            Endian::Little => "Little",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    pub short_offset_min: i64,
    pub short_offset_max: i64,
    pub max_short_branch_instructions: u32,
    pub pipeline_delay_bytes: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Encoding {
    pub style: EncodingStyle,
    pub length_dispatch: LengthDispatch,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EncodingStyle {
    BitFields,
    RomTable,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LengthDispatch {
    PerOpcode,
    FirstWordBits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    pub name: String,
    pub class: RegClass,
    pub index: u32,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegClass {
    Gpr,
    Acc,
    Ext,
    Xr,
    Reserved,
    Pc,
    Sp,
    Fp,
    Flags,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterBank {
    pub name_prefix: String,
    pub class: RegClass,
    pub count: u32,
    #[serde(default)]
    pub index_base: u32,
    #[serde(default)]
    pub zero_register: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedPair {
    pub hi: String,
    pub lo: String,
    pub purpose: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Format {
    pub name: String,
    pub size_bytes: usize,
    #[serde(default)]
    pub discriminator: Option<Discriminator>,
    #[serde(rename = "field", default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discriminator {
    pub offset: u32,
    pub width: u32,
    pub value: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub kind: FieldKind,
    pub offset: u32,
    pub width: u32,
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub value: Option<u64>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Opcode,
    Reg,
    Imm,
    Disp,
    Indirect,
    Tag,
    Reserved,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opcode {
    pub name: String,
    pub mnemonic: String,
    pub value: u64,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub formats: Option<Vec<String>>,
    #[serde(default)]
    pub role: Option<String>,
}

impl Opcode {
    /// All format names this opcode appears in (1 for single-format, N for multi-format).
    pub fn format_names(&self) -> Vec<&str> {
        if let Some(ref f) = self.format {
            vec![f.as_str()]
        } else if let Some(ref fs) = self.formats {
            fs.iter().map(String::as_str).collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("toml parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid spec: {0}")]
    Invalid(String),
}

impl Spec {
    pub fn parse(text: &str) -> Result<Spec, SpecError> {
        let spec: Spec = toml::from_str(text)?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn parse_file(path: &std::path::Path) -> Result<Spec, SpecError> {
        let text = std::fs::read_to_string(path)?;
        Self::parse(&text)
    }

    fn validate(&self) -> Result<(), SpecError> {
        let invalid = |msg: String| SpecError::Invalid(msg);

        // Format names must be unique.
        let mut format_names = BTreeSet::new();
        for f in &self.formats {
            if !format_names.insert(f.name.as_str()) {
                return Err(invalid(format!("duplicate format name: {}", f.name)));
            }
        }

        // Every opcode references exactly one of {format, formats}; each
        // referenced format must exist.
        let mut opcode_names = BTreeSet::new();
        for op in &self.opcodes {
            if !opcode_names.insert(op.name.as_str()) {
                return Err(invalid(format!("duplicate opcode name: {}", op.name)));
            }
            match (&op.format, &op.formats) {
                (Some(_), Some(_)) => {
                    return Err(invalid(format!(
                        "opcode {} sets both format and formats",
                        op.name
                    )));
                }
                (None, None) => {
                    return Err(invalid(format!(
                        "opcode {} sets neither format nor formats",
                        op.name
                    )));
                }
                _ => {}
            }
            for fname in op.format_names() {
                if !format_names.contains(fname) {
                    return Err(invalid(format!(
                        "opcode {} references unknown format {fname}",
                        op.name
                    )));
                }
            }
        }

        // Opcode values unique within a format.
        for f in &self.formats {
            let mut seen: BTreeSet<u64> = BTreeSet::new();
            for op in &self.opcodes {
                if op.format_names().iter().any(|n| *n == f.name) && !seen.insert(op.value) {
                    return Err(invalid(format!(
                        "duplicate opcode value 0x{:x} in format {}",
                        op.value, f.name
                    )));
                }
            }
        }

        // Register index uniqueness.
        let mut reg_indices = BTreeSet::new();
        let mut reg_names = BTreeSet::new();
        for r in &self.registers {
            if !reg_indices.insert(r.index) {
                return Err(invalid(format!(
                    "duplicate register index {} (register {})",
                    r.index, r.name
                )));
            }
            if !reg_names.insert(r.name.as_str()) {
                return Err(invalid(format!("duplicate register name: {}", r.name)));
            }
        }

        // Fixed pairs reference existing registers.
        for p in &self.fixed_pairs {
            if !reg_names.contains(p.hi.as_str()) {
                return Err(invalid(format!(
                    "fixed_pair references unknown hi register: {}",
                    p.hi
                )));
            }
            if !reg_names.contains(p.lo.as_str()) {
                return Err(invalid(format!(
                    "fixed_pair references unknown lo register: {}",
                    p.lo
                )));
            }
        }

        // bit_fields encoding: each format covers exactly size_bytes*8 bits with
        // no gaps / overlaps.
        if self.encoding.style == EncodingStyle::BitFields {
            for f in &self.formats {
                let total_bits = (f.size_bytes * 8) as u32;
                let mut covered = vec![false; total_bits as usize];
                for fld in &f.fields {
                    let end = fld.offset.checked_add(fld.width).ok_or_else(|| {
                        invalid(format!(
                            "field {}.{} offset+width overflow",
                            f.name, fld.name
                        ))
                    })?;
                    if end > total_bits {
                        return Err(invalid(format!(
                            "field {}.{} extends past format size ({}+{} > {})",
                            f.name, fld.name, fld.offset, fld.width, total_bits
                        )));
                    }
                    for i in fld.offset..end {
                        if covered[i as usize] {
                            return Err(invalid(format!(
                                "field {}.{} overlaps another field at bit {i}",
                                f.name, fld.name
                            )));
                        }
                        covered[i as usize] = true;
                    }
                }
                if let Some((bit, _)) = covered.iter().enumerate().find(|(_, b)| !**b) {
                    return Err(invalid(format!(
                        "format {} has uncovered bit {bit}",
                        f.name
                    )));
                }
            }
        }

        // first_word_bits dispatch: every format declares a discriminator.
        if self.encoding.length_dispatch == LengthDispatch::FirstWordBits {
            for f in &self.formats {
                if f.discriminator.is_none() {
                    return Err(invalid(format!(
                        "format {} missing discriminator (length_dispatch = first_word_bits)",
                        f.name
                    )));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Spec({}, {} opcodes, {} registers, {} formats)",
            self.arch.display_name,
            self.opcodes.len(),
            self.registers.len(),
            self.formats.len(),
        )
    }
}
