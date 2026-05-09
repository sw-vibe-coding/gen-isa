# Scaffolder Validation Against Existing `sw-cor24-isa`

> Status: completed 2026-05-09 in saga step `scaffolder-validation`. Acts
> as the gate before steps 7-11 (real IBM 1130 bring-up via the
> scaffolder). Verdict at the bottom.

## Method

1. Ran `gen-isa scaffold --slug cor24 --display-name "COR24" --type-name
   Cor24 --out <tempdir> --framework-path $HOME/github/sw-langtools
   --spec docs/spec-examples/cor24.toml`. Generated quintet at
   `<tempdir>/sw-cor24-{isa,target,codegen,asm,emulator}`.
2. `cargo build` and `cargo test` succeed in the generated `sw-cor24-isa`.
   Round-trip test is `#[ignore]`'d (rom_table mode opts out of generated
   encode/decode, as designed).
3. Per-module compared the generated `sw-cor24-isa/src/*.rs` against the
   existing `~/github/sw-embed/sw-cor24-isa/src/*.rs`.

## Per-module classification

### `opcode.rs`

**Match** for everything the spec sample covers. Same opcode names, same
discriminant values, mnemonics overload correctly (`AddReg` and `AddImm`
both -> `"add"`; `SubSp` -> `"sub"`).

**Gap (intentional)**: spec sample has 10 opcodes; existing crate has 32
(0x00-0x1F) plus `Invalid = 0xFF`. Per the spec sample's own header
comment, this is a representative subset for format-format-coverage, not
the full ISA. **Action**: extend `cor24.toml` with the remaining 22
opcodes during the COR24 retrofit (saga phase 6, separate from this
saga); not a generator gap.

**Design difference (acceptable)**: existing has `pub enum
InstructionFormat { SingleByte, TwoBytes, FourBytes }` plus
`Opcode::format() -> InstructionFormat`. Generated has `Opcode::formats()
-> &'static [&'static str]` returning format-name slices. The generated
shape is more flexible (supports multi-format opcodes like 1130's
short/long Load); the existing shape is more type-safe but COR24-specific.
**Action**: accept the divergence. The COR24 retrofit can choose to
hand-add an `InstructionFormat` enum on top of the generated `formats()`
if the type-safety is wanted; not blocking for the IBM 1130 bring-up
that this saga targets.

**Missing**: existing `DecodedInstruction { opcode, ra, rb }` plus
`from_decoded(u16)` for the ROM bit-fields. The generator does not emit
these because they are COR24-specific (ROM decoder; no other ISA shares
the (5+3+3=11) decoded-bit shape). **Action**: hand-supply during the
retrofit. Not a generator gap; it's a per-ISA quirk that belongs in the
hand-written portion (matches the rom_table opt-out spirit).

**Verdict: Match** (with documented intentional gaps).

### `register.rs`

**Match in semantics**: same 8 register names (`r0`, `r1`, `r2`, `fp`,
`sp`, `z`, `r6`, `r7`), same indices.

**Design difference (significant)**: existing uses `pub const REG_NAMES:
[&str; 8]` array with free functions `reg_name(u8) -> &'static str` and
`parse_register(&str) -> Option<u8>`. Generated emits `pub enum Reg`
with `name()` / `class()` methods plus `impl
sw_isa_core::register::RegisterId`.

**Why it diverges**: the existing API is pragmatic for ROM-decoder
bit-fiddling that already works in raw `u8`. The generated API is
type-safer and integrates with the framework via `RegisterId`. Both are
valid; one is current legacy, one is forward-looking.

**Action**: COR24 retrofit (phase 6) will pick. Either keep the existing
const-array API and add a thin `Reg` enum wrapper that implements
`RegisterId`, or switch fully to the enum. Not blocking; not a generator
gap.

**Spec gap**: existing `parse_register` accepts aliases (`"c"` for `z`,
`"iv"` for `r6`, `"ir"` for `r7`). The TOML spec format does not model
aliases. **Action**: defer. If a future ISA needs aliases, extend the
spec format with `aliases = ["x", "y"]` on register entries. For now,
the COR24 retrofit can hand-add the alias arms.

**Verdict: Match** (semantic), **Design choice deferred** (API shape).

### `branch.rs`

**Match**: same constants -- `BRANCH_OFFSET_MIN = -128`, `BRANCH_OFFSET_MAX
= 127`, `MAX_INSTRUCTION_BYTES = 4`, `MAX_SHORT_BRANCH_INSTRUCTIONS =
31`, pipeline delay = 4.

**Naming difference (cosmetic)**: existing exports
`BRANCH_PIPELINE_DELAY`; generated emits `PIPELINE_DELAY_BYTES`. Same
value, different name. **Action**: accept; the `_BYTES` suffix is more
descriptive. Retrofit can re-export under both names if needed for
backward compatibility.

**Bug fix during validation**: the generated `can_short_branch(from, to)`
originally used signed subtraction (`to as isize - from as isize`),
which returns `false` if `from > to` even within range -- subtly wrong.
The existing implementation uses `from.abs_diff(to) <=
MAX_SHORT_BRANCH_INSTRUCTIONS`, which is symmetric and correct. The
generator was patched in this step to match.

**Verdict: Match** (after the abs_diff fix).

### `lib.rs`

**Expected divergence**: existing `lib.rs` has *no* `impl
sw_isa_core::Architecture` (that retrofit is phase 6 in
[`plan.md`](plan.md), explicitly out of this saga's scope). Generated
`lib.rs` does emit `impl Architecture for Cor24` with all the required
constants and methods. Both are correct for their respective phases.

**Architecture constants (generated)**: `NAME = "COR24"`, `ENDIAN = Big`,
`ADDRESS_UNIT = Byte`, `WORD_BITS = 24`, `MIN_INSTR_BYTES = 1`,
`MAX_INSTR_BYTES = 4`. Match the spec and the project's understanding of
COR24.

**Stub Instruction**: generator emits `pub struct Instruction;` (unit
struct) for rom_table mode with a comment to hand-fill. The existing
crate uses `DecodedInstruction { opcode, ra, rb }`; the retrofit would
replace the generated stub with that shape during phase 6.

**Verdict: Match** (forward-looking; current state is pre-retrofit).

### `encode.rs` / `decode.rs`

**Correctly opts out of generation**: with `encoding_style = "rom_table"`
in the spec, the generator emits stubs that return
`Err(EncodeError::InvalidOperands)` / `Err(DecodeError::Invalid)` and
include a TODO comment pointing at decisions.md Sec 14. The existing
`encode.rs` has 409 lines of hand-written per-opcode match-tree
encoding; that whole body is what would replace the generated stub
during the COR24 retrofit.

**Verdict: Match** (rom_table opt-out works as designed; the
hand-supplied portion is preserved as a separate concern).

## Bottom-line verdict

**The scaffolder is faithful enough to proceed to step 7 (IBM 1130
bring-up).** All gaps surfaced are one of:

- intentional spec subset (covered by retrofit-extends-spec, not a
  generator change),
- design choices where both shapes are valid (retrofit picks),
- COR24-specific quirks that legitimately belong in the hand-supplied
  rom_table portion (DecodedInstruction, encode match-tree, register
  aliases),
- minor cosmetic differences (constant naming),
- one real bug (can_short_branch directional vs symmetric) which has
  been patched in `gen-isa/src/emit.rs` and tested.

For IBM 1130 (steps 7-11) the path is much easier than COR24's retrofit
will be: 1130 uses `bit_fields` encoding, so the generator emits real
encode/decode bodies (no rom_table stubs). And the framework crate trait
surfaces accommodate everything 1130 needs (per the smoke tests
already passing in `sw-langtools/sw-isa-core`).

## Spec extensions deferred (non-blocking)

These are not needed for IBM 1130 bring-up but worth tracking for later:

- **Register aliases** (`aliases = ["x", "y"]` on `[[register]]` entries):
  for COR24 retrofit's `"z"`/`"c"` aliases. Add when the COR24 retrofit
  saga starts.
- **InstructionFormat enum support**: optional spec field
  `format_enum_name = "InstructionFormat"` that triggers an additional
  enum emission alongside `formats() -> &[&str]`. Add when a target
  needs the extra type-safety.
- **Custom mnemonic-overload model**: the existing `Sub` and `SubSp`
  share mnemonic `"sub"`. The generator handles this correctly because
  the spec already declares `mnemonic = "sub"` on each. No extension
  needed; documenting that overload is supported.

## Files changed by this validation step

- `gen-isa/src/emit.rs`: `branch_rs()` updated to emit `abs_diff`-based
  `can_short_branch`.
- `gen-isa/docs/cor24-validation.md`: this file.
- `gen-isa/docs/status.md`: recent-changes entry.
