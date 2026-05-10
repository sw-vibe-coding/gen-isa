# Status: Multi-ISA Toolchain Bring-up

> Status: exploratory draft, not git-tracked. Created 2026-05-08. Update as work progresses.

## Last updated

2026-05-09 — saga step 8 (`ibm1130-target`) complete. `sw-ibm1130-target` pushed to `sw-comp-history`; ABI invented and documented; `Target`, `CallingConvention`, `RegisterClasses` impls live with 13 smoke tests. See "Recent changes" for prior milestones.

## Phase summary

Phases per `plan.md` (resequenced 2026-05-08).

| Phase | Description                                              | State          |
| ----- | -------------------------------------------------------- | -------------- |
| 0     | Discovery: audit existing compilers' backend shape       | not started    |
| 1     | Layered scaffolding (minimal, co-developed with phase 2) | **skeletons up** (saga step 3 of 13 complete; trait surfaces will iterate during 1130 bring-up) |
| 2     | First new ISA (IBM 1130 recommended)                     | not started    |
| 3     | Pilot frontend refactor (BASIC suggested) on phase-2 ISA | not started    |
| 4     | Second new ISA (CDP1802 or RISC-V I32)                   | not started    |
| 5     | Third new ISA (the other of CDP1802 / RISC-V I32)        | not started    |
| 6     | COR24 retrofit (now that the design is mature)           | not started    |
| 7     | Refactor remaining frontends to emit `sw-tir`            | not started    |
| 8     | Remaining ISAs (S/370, S/390, …)                         | not started    |

## Crate state

### Layered scaffolding

| Crate              | Exists | Trait surface | Tests | Notes                                  |
| ------------------ | ------ | ------------- | ----- | -------------------------------------- |
| `sw-isa-core`      | **yes** | skeleton (built) | smoke | design.md §2; sw-langtools repo. ByteStream dropped from Endian (orthogonal to byte order). |
| `sw-target-core`   | **yes** | skeleton (built) | smoke | design.md §3; sw-langtools repo.       |
| `sw-tir`           | **yes** | skeleton (built) | smoke | design.md §4; sw-langtools repo.       |
| `sw-tir-opt`       | **yes** | skeleton (built) | smoke | design.md §5; sw-langtools repo. PassDriver + 3 stub passes. |
| `sw-codegen-core`  | **yes** | skeleton (built) | smoke | design.md §6; sw-langtools repo. Sub-modules (regalloc/branch/frame/asm) are stubs. |

### Per-ISA crates

| ISA       | `-isa`                | `-target` | `-codegen` | `-asm`            | `-emulator`      |
| --------- | --------------------- | --------- | ---------- | ----------------- | ---------------- |
| COR24     | **exists** (this repo, not yet `Architecture`-impl'd) | no | no | exists (separate) | exists (separate) |
| IBM 1130  | **exists** (`sw-comp-history/sw-ibm1130-isa`, 24 opcodes, 12 tests passing) | **exists** (`sw-comp-history/sw-ibm1130-target`, invented ABI, 13 tests) | no | no | no |
| CDP1802   | no                    | no        | no         | no                | no               |
| RISC-V I32| no                    | no        | no         | no                | no               |
| S/370     | no                    | no        | no         | no                | no               |
| S/390     | no                    | no        | no         | no                | no               |

The existing `sw-cor24-isa` crate (this repo) does **not yet** implement `sw_isa_core::Architecture`. Per the resequenced plan, this retrofit happens in phase 6, after 1130 + two more new ISAs. The existing COR24 toolchain (cc, asm, emulator) keeps working untouched until then.

## ISA disruption ranking (informational)

From `architecture.md §6.1`. Axis A = abstraction-surface disruption (drives phase-2 selection). Axis B = absolute effort. Axis C = codegen complexity.

| ISA       | Axis A (disruption) | Axis B (effort) | Axis C (codegen) | Suggested phase |
| --------- | ------------------- | --------------- | ---------------- | --------------- |
| COR24     | 0                   | n/a (exists)    | easy             | 6 (retrofit)    |
| IBM 1130  | **2**               | low–medium      | **hardest**      | **2 (first)**   |
| CDP1802   | 1                   | **lowest**      | hard             | 4 or 5          |
| RISC-V I32| 1                   | low             | easy             | 4 or 5          |
| S/370     | 2                   | high            | medium           | 8               |
| S/390     | 2                   | **highest**     | medium           | 8               |

## Frontend state

Discovery (phase 0) has not run. Until it does, the table below is a placeholder.

| Language    | Repo  | Backend category (A/B/C) | Refactor to `sw-tir` | Multi-ISA target list |
| ----------- | ----- | ------------------------ | -------------------- | --------------------- |
| APL         | ?     | unknown                  | not planned          | ?                     |
| BASIC       | ?     | unknown                  | not planned          | ?                     |
| C           | ?     | unknown                  | not planned          | ?                     |
| FORTH       | ?     | unknown                  | not planned          | ?                     |
| FORTRAN     | ?     | unknown                  | not planned          | ?                     |
| Lisp        | ?     | unknown                  | not planned          | ?                     |
| Pascal      | ?     | unknown                  | not planned          | ?                     |
| PL/SW       | ?     | unknown                  | not planned          | ?                     |
| Smalltalk   | ?     | unknown                  | not planned          | ?                     |
| SNOBOL4     | ?     | unknown                  | not planned          | ?                     |
| sws         | ?     | unknown                  | not planned          | ?                     |
| tuplet      | ?     | unknown                  | not planned          | ?                     |

Categories from `plan.md §3`:
- **A** = already has IR boundary, refactor mechanical
- **B** = no IR but clean boundary, IR insertion moderate effort
- **C** = tightly coupled to COR24, no clean boundary

## Conformance matrix

`(frontend × ISA)` matrix. Per the resequenced plan, the COR24 column will remain `✗` until phase 6 (the retrofit) — note that this means "wired through the new pipeline", not "works at all". Today's compilers continue to work via direct emission throughout phases 0–5.

|             | COR24 | IBM 1130 | CDP1802 | RISC-V I32 | S/370 | S/390 |
| ----------- | ----- | -------- | ------- | ---------- | ----- | ----- |
| APL         | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| BASIC       | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| C           | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| FORTH       | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| FORTRAN     | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| Lisp        | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| Pascal      | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| PL/SW       | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| Smalltalk   | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| SNOBOL4     | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| sws         | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |
| tuplet      | ✗     | ✗        | ✗       | ✗          | ✗     | ✗     |

`✓` = end-to-end (source → IR → codegen → asm → emulator) passing for representative programs.
`◐` = partial (codegen exists but emulator/runtime missing, or vice versa).
`✗` = nothing wired up yet.

## Open decisions waiting on user

From `plan.md §14`:

- **D1.** Workspace vs sibling repos for `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-codegen-core`?
- **D2.** Confirm SSA-with-block-params IR shape?
- **D3.** Pilot frontend (default suggestion: BASIC)?
- **D4.** Any existing IR-shaped tools in the ecosystem to reuse?
- **D5.** Crate naming prefix?
- **D6.** Phase 4 = CDP1802 or RISC-V I32 first? (Recommendation: CDP1802; user may prefer RISC-V for speed.)

From `design.md §11`:

- **D4 (design).** Distinct address types per ISA (proposed: yes)?
- **D5 (design).** Custom allocator on IBM 1130 (proposed: defer)?
- **D6 (design).** ABI documentation format (proposed: markdown)?
- **D7 (design).** Workspace conformance crate (proposed: yes, post-second-ISA)?

## Recent changes

- 2026-05-08: created `architecture.md`, `prd.md`, `design.md`, `plan.md`, `status.md`.
- 2026-05-08: added `porting-guide.md` (contributor-facing how-to + cherry-pick guide for COR24 patterns).
- 2026-05-08: resequenced `plan.md` to greenfield-first; COR24 retrofit moved from phase 2 to phase 6.
- 2026-05-08: added explicit ISA-disruption scale to `architecture.md §6.1` (replaces ambiguous "worst-case" language); updated cross-references in `plan.md`, `design.md`, `porting-guide.md`.
- 2026-05-08: initialised agentrail saga `foundation-and-1130-bringup`; committed `docs/decisions.md` resolving plan.md D1-D6, design.md D1-D8, and saga-specific decisions.
- 2026-05-08: linked IBM 1130 reference implementations (`sw-comp-history/ibm-1130-rs`, `softwarewrighter/demo-ibm-1130-system`, `softwarewrighter/S1130`) from `porting-guide.md` Sec 1 Prerequisites for steps 7-11 cross-checking.
- 2026-05-08: completed step `spec-format`. `docs/spec-format.md` + `docs/spec-examples/{cor24,ibm1130}.toml` define the ISA TOML spec format and provide worked samples.
- 2026-05-08: completed step `framework-skeletons`. Five sibling crates pushed to `sw-langtools` org: `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`, `sw-codegen-core`. Each compiles, tests, clippy-clean, fmt-clean. Cross-deps via `path = "../<crate>"` assuming sibling clones at `~/github/sw-langtools/`.
- 2026-05-08: trait-surface refinement during step 3 -- `Endian::ByteStream` dropped (was conflating instruction-encoding layout with byte order). `Endian` is now `Big | Little`; ISAs without multi-byte instruction fields (COR24) document the const as data-side endianness. Updated `docs/spec-format.md` and `docs/spec-examples/cor24.toml` accordingly.
- 2026-05-08: locked 5 saga-direction decisions (`docs/decisions.md` Sec 16-20): HLASM-grade asm with simple-subset-first, HLASM-in-HLASM bootstrap as parallel future saga, FPGA augments emulator (shares -isa/-target), real IBM 1130 encoding (not the toy from ibm-1130-rs), no code borrowing from S1130.
- 2026-05-09: completed step `scaffolder-mvp`. Rust binary `gen-isa` in this repo emits the 5-crate skeleton quintet (Cargo.toml / LICENSE / COPYRIGHT / README.md / .gitignore / src/lib.rs / per-role module stubs / tests/smoke.rs / -target's docs/abi.md). Filesystem trait + InMemoryFs for testing. 8 integration tests, clippy-D-warnings clean, fmt clean. Acceptance: scaffolded into a tempdir with `--framework-path $HOME/github/sw-langtools`; all 5 generated crates compile cleanly.
- 2026-05-09: completed step `scaffolder-codegen`. The scaffolder now parses an ISA TOML spec via `--spec <PATH>` and emits real Rust code for the `-isa` crate's mechanical bits: `opcode.rs` (enum + mnemonic + formats + try_from_value + Mnemonic impl), `register.rs` (enum + name + class + RegisterId impl + parse_register), `branch.rs` (constants + can_short_branch), `encode.rs` and `decode.rs` (working bit-field math for `bit_fields` style; stubs for `rom_table`), `lib.rs` (Architecture impl + Address newtype + Instruction enum), `tests/roundtrip.rs` (curated round-trip per format). 11 integration tests; clippy-D-warnings clean; fmt clean. Acceptance: scaffolded `sw-testibm-isa` from `ibm1130.toml` -- builds + 2 roundtrip tests pass; scaffolded `sw-testcor-isa` from `cor24.toml` (rom_table) -- builds + 1 ignored roundtrip + smoke passes.
- 2026-05-09: completed step `scaffolder-validation`. Cross-checked the smart scaffolder against the existing `sw-embed/sw-cor24-isa`. Per-module verdicts in `docs/cor24-validation.md`: opcode (Match), register (Match semantic; design choice deferred), branch (Match after abs_diff fix), lib (forward-looking; pre-retrofit divergence expected), encode/decode (rom_table opt-out works as designed). Bottom line: scaffolder is faithful enough to proceed to step 7. Generator improvement: `branch_rs()` now emits `abs_diff`-based `can_short_branch` matching the existing crate's symmetric semantics.
- 2026-05-09: completed step `ibm1130-isa`. **First real per-ISA crate live**: [`sw-comp-history/sw-ibm1130-isa`](https://github.com/sw-comp-history/sw-ibm1130-isa). Extended `docs/spec-examples/ibm1130.toml` from the 6-opcode sample to the full 24-opcode authoritative table (cross-checked against IBM 1130 Functional Characteristics via S1130's transcribed values). Generated via `gen-isa scaffold --spec`; hand-added `src/addr_mode.rs` (AddressingMode) and `src/branch_cond.rs` (BranchCondition) ported from `sw-comp-history/ibm-1130-rs` (MIT). 12 tests pass: 5 unit tests, 4 reference-vector + exhaustive (23552 short-form cases + 320 sampled long-form), 2 generated curated roundtrips, 1 smoke. cargo build/test/clippy -D warnings/fmt --check all clean. Generator improvements during step: shift-by-0 elision in encode/decode emit; reserved-fields-with-value-0 skip; reserved-fields-with-nonzero-value skip mask; emit `#![allow(unused_parens, clippy::all)]` in generated encode.rs/decode.rs (generated bit-fiddling code resists hand-quality lint passes).
- 2026-05-09: completed step `ibm1130-target`. [`sw-comp-history/sw-ibm1130-target`](https://github.com/sw-comp-history/sw-ibm1130-target). Scaffolded via `gen-isa scaffold --spec` (target subdir only). Hand-authored ABI documented in `docs/abi.md` (11 sections); the 1130 has no native calling convention, so the ABI is invented for this toolchain. Decisions: ACC = first scalar arg + scalar return; ACC+EXT pair for 32-bit return; XR1 caller-saved scratch; XR2 = logical SP (callee-saved); XR3 = FP (callee-saved); stack grows down; word alignment everywhere; ptr = 1 word = 16 bits; types I8/U8/Bool/I16/U16/Ptr = 1 word, I32/U32 = 2 words, I64/U64 = 4 words. `Target`, `CallingConvention`, `RegisterClasses` all implemented with 13 smoke tests covering register partitioning, type widths, and saved-set disjointness. cargo build/test/clippy -D warnings/fmt --check all clean.

## Update protocol

After every meaningful change:

1. Update the affected row(s) in this file.
2. Bump the "Last updated" date at the top.
3. Add a one-line entry to "Recent changes".

Do not let this file drift; if it's stale, the rest of the docs become unreliable.
