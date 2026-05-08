Step 2 of saga "foundation-and-1130-bringup".

# Goal

Design the TOML spec format that the hybrid scaffolder (in this repo) will
consume to drive the mechanical parts of per-ISA `-isa` crate generation.

Document the format in `docs/spec-format.md`. Provide sample specs in
`docs/spec-examples/` for COR24 (reference cross-check, lifted from the
existing `sw-cor24-isa`) and IBM 1130 (target).

**Do NOT build the parser or generator in this step.** This is design and
worked-example only. The next step (`framework-skeletons`) writes the
framework crates; the step after (`scaffolder-mvp`) starts the generator.

# Required spec coverage

The format must let a human author specify, for one ISA:

- Identity: ISA name (display string), `Architecture::NAME` constant,
  Rust type name (e.g. `Ibm1130`), crate-name slug (`ibm1130`).
- Memory model: address unit (`Byte`, `Word16`, `Word24`, `Word32`),
  endianness (`Big`, `Little`, `ByteStream`), word bits, min/max
  instruction bytes.
- Registers: per-register name + class (e.g. `Acc`, `Ext`, `Xr`,
  `GprBank`) + index. Should support both small fixed enums (1130's
  ACC/EXT/XR1-3) and bulk newtype-around-u8 banks (RISC-V's 32 GPRs).
- Opcodes: per-opcode mnemonic, numeric value, format reference,
  optional semantic role tag (e.g. `arith.add`, `mem.load`, `branch.cond`,
  `call`, `ret`) for codegen-pattern hooks.
- Instruction formats: per format, the bit-field layout of operands
  (offset + width + signed/unsigned + role like `opcode|reg|disp|imm`).
  Length-only ISAs use the predefined `Length` format from `sw-isa-core`;
  multi-format ISAs (S/370) define their own enum.
- Branch ranges: short-branch min/max offsets, max instruction bytes,
  pipeline-delay or branch-base semantics if relevant.

# Format choices to make

- One spec file per ISA, or shared spec with multiple ISA tables? **Choose one.**
- Field naming: snake_case TOML keys throughout. Confirm.
- How to express register pairs (1130 ACC+EXT) in the register section.
- How to express "reserved" / "pseudo" registers (PC, SP) that the
  scaffolder should NOT emit as allocatable.
- How to mark opcodes that share a mnemonic (COR24's `AddReg` and
  `AddImm` both have mnemonic `add`).
- How to handle instruction subforms (1130's Short vs Long). Options:
  separate format entries with shared opcode prefix, or nested `[short]` /
  `[long]` blocks under one opcode.

Pick a side for each, document the rationale briefly in `spec-format.md`.

# Sample specs

`docs/spec-examples/cor24.toml` -- enough of the COR24 spec to demonstrate
that the format covers an existing implementation. Reference the existing
`sw-cor24-isa` modules to extract opcodes/registers; do not exhaustively
copy every opcode -- a representative subset (one of each format flavour,
plus a few normal ops) is sufficient for the worked example.

`docs/spec-examples/ibm1130.toml` -- enough of the IBM 1130 spec to
demonstrate the format covers the upcoming target. Pull opcode bytes from
`docs/porting-guide.md` Sec 8 and any IBM 1130 reference you have. Cover:

- Short and Long form opcodes.
- ACC + EXT pair declaration.
- XR1, XR2, XR3 (with XR3 reserved as future frame pointer).
- F-bit format dispatch.
- Branch range constants.
- Five opcodes minimum (e.g. LD, A, STO, BSC, M).

# Constraints

- `markdown-checker -f "docs/**/*.md"` must pass (ASCII-only).
- TOML files: ASCII-only, parseable by the standard `toml` crate.
- No code yet -- no `gen-isa/src/`, no `Cargo.toml` for a binary.

# Acceptance

- `docs/spec-format.md` covers all required spec coverage above and
  documents every format choice with a one-sentence rationale.
- `docs/spec-examples/cor24.toml` and `docs/spec-examples/ibm1130.toml`
  exist and parse as valid TOML (`python3 -c 'import tomllib; tomllib.load(open("docs/spec-examples/ibm1130.toml","rb"))'`
  or equivalent).
- markdown-checker passes on the new docs.

# What to do this step

1. Read `docs/decisions.md` (committed in step 1) for locked-in choices.
2. Read existing `sw-cor24-isa` module shapes referenced from
   `docs/porting-guide.md` Sec 3, Sec 9.
3. Draft `docs/spec-format.md`. Then draft the two sample specs.
4. Validate: parse both TOML files; run markdown-checker on docs.
5. `git add docs/spec-format.md docs/spec-examples/`
6. `git commit -m "docs: define ISA TOML spec format with COR24 and IBM 1130 samples"`
7. `agentrail complete` with:
   - `--summary` describing the format and why each format choice was made.
   - `--reward 1` (or `-1` plus `--failure-mode "..."` if blocked).
   - `--actions` describing what was authored.
   - `--next-slug framework-skeletons`
   - `--next-prompt` defining step 3 (write the 5 framework crate
     skeletons in `sw-langtools`: `sw-isa-core`, `sw-target-core`,
     `sw-tir`, `sw-tir-opt`, `sw-codegen-core`. Trait surfaces from
     `docs/design.md` Sec 2-6. Each crate is a separate sibling repo
     pushed to the `sw-langtools` GitHub org via `gh repo create`. Each
     compiles standalone; cross-deps use `path = "../<crate>"`.)
8. STOP. Do not begin step 3.
