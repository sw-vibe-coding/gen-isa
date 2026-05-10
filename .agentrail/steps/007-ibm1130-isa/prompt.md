Step 7 of saga "foundation-and-1130-bringup".

# Goal

Bring up the IBM 1130 ISA description crate -- `sw-comp-history/sw-ibm1130-isa`
-- as the FIRST real per-ISA crate in the new toolchain. Use the
scaffolder to generate the mechanical baseline; hand-fill the
non-mechanical bits; push to `sw-comp-history`.

# Inputs

- `gen-isa/docs/spec-examples/ibm1130.toml` -- spec sample. **Note**:
  illustrative subset (6 opcodes); extend to cover the real opcode set
  before generation.
- `gen-isa/docs/decisions.md` Sec 19 -- real IBM 1130 encoding per
  *Functional Characteristics* (GA26-5881), not the toy encoding from
  `ibm-1130-rs`.
- `~/github/sw-comp-history/ibm-1130-rs/src/cpu/instruction.rs` --
  primary code-reuse reference (~85% portable per `gen-isa/docs/cor24-validation.md`'s
  reuse analysis predecessor in session memory). MIT, user-controlled.
  AddressingMode and BranchCondition enums port nearly directly.
- `softwarewrighter/S1130/src/S1130.SystemObjects/Instructions/OpCodes.cs`
  -- read-only reference (do NOT borrow code per
  `gen-isa/docs/decisions.md` Sec 20). Useful for cross-checking the
  full opcode table.
- *IBM 1130 Functional Characteristics* (GA26-5881) on bitsavers.org for
  authoritative opcode values and reference vectors.

# What to do

1. **Extend `ibm1130.toml`** to cover the real IBM 1130 instruction set.
   The current sample has 6 opcodes; the full set is ~30. Pull values
   from the *Functional Characteristics* manual (or cross-check against
   `S1130/.../OpCodes.cs` and `ibm-1130-rs/src/cpu/instruction.rs`).
   Cover at minimum: LD, LDD, STO, STD, A, AD, S, SD, M, D, AND, OR, EOR,
   SLA, SLT, SRA, SRT, BSC (short and long forms), BSI, MDX, LDX, STX,
   LDS, STS, WAIT. Document any deferred opcodes (XIO, system-mode) in
   the spec file's header comment.

2. **Run the scaffolder** against the extended spec into a tempdir:
   ```
   gen-isa scaffold --slug ibm1130 --display-name "IBM 1130" \
       --type-name Ibm1130 --out <tempdir> \
       --framework-path $HOME/github/sw-langtools \
       --spec docs/spec-examples/ibm1130.toml
   ```

3. **Build + test** the generated `sw-ibm1130-isa` (in the tempdir).
   Roundtrip tests must pass for both short and long forms. Fix any
   spec-format issues that surface (and update `gen-isa/docs/spec-format.md`
   if format extension is needed).

4. **Port `AddressingMode` and `BranchCondition`** from
   `ibm-1130-rs/src/cpu/instruction.rs` into the generated
   `sw-ibm1130-isa`. These are not spec-driven (they're per-ISA semantic
   types). Add them to `lib.rs` or a new `addr_mode.rs` / `branch_cond.rs`
   module.

5. **Add 5 reference vectors** from *Functional Characteristics* to
   `tests/roundtrip.rs`. Each vector: known byte sequence + expected
   decoded `Instruction`. Cite the manual page in a comment.

6. **Exhaustive round-trip**: for each (opcode, tag, disp) combination
   in the short form, and (opcode, tag, indirect, address) in the long
   form, encode and decode and verify equality. Use a reasonable
   bound (e.g. all tags, displacements `i8::MIN..=i8::MAX`, indirect
   on/off, addresses `0..=0xFFFF` sampled).

7. **Move the generated crate from the tempdir to its final home**:
   ```
   mv <tempdir>/sw-ibm1130-isa ~/github/sw-comp-history/sw-ibm1130-isa
   cd ~/github/sw-comp-history/sw-ibm1130-isa
   git init && git add -A && git commit -m "init: sw-ibm1130-isa from gen-isa scaffolder + ibm-1130-rs port"
   gh repo create sw-comp-history/sw-ibm1130-isa --public --source=. --push \
       --description "IBM 1130 ISA description (opcodes, encoding, decoding) for the sw-langtools toolchain"
   ```
   Don't forget to drop the OTHER 4 generated crates (`sw-ibm1130-target`,
   `-codegen`, `-asm`, `-emulator`) in the tempdir; they belong in their
   own steps (8-11).

8. **Update gen-isa/docs/status.md**: crate-state row for
   `sw-ibm1130-isa` -> exists, tests pass, repo URL.

9. **Commit gen-isa changes** (extended `ibm1130.toml`, status update,
   any spec-format extensions).

10. **`agentrail complete`** with:
    - `--summary` describing what got built and reference-vector count.
    - `--reward 1`.
    - `--actions` describing approach.
    - `--next-slug ibm1130-target`.
    - `--next-prompt` defining step 8 (scaffold `sw-ibm1130-target` from
      a fresh scaffolder run; hand-write the ABI in `docs/abi.md`;
      implement `CallingConvention` and `RegisterClasses` (ACC+EXT pair,
      XR1/XR2 allocatable, XR3 reserved as frame pointer); push to
      `sw-comp-history`).

11. STOP.

# Constraints

- ASCII-only in all .md files.
- Real IBM 1130 encoding per Functional Characteristics; not the toy
  encoding from `ibm-1130-rs` (per decisions.md Sec 19).
- No code borrowing from S1130 (per decisions.md Sec 20).
- Generated `sw-ibm1130-isa` must `cargo build / test / clippy -D
  warnings / fmt --check` clean before pushing.
- The other 4 generated crates (`-target`, `-codegen`, `-asm`,
  `-emulator`) stay in the tempdir for now; they get their own
  agentrail steps so each is a clean isolated change.

# Acceptance

- `~/github/sw-comp-history/sw-ibm1130-isa` exists and is pushed to
  GitHub.
- Round-trip tests pass: short and long form curated cases plus 5
  reference vectors plus the exhaustive round-trip sweep.
- `cargo build / test / clippy -D warnings / fmt --check` all pass in
  the new crate.
- `gen-isa/docs/spec-examples/ibm1130.toml` covers the real (not toy)
  opcode set or documents which opcodes are deferred.
- `gen-isa/docs/status.md` updated.

# What NOT to do

- Don't bring up the other crates yet. Steps 8-11 each scaffold and
  flesh out one of `-target` / `-codegen` / `-asm` / `-emulator`.
- Don't attempt a HLASM-grade assembler in this step -- that's step 10.
- Don't attempt instruction execution -- that's step 11.
- Don't push the toy encoding from `ibm-1130-rs`. Re-derive byte
  values from the manual.
