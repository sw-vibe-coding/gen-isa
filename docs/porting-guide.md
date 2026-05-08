# Porting Guide: Bringing Up a New Target ISA

> Status: exploratory draft, not git-tracked. Created 2026-05-08.

This is a contributor-facing **cookbook** for adding support for a new target ISA. Read `architecture.md` and `design.md` first so the layering is clear; this doc is the step-by-step.

The guide assumes the layered scaffolding (`sw-isa-core`, `sw-target-core`, `sw-codegen-core`, `sw-tir`, `sw-tir-opt`) already exists. If you are **building the very first new ISA** (likely IBM 1130), the scaffolding is being designed alongside your work — see §11.

## 0. Strategy: greenfield first, retrofit COR24 last

It is **easier and lower-risk** to bring up new ISAs against a fresh layered design than to retrofit COR24 first. Reasons:

- COR24 is ISA #1; if we use it to validate the abstraction, we'll bake COR24-isms into the trait surface (3-bit register fields, byte-addressed memory, ROM-based encoding, pipelined branch base = PC+4). The abstraction will then strain when ISA #2 doesn't share these properties.
- COR24 keeps working **untouched** while the new layering matures. No regressions to manage.
- Letting 2–3 new ISAs drive the design produces a trait surface forged against real diversity. By the time we retrofit COR24, the design is stable and the retrofit is mechanical.

So the order is:

1. Build minimal scaffolding crates (just enough for the first new ISA).
2. Bring up the first new ISA (IBM 1130 recommended — it scores highest on the abstraction-disruption axis defined in `architecture.md §6.1`; this is *not* the same as "absolute hardest" — S/390 is harder in absolute terms but is the wrong choice for early validation).
3. Bring up additional ISAs (CDP1802, RISC-V I32) — each will push back on the design; iterate.
4. Retrofit COR24 once the trait surface has survived 2–3 ISAs.

This guide is structured around step 2 (your first new ISA) and 3 (subsequent ISAs). For step 4, see `plan.md §8`.

## 1. Prerequisites

Before starting, gather:

- [ ] **ISA reference manual.** Authoritative source for opcodes, encodings, instruction formats, and machine state. For each planned ISA:
  - **IBM 1130:** *IBM 1130 Functional Characteristics* (GA26-5881) and *IBM 1130 Subroutine Library* (manuals are on bitsavers.org).
  - **RCA CDP1802:** *RCA Solid State '76 Databook* and *User Manual for the CDP1802 COSMAC Microprocessor* (MPM-201).
  - **IBM S/370:** *IBM System/370 Principles of Operation* (GA22-7000).
  - **IBM S/390:** *IBM ESA/390 Principles of Operation* (SA22-7201).
  - **RISC-V I32:** *RISC-V Instruction Set Manual, Volume I: Unprivileged ISA*.
- [ ] **Sample programs in machine code.** A handful of known-good byte sequences and their disassembly, for round-trip validation.
- [ ] **An existing emulator** (preferably open-source) for spot-checking your decoder output. SimH covers many historical ISAs; spike covers RISC-V.
- [ ] **Reference implementations.** For IBM 1130 specifically, three locally-cloned projects provide cross-check material. **For code reuse, prefer the two MIT-licensed projects (`ibm-1130-rs`, `demo-ibm-1130-system`) -- the user controls those licenses. Treat `S1130` as read-only reference; do NOT port or copy its code.**
  - [`sw-comp-history/ibm-1130-rs`](https://github.com/sw-comp-history/ibm-1130-rs) (MIT) -- Rust browser-based educational emulator with interactive assembler. **Primary code-reuse reference** for CPU and assembler. CPU in `src/cpu/{instruction,executor,state,assembler}.rs`; memory and register layout in `docs/architecture.md` (4K words, XR1-3 memory-mapped at addresses 1-3, flags C/V/P/Z).
  - [`softwarewrighter/demo-ibm-1130-system`](https://github.com/softwarewrighter/demo-ibm-1130-system) (MIT) -- Rust+WASM **peripheral** simulator (no CPU). **Primary code-reuse reference** for any peripheral support. Disk geometry/timing in `documentation/ibm_1130_disk_i_o_simulator_starter_docs.md`; device traits in `crates/core-sim/src/{disk,card,printer,mux}/`; sample disk/card fixtures in `crates/fixtures/data/`.
  - [`softwarewrighter/S1130`](https://github.com/softwarewrighter/S1130) (fork; no top-level LICENSE -- **read-only reference only, do not copy code**) -- C#/.NET CPU emulator with 335+ unit tests. Useful for cross-checking opcode tables, instruction semantics, and assembler directive coverage. Full opcode table in `src/S1130.SystemObjects/Instructions/OpCodes.cs` (top-5-bit dispatch, 32 opcodes including I/O and double-precision). Per-opcode execution in `Instructions/*.cs`. Two-pass assembler reference in `docs/Assembler.md`. Numeric test vectors derived from the *IBM 1130 Functional Characteristics* manual may be re-derived for our `tests/roundtrip.rs`, but do not copy from `tests/UnitTests.S1130.SystemObjects/`.
- [ ] **Read `architecture.md` and `design.md`.** Especially design.md §1 (tenets) and §7 (per-ISA crate template).

## 2. Crate scaffolding

Create three crates: `sw-{arch}-isa`, `sw-{arch}-target`, `sw-{arch}-codegen`. The `-asm` and `-emulator` crates can wait until ISA + target + codegen exist.

```bash
cd sw-embed/
cargo new --lib sw-ibm1130-isa
cargo new --lib sw-ibm1130-target
cargo new --lib sw-ibm1130-codegen
```

`Cargo.toml` template (per-ISA, identical shape to `sw-cor24-isa`):

```toml
[package]
name = "ibm1130-isa"
version = "0.1.0"
edition = "2024"
description = "IBM 1130 ISA definitions: opcodes, encoding, registers"
license = "MIT"

[features]
default = []
serde = ["dep:serde", "sw-isa-core/serde"]

[dependencies]
sw-isa-core = { path = "../sw-isa-core" }
serde = { version = "1.0", features = ["derive"], optional = true }
```

Copy `LICENSE` and `COPYRIGHT` from `sw-cor24-isa`. Adapt `README.md` to the new ISA but keep the tone — small foundational crate, no runtime, no CLI.

## 3. The five-module skeleton (in `sw-{arch}-isa`)

This is the same shape as `sw-cor24-isa`. Modules go in `src/`:

```
opcode.rs    ← Opcode enum, mnemonic, format-of-opcode
register.rs  ← register type + name table + parser
encode.rs    ← encoding helpers (operands → bytes)
decode.rs    ← decoding helpers (bytes → operands) — split from cor24's combined opcode/encode
branch.rs    ← branch range constants and reachability helpers
format.rs    ← (optional, for ISAs with multi-format encoding) Format enum
lib.rs       ← re-exports + impl sw_isa_core::Architecture
```

Build each in order. Don't move on until the previous one has tests.

### 3.1 `opcode.rs`

A `#[repr(u8)]` (or u16 for 16-bit-opcode ISAs) fieldless enum with one variant per operation:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Opcode {
    Load = 0x18,    // LD ACC <- M
    LoadDbl = 0x19, // LDD ACC,EXT <- M
    // …
    Invalid = 0xFF,
}

impl From<u8> for Opcode { /* exhaustive match */ }

impl Opcode {
    pub fn mnemonic(&self) -> &'static str { /* match */ }
    pub fn format(&self) -> InstructionFormat { /* match */ }
}

impl sw_isa_core::Mnemonic for Opcode {
    fn mnemonic(&self) -> &'static str { Opcode::mnemonic(self) }
}
```

**Pitfalls:**

- Some ISAs reuse the same mnemonic for multiple encodings (e.g. COR24's `Add` is both `AddReg` and `AddImm`; treat them as **separate enum variants** with the same mnemonic string).
- ISAs with extended opcode bits (S/370 `B2xx` ops) need either a wider repr or a tagged enum. Don't try to flatten; readability matters.
- Always include an `Invalid` variant — decoders must produce something for unrecognised bytes.

### 3.2 `register.rs`

Define a register type and a name table. **Don't** assume registers are u8-indexed integers; make the type strongly-typed:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reg { Acc, Ext, Xr1, Xr2, Xr3 }

impl Reg {
    pub fn name(self) -> &'static str { /* match */ }
}

impl sw_isa_core::register::RegisterId for Reg {
    fn index(self) -> u32 { /* … */ }
    fn name(self) -> &'static str { Reg::name(self) }
}

pub fn parse_register(s: &str) -> Option<Reg> { /* … */ }
```

For ISAs with many GPRs (RISC-V's 32, S/370's 16), a newtype around u8 is fine:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Gpr(u8);
impl Gpr { pub const fn new(idx: u8) -> Self { Self(idx & 31) } }
```

### 3.3 `decode.rs`

Define instruction structs and a `decode` function. Variable-length ISAs return `(Instruction, bytes_consumed)`:

```rust
pub struct Short { pub op: Opcode, pub tag: u8, pub disp: i8 }
pub struct Long  { pub op: Opcode, pub tag: u8, pub indirect: bool, pub address: u16 }
pub enum Instruction { Short(Short), Long(Long) }

pub fn decode(bytes: &[u8]) -> Result<(Instruction, usize), DecodeError> {
    if bytes.len() < 2 { return Err(DecodeError::Truncated); }
    let word = u16::from_be_bytes([bytes[0], bytes[1]]); // big-endian for 1130
    let f_bit = (word >> 10) & 1;
    if f_bit == 0 {
        // short form, 1 word
        Ok((Instruction::Short(decode_short(word)), 2))
    } else {
        // long form, 2 words
        if bytes.len() < 4 { return Err(DecodeError::Truncated); }
        let addr = u16::from_be_bytes([bytes[2], bytes[3]]);
        Ok((Instruction::Long(decode_long(word, addr)), 4))
    }
}
```

**Pitfalls:**

- **Endianness.** Big-endian for IBM, CDP1802, S/370, S/390. Little-endian for RISC-V. COR24 is byte-stream (no endianness applies). Be explicit; don't rely on host byte order.
- **Bit-field positions.** Document them in source comments next to the decode logic. Future readers (including you) need them.
- **Sign extension.** Displacement fields are usually signed; explicitly cast `i8`, `i16`, `i32` rather than relying on integer-promotion rules.

### 3.4 `encode.rs`

Inverse of decode. **Round-trip is the test:** `decode(encode(x)) == x`.

```rust
pub fn encode(insn: &Instruction, out: &mut [u8]) -> Result<usize, EncodeError> {
    match insn {
        Instruction::Short(s) => {
            if out.len() < 2 { return Err(EncodeError::BufferTooSmall); }
            let word = ((s.op as u16) << 11)
                     | (0 << 10) // F = 0
                     | ((s.tag as u16) << 8)
                     | ((s.disp as u8 as u16) & 0xFF);
            out[0..2].copy_from_slice(&word.to_be_bytes());
            Ok(2)
        }
        Instruction::Long(l) => { /* … */ }
    }
}
```

**Pitfall — don't copy COR24's table-driven encoding.** COR24 encodes via a `(opcode, ra, rb) → byte` perfect-hash because the hardware decoder is itself a ROM (`dis_rom.v`). **No other ISA has this property.** Encode positionally via bit-field assembly. Save the table-driven approach for ISAs whose hardware *actually* uses an external decode ROM.

### 3.5 `branch.rs`

Range constants + reachability helper. The COR24 module is the right shape:

```rust
pub const BRANCH_OFFSET_MIN: i32 = -128;
pub const BRANCH_OFFSET_MAX: i32 = 127;
pub const MAX_INSTRUCTION_BYTES: usize = 4;
pub const MAX_SHORT_BRANCH_INSTRUCTIONS: usize = 31;

pub fn can_short_branch(from: usize, to: usize) -> bool {
    from.abs_diff(to) <= MAX_SHORT_BRANCH_INSTRUCTIONS
}
```

Adjust constants per ISA. Note `BRANCH_PIPELINE_DELAY = 4` is COR24-specific (3 prefetch + 1 execute); other ISAs typically use PC-relative-to-current-instruction or PC-relative-to-next-instruction. Pick whichever matches the architecture and document it.

### 3.6 `format.rs` (when needed)

For ISAs whose instruction format isn't just "length", define a Format enum:

```rust
// S/370
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Format { RR, RX, RS, RSI, SI, SS, S, E, I, /* … */ }
impl sw_isa_core::format::FormatInfo for Format {
    fn size_bytes(&self) -> usize {
        match self { Format::RR | Format::E => 2, Format::SS => 6, _ => 4 }
    }
}
```

For COR24, CDP1802, IBM 1130 — `sw_isa_core::format::Length` (the prelude enum) is sufficient.

### 3.7 `lib.rs` — adapter to `sw-isa-core`

```rust
pub mod opcode;
pub mod register;
pub mod decode;
pub mod encode;
pub mod branch;

pub use opcode::Opcode;
pub use register::Reg;
pub use decode::{Instruction, Short, Long, decode};
pub use encode::encode;

pub struct Ibm1130;

impl sw_isa_core::Architecture for Ibm1130 {
    type Opcode      = opcode::Opcode;
    type Register    = register::Reg;
    type Instruction = decode::Instruction;
    type Address     = WordAddress;
    type Format      = sw_isa_core::format::Length;

    const NAME: &'static str = "IBM 1130";
    const ENDIAN: sw_isa_core::endian::Endian = sw_isa_core::endian::Endian::Big;
    const ADDRESS_UNIT: sw_isa_core::address::AddressUnit
        = sw_isa_core::address::AddressUnit::Word16;
    const WORD_BITS: u32 = 16;
    const MAX_INSTR_BYTES: usize = 4;
    const MIN_INSTR_BYTES: usize = 2;

    fn decode(bytes: &[u8], _pc: Self::Address) -> _ { decode::decode(bytes) }
    fn encode(insn: &Self::Instruction, out: &mut [u8]) -> _ { encode::encode(insn, out) }
    fn disassemble(insn: &Self::Instruction, w: &mut dyn _) -> _ { /* … */ }
}
```

## 4. Tests for `sw-{arch}-isa`

Three categories:

### 4.1 Round-trip

```rust
// tests/roundtrip.rs
#[test]
fn every_short_form_roundtrips() {
    for op in ALL_OPCODES {
        for tag in 0..4 {
            for disp in i8::MIN..=i8::MAX {
                let insn = Instruction::Short(Short { op, tag, disp });
                let mut buf = [0u8; 4];
                let n = encode(&insn, &mut buf).unwrap();
                let (decoded, m) = decode(&buf[..n]).unwrap();
                assert_eq!(insn, decoded);
                assert_eq!(n, m);
            }
        }
    }
}
```

Run for every legal instruction shape. **Don't ship `-isa` without this passing exhaustively.**

### 4.2 Reference vectors

Take 5–20 known byte sequences from your reference manual or a published assembler's output. Hard-code them as expected values:

```rust
#[test]
fn known_vector_lda_xr1() {
    // From IBM 1130 Functional Characteristics, page 23, example 4
    let bytes = [0xC4, 0x10];
    let (insn, _) = decode(&bytes).unwrap();
    assert_eq!(insn, Instruction::Short(Short { op: Opcode::LoadIndex, tag: 1, disp: 0 }));
}
```

### 4.3 Conformance smoke test (after `sw-isa-core` exists)

```rust
// In sw-isa-core/tests/conformance.rs (parameterised)
fn check_arch<A: Architecture>() {
    assert!(A::MIN_INSTR_BYTES <= A::MAX_INSTR_BYTES);
    assert!(A::MAX_INSTR_BYTES > 0);
    // Round-trip a curated set of bytes for each registered Architecture.
}
```

Add a `#[test]` that calls `check_arch::<Ibm1130>()`.

## 5. `sw-{arch}-target`

Once `-isa` is solid, define the target:

### 5.1 ABI

You're inventing this for historical ISAs. Document it in `docs/abi.md` of the target crate, not just in code. Cover:

- Argument passing: which registers (or memory slots) carry the first N args.
- Return value: which register.
- Caller-saved vs callee-saved register lists.
- Stack: growth direction, alignment, frame layout.
- Frame pointer convention.
- Variadic functions: how they're handled (or "not supported").
- System call interface (if applicable).

### 5.2 Type widths

```rust
fn type_width(ty: PrimType) -> usize {
    match ty {
        PrimType::I8 | PrimType::U8 => 1,
        PrimType::I16 | PrimType::U16 => 2,
        PrimType::I32 | PrimType::U32 => 4,
        PrimType::I64 | PrimType::U64 => 8,
        PrimType::Ptr => 2, // 1130: 16-bit word address
        PrimType::Bool => 1,
    }
}
```

Adjust per ISA. RISC-V I32: pointer = 4 bytes. S/370: pointer = 4 bytes. S/390 (ESAME): pointer = 8 bytes. CDP1802: pointer = 2 bytes.

### 5.3 Register classes

```rust
impl RegisterClasses<Ibm1130> for Ibm1130Target {
    fn gpr() -> &'static [Reg] { &[Reg::Acc, Reg::Xr1, Reg::Xr2] } // ACC + 2 XRs
    fn reserved() -> &'static [Reg] { &[Reg::Xr3] } // FP, never allocated
    fn fixed_pairs() -> &'static [(Reg, Reg)] { &[(Reg::Acc, Reg::Ext)] } // 32-bit ops
}
```

For 1130 specifically, the allocator will need to be very spill-friendly. See `design.md §11 D5`.

## 6. `sw-{arch}-codegen`

Lower IR ops to native instructions. Organise per-IR-op:

```
src/
  lib.rs
  select.rs        ← top-level selection dispatch
  lower/
    arith.rs       ← Add, Sub, Mul, Div…
    mem.rs         ← Load, Store, Alloca
    cmp_branch.rs  ← Icmp, CondBranch, Branch
    call.rs        ← Call, Return, prologue/epilogue
    intrinsic.rs   ← target-specific lowering
  peephole.rs
```

Selection patterns are hand-written. For 12 languages × 5 ISAs, hand-writing is manageable. Don't reach for table-driven matchers (Burg, etc.) until you have 3+ ISAs and the patterns are stable.

### Snapshot tests

```rust
// tests/snapshot.rs
#[test]
fn lower_simple_add() {
    let ir = parse_ir("define i16 @f(i16 %a, i16 %b) { %r = add %a, %b; ret %r }");
    let asm = emit_asm(&ir);
    insta::assert_snapshot!(asm, @r###"
        f:
          STX  XR1, 4(FP)
          LD   8(FP)
          A    10(FP)
          STO  result
          LDX  XR1, 4(FP)
          BSC  0(FP)
    "###);
}
```

Use `insta` or similar. **Goldens are the test;** when the codegen changes, review the diff.

## 7. `sw-{arch}-asm` and `sw-{arch}-emulator`

Out of scope of this guide but worth noting:

- **Assembler:** parses target asm, calls `sw-{arch}-isa::encode`. Most of the work is the parser; encoding is a one-liner per instruction.
- **Emulator:** executes bytes. This is where execution semantics live (the part `-isa` deliberately omits). For the first end-to-end test, a tiny emulator that handles only the ops the pilot frontend uses is fine.

## 8. End-to-end smoke test

After `-isa`, `-target`, `-codegen`, `-asm`, `-emulator` exist:

```
$ swbasic -O hello.bas -t ibm1130 -o hello.bin
$ sw-ibm1130-emulator hello.bin
HELLO, WORLD
```

If this passes, the ISA is **alpha-quality**. To reach beta, expand snapshot tests to cover every IR op the pilot frontend can emit, plus all reference vectors from the manual.

## 9. What to copy from `sw-cor24-isa` and what NOT to copy

This is the cherry-pick guide.

### COPY

- **Module organisation.** Five files: `opcode.rs`, `register.rs`, `branch.rs`, `encode.rs`, `lib.rs` (plus `decode.rs`, `format.rs` as needed). Same names, same responsibilities.
- **Public API style.** Small free functions, fieldless enums with `From<u8>`, `Display`, `mnemonic()`, `format()`. Idiomatic and predictable.
- **`Cargo.toml` shape.** Same package metadata layout, same optional `serde` feature gate.
- **Documentation tone.** Short doc comments, no marketing prose, code-comment-style explanations of bit fields.
- **`branch.rs` structure.** Constants + `can_short_branch` helper. Adjust the constants; keep the shape.
- **Test patterns.** `#[test] fn test_encode_X` per opcode; round-trip helper.
- **License + COPYRIGHT files.**

### DO NOT COPY

- **The encoding table mechanism.** `sw-cor24-isa::encode::encode_instruction` is a 300-line `match (opcode, ra, rb)` table because COR24's hardware decoder is a ROM (`dis_rom.v`). For every other planned ISA, instructions encode positionally via bit-field assembly. Use `(value << shift) | mask` arithmetic, not exhaustive tables.
- **The 5-bit-opcode-via-decoded-rom convention.** COR24's `Opcode::from_decoded` reads bits 10..6 of a 12-bit ROM output. This is COR24-specific. Other ISAs read opcode bits directly from the instruction word.
- **The `(ra=7, rb=7) = no-op-operands` convention.** COR24 uses unused register fields to fill encoding slots. Other ISAs encode these positions semantically.
- **The 3-bit register field assumption.** COR24 has 8 registers. Don't generalise the bit width — declare the register type per ISA.
- **The `BRANCH_PIPELINE_DELAY = 4` and `branch_base = PC + 4` semantics.** This reflects COR24's 3-prefetch + 1-execute pipeline. Most ISAs use PC-relative-to-next-instruction or PC-relative-to-current-instruction. Match your ISA's actual behaviour.
- **Byte-addressed memory assumptions in any helper that touches addresses.** IBM 1130 is word-addressed; functions like "advance address by 1 instruction" must respect `AddressUnit`.
- **The `MAX_INSTRUCTION_BYTES = 4` constant value.** S/370 has 6-byte instructions; CDP1802 has 1–3 byte instructions. Set per ISA.

### LOOK AT BUT REIMPLEMENT

- **`opcode::Opcode::format()`** — the *idea* of mapping opcodes to a format enum is universal; the contents are per-ISA.
- **Test fixture style** — `tests/encode_decode.rs` patterns transfer; the actual vectors don't.
- **`reg_name` and `parse_register`** — same concept, different table.

## 10. Common pitfalls per ISA family

| ISA family | Pitfall | Mitigation |
| ---------- | ------- | ---------- |
| Word-addressed (IBM 1130) | Forgetting that `address + 1` advances by 2 bytes | Use the `AddressType::step` API; never hand-arithmetic byte counts |
| Accumulator-only (1130, CDP1802) | Naive register allocator runs out of registers immediately | Set `RegisterClasses::gpr()` correctly; expect heavy spilling; consider custom allocator |
| Indirect-via-register-pair (1130 Long form, S/370 RX) | Forgetting that the register operand is a *pointer*, not a value | Decode and encode operand semantics carefully; document indirect bits |
| Multi-format (S/370) | Conflating format with opcode | Keep `Format` orthogonal to `Opcode`; format determines size and operand layout |
| Variable-length (most non-RISC-V) | Off-by-one on bytes consumed | Always return bytes consumed from `decode`; assert in tests |
| Fixed-length 32-bit (RISC-V I32) | Confusion between u32 instruction word and the 4 bytes | Pick one (we use bytes throughout) and stick with it |
| Big-endian (IBM, CDP1802, S/370) | Host byte order leakage | Use `u16::from_be_bytes` / `to_be_bytes`; never transmute |
| Pipelined branch base (COR24-style) | Off-by-N on branch offset semantics | Document the branch base in `branch.rs`; cite the manual |

## 11. Special case: bringing up the *first* new ISA

If you are bringing up the first new ISA (the trait surfaces in `sw-isa-core` etc. don't exist yet), expect to:

- **Iterate on the trait surface as you go.** When you find that `Architecture::decode`'s signature doesn't accommodate a 1130-specific case, change the trait. The 1130 implementation IS the reference.
- **Delete and rewrite freely.** The first ISA's code is throwaway-ish until the design stabilises. Don't optimise it.
- **Push back on premature abstractions in `sw-isa-core` / `sw-target-core`.** If a method exists "for future ISAs we haven't built yet", remove it. Drive abstraction strictly from concrete need.
- **Document open questions in `status.md`.** Anything you punted goes there for ISA #2 to revisit.

After ISA #2 lands, the trait surface should stabilise. After ISA #3 lands, retrofitting COR24 is the next milestone.

## 12. Validation checklist (per ISA)

Before declaring an ISA "done":

- [ ] Every opcode in the reference manual has a variant in `Opcode`.
- [ ] Round-trip test passes for every legal instruction shape (exhaustive where feasible).
- [ ] At least 5 reference vectors from the manual pass `decode` and produce identical `encode` output.
- [ ] `impl Architecture` compiles and conformance smoke test passes.
- [ ] `-target` exists with documented ABI in `docs/abi.md`.
- [ ] `-codegen` exists with snapshot tests for every IR op the pilot frontend emits.
- [ ] `-asm` round-trips: `asm → bytes → disasm → asm` (textually equivalent).
- [ ] `-emulator` runs the pilot frontend's "hello world" successfully.
- [ ] `status.md` updated.
- [ ] Postmortem added (if anything in the layering had to change to accommodate this ISA).

## 13. Estimated effort (rough)

For the **first** new ISA, expect 4–8 weeks of focused work — most of it spent iterating on the layered scaffolding alongside the ISA implementation.

For **subsequent** ISAs, once the scaffolding is stable:

| Phase | RISC-V I32 | CDP1802 | IBM 1130 | S/370 | S/390 |
| ----- | ---------- | ------- | -------- | ----- | ----- |
| `-isa`     | 1 wk     | 1 wk    | 1.5 wk   | 3 wk  | 1 wk (extends S/370) |
| `-target`  | 0.5 wk   | 0.5 wk  | 0.5 wk   | 1 wk  | 0.5 wk |
| `-codegen` | 2 wk     | 2 wk    | 3 wk     | 4 wk  | 2 wk  |
| `-asm`     | 1 wk     | 1 wk    | 1 wk     | 2 wk  | 1 wk  |
| `-emulator`| 2 wk     | 1.5 wk  | 2 wk     | 4 wk+ | 2 wk+ |
| **Total**  | ~6.5 wk  | ~6 wk   | ~8 wk    | ~14 wk| ~6.5 wk |

These are estimates for one experienced contributor. Actual durations vary with familiarity and how much of the toolchain reuse is exercised end-to-end.

## 14. After the ISA is up

- Update `status.md`'s conformance matrix.
- Add the ISA to the canonical doc-site (whenever that exists).
- Backfill any frontend support — once `-codegen` works for one frontend, the others come along automatically *if they emit `sw-tir`*. If not, the frontend refactor (plan.md Phase 4) is on the critical path.
