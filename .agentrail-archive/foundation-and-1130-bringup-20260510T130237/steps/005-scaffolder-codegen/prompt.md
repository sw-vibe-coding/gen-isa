Step 5 of saga "foundation-and-1130-bringup".

# Goal

Extend the scaffolder (built in step 4) to **parse a TOML ISA spec** and
emit real Rust code for the mechanical bits of the `-isa` crate:
`opcode.rs`, `register.rs`, `encode.rs`, `decode.rs`, `branch.rs`, and the
`Architecture` impl in `lib.rs`. Other crates in the quintet (`-target`,
`-codegen`, `-asm`, `-emulator`) still get the same skeleton templates as
step 4 -- their content is judgement-based and stays hand-written.

# Inputs

- The TOML spec format documented in `gen-isa/docs/spec-format.md` and
  the worked examples in `gen-isa/docs/spec-examples/{cor24,ibm1130}.toml`.
- The existing scaffolder code in `gen-isa/src/`.
- The framework trait surfaces in `~/github/sw-langtools/sw-isa-core/`
  (so generated code uses the right names / signatures).

# Output

A new CLI flag `--spec <PATH>` on the `scaffold` subcommand. When
provided, the scaffolder parses the TOML and emits:

1. **`src/opcode.rs`** -- `#[repr(u8)]` (or `u16` if needed) `Opcode`
   enum populated from the spec's `[[opcode]]` entries. Includes
   `mnemonic()` method, `format()` method (returning the format-name
   that ties to `[[format]]` entries), `From<u8>` (for valid bytes),
   and `impl sw_isa_core::Mnemonic`.

2. **`src/register.rs`** -- `Reg` enum (or newtype around u8 if
   `[[register_bank]]` is present in the spec) populated from
   `[[register]]` entries. `name()` method. `parse_register()` free
   function. `impl sw_isa_core::register::RegisterId`.

3. **`src/encode.rs`** -- For specs with `encoding_style = "bit_fields"`,
   emit per-format `encode_<format_name>()` helpers that assemble bytes
   from operands using the format's `[[format.field]]` declarations.
   For `encoding_style = "rom_table"`, emit a placeholder noting that
   encoding is hand-supplied (matches the COR24 case).

4. **`src/decode.rs`** -- Inverse of encode. Reads bytes -> Instruction
   variants per format. Uses the `discriminator` from `[[format]]` entries
   when `length_dispatch = "first_word_bits"`; uses the opcode-to-format
   map when `length_dispatch = "per_opcode"`.

5. **`src/branch.rs`** -- Constants from `[branch]`: `BRANCH_OFFSET_MIN`,
   `BRANCH_OFFSET_MAX`, `MAX_INSTRUCTION_BYTES`, plus `can_short_branch()`
   helper.

6. **`src/lib.rs`** -- Re-exports + `impl sw_isa_core::Architecture` for
   the marker type. Method bodies route to the per-module helpers.

7. **`tests/roundtrip.rs`** -- Per-format exhaustive round-trip test
   skeleton. The body asserts `decode(encode(x)) == x` over a small
   curated set of instructions; the test author can extend.

For specs without a `--spec` flag (i.e. step-4 mode), keep the empty-stub
behaviour from step 4 unchanged.

# Spec parsing

- Use the `toml` crate (latest stable).
- Parse into typed structs in `gen-isa/src/spec.rs` (new module).
- Validate the spec on parse: every `[[opcode].format]` references an
  existing `[[format]]`; every `[[fixed_pair]]` references existing
  registers; opcode `name`s and `value`s unique within their format;
  for `bit_fields`, format fields cover exactly `size_bytes * 8` bits
  with no gaps / overlaps; for `first_word_bits` length dispatch, every
  format declares a `discriminator`. See `spec-format.md` Sec
  "Validation rules" for the full list.
- Surface validation failures as a typed error variant
  (`ScaffoldError::InvalidSpec(String)`).

# Constraints

- Generated `-isa` crate must `cargo build` against the real
  `sw-langtools/sw-isa-core` (path-dep). Verify by scaffolding both
  COR24 and IBM 1130 specs to a tempdir with `--framework-path
  $HOME/github/sw-langtools` and running `cargo build`.
- Generated `-isa` crate must `cargo test` cleanly: the
  `tests/roundtrip.rs` skeleton compiles and the bundled
  curated-set test passes for at least the IBM 1130 sample
  (curated set may be small; full round-trip exhaustiveness is
  step 7's concern).
- Generated code must be `cargo fmt`-clean and `cargo clippy
  --all-targets --all-features -- -D warnings`-clean. Either
  the emitted code is fmt-correct out of the box, or the
  scaffolder runs `rustfmt` on its output before writing.
- `gen-isa`'s own clippy/fmt/test bar from step 4 still applies.
- Tests in `gen-isa` must cover: valid spec round-trip, invalid
  specs rejected (each validation rule), generated code shape
  (snapshot tests are fine for the easy bits).

# What to do this step

1. Read `gen-isa/docs/spec-format.md` and the two
   `spec-examples/*.toml` carefully -- they are the contract.
2. Add a `[dependencies] toml = "0.8"` (or current) to
   `gen-isa/Cargo.toml`.
3. Author `gen-isa/src/spec.rs` -- typed structs + parse +
   validation.
4. Extend `templates.rs` with spec-driven helpers for opcode.rs /
   register.rs / encode.rs / decode.rs / branch.rs / lib.rs / 
   tests/roundtrip.rs. Keep the step-4 empty-stub fallback for
   when no spec is provided.
5. Wire `--spec` into the CLI in `main.rs`.
6. Tests:
   - Spec parse: positive cases (cor24, ibm1130).
   - Spec validation: rejects each invalid case (overlapping
     bit fields, dangling format reference, etc.).
   - Snapshot: generated `opcode.rs` for cor24 contains
     `enum Opcode { AddReg = 0x00, AddImm = 0x01, ... }`.
   - Integration: scaffold ibm1130 to tempdir, `cargo build`
     succeeds.
7. Verify with `cargo build/test/clippy -D warnings/fmt --check`.
8. Run end-to-end: scaffold COR24 and IBM 1130 to tempdirs with
   `--framework-path $HOME/github/sw-langtools`; verify both
   compile and the round-trip test passes.
9. Update `gen-isa/docs/status.md` with what changed.
10. Commit, then `agentrail complete` with:
    - `--summary` describing what got built and acceptance evidence.
    - `--reward 1`.
    - `--actions` describing approach.
    - `--next-slug scaffolder-validation`.
    - `--next-prompt` defining step 6 (cross-check the smart
      scaffolder by regenerating `sw-cor24-isa` from the COR24
      spec into a tempdir; diff the generated module files
      against the existing `sw-embed/sw-cor24-isa` modules; the
      generated implementation should match the existing one
      modulo formatting and irrelevant differences).
11. STOP.

# What NOT to do

- Don't generate code for `-target`, `-codegen`, `-asm`,
  `-emulator` from the spec -- they remain skeletons.
- Don't pre-generate codegen patterns or instruction-selection
  rules -- those live in saga step 9.
- Don't try to lift the existing `sw-cor24-isa` source into the
  generator. The point of step 6 is to validate that the
  *generator* can produce equivalent code from the spec; it does
  this by regenerating, not copying.
- Don't push generated crates anywhere in this step.
