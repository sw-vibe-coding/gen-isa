Step 6 of saga "foundation-and-1130-bringup".

# Goal

Validate the smart scaffolder by cross-checking it against the existing
`sw-embed/sw-cor24-isa`. The scaffolder is "faithful" if it can regenerate
COR24's `-isa` crate from the COR24 spec and produce code that matches the
existing implementation modulo formatting and irrelevant differences.

This is the **last** step before steps 7-11 use the scaffolder for real
(IBM 1130 bring-up).

# What to do

1. Read the existing `~/github/sw-embed/sw-cor24-isa/src/{opcode,register,branch,encode,lib}.rs`
   carefully. Compare them mentally against `gen-isa/docs/spec-examples/cor24.toml`.

2. Identify gaps in the spec or the generator. Likely candidates:
   - The COR24 spec sample is a 10-opcode subset; the real `sw-cor24-isa`
     has 32. Either extend the spec sample to cover all 32 opcodes
     (preferred) or document which opcodes are deliberately omitted.
   - COR24 uses `encoding_style = "rom_table"`, so the generator's
     encode/decode stubs are correct -- but the existing `sw-cor24-isa`
     does have a real encode (the per-opcode match-tree). Confirm that
     the generated skeleton correctly *opts out* of generating that and
     leaves a place for the human to plug in the existing encoder.
   - COR24 may have ISA properties not yet expressible in the spec
     format (3-bit register field convention, pipeline-delay branch base,
     ROM-based decoding, etc.). Inventory them and decide: extend the
     spec format, or document as out-of-scope for the generator.

3. Run the scaffolder against the COR24 spec into a tempdir:
   ```
   gen-isa scaffold --slug cor24 --display-name "COR24" --type-name Cor24 \
       --out /tmp/cor24-regen --framework-path $HOME/github/sw-langtools \
       --spec docs/spec-examples/cor24.toml
   ```
   Verify the generated `sw-cor24-isa` (in the tempdir, not in sw-embed)
   compiles and the (ignored) roundtrip test compiles.

4. **Cross-check shape, not content**. For each module, diff the *generated*
   file's structure against the *existing* `sw-embed/sw-cor24-isa`:
   - `opcode.rs`: same Opcode enum variants? same mnemonics? same value
     bytes? Order may differ; that's OK. Variants the spec didn't cover
     are documented gaps.
   - `register.rs`: same Reg names? same indices? Note the existing impl
     uses a `pub const REG_NAMES: [&str; 8]` array rather than an enum;
     decide whether that's worth supporting in the generator or whether
     the enum approach is acceptable for the COR24 retrofit.
   - `branch.rs`: same constant values?
   - `lib.rs`: does the generated `Architecture` impl declare the same
     constants (`NAME`, `ENDIAN`, `ADDRESS_UNIT`, `WORD_BITS`,
     `MIN/MAX_INSTR_BYTES`)?

5. Capture findings in `gen-isa/docs/cor24-validation.md` with:
   - One section per module ("opcode", "register", "branch", "lib",
     "encode/decode").
   - "Match" / "Gap" / "Spec extension needed" classification.
   - For each Gap, a sentence on whether to fix in spec, fix in generator,
     or accept (with rationale).
   - A bottom-line "scaffolder is / is not faithful enough" verdict.

6. If the validation reveals real gaps that block the scaffolder from
   handling COR24:
   - Either extend the spec format and the generator (if low-risk),
     OR document the gaps as known scope for a future saga step (if
     high-risk).
   - Update `gen-isa/docs/spec-format.md` and `cor24.toml` if extended.

7. Update `docs/status.md` (Recent changes + crate-state hint).

8. `git add` and commit.

9. `agentrail complete` with:
   - `--summary` describing the validation outcome.
   - `--reward 1` if the scaffolder is faithful enough to proceed,
     `-1 --failure-mode "scaffolder-not-faithful"` plus a description if
     blockers were found.
   - `--actions` describing approach.
   - `--next-slug ibm1130-isa`.
   - `--next-prompt` defining step 7 (scaffold `sw-ibm1130-isa` into
     `~/github/sw-comp-history/`, port the AddressingMode and
     BranchCondition enums from `sw-comp-history/ibm-1130-rs/src/cpu/instruction.rs`,
     extend with real IBM 1130 *Functional Characteristics* opcode values,
     add 5 reference vectors from the manual, exhaustive round-trip on
     all opcode shapes, push to `sw-comp-history` org via
     `gh repo create --public --source --push`).

10. STOP.

# Constraints

- This step does NOT touch `sw-embed/sw-cor24-isa`. It reads from there
  for cross-check; it writes only to a tempdir and to `gen-isa/docs/`.
- The COR24 retrofit (writing actual code into `sw-embed/sw-cor24-isa`)
  is phase 6 in `plan.md`; out of scope for this saga.
- ASCII-only in `cor24-validation.md` (markdown-checker must pass).
- gen-isa's own clippy/fmt/test bar from steps 4 and 5 still applies.

# Acceptance

- `gen-isa/docs/cor24-validation.md` exists and covers the per-module
  classification described above.
- The verdict is recorded explicitly. If "not faithful enough", the saga
  cannot proceed to step 7 without first addressing the gaps; capture the
  follow-up plan.
- `gen-isa` itself remains clippy/fmt/test-clean.
