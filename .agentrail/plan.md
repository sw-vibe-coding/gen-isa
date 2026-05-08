# Saga: foundation-and-1130-bringup

## Goal

Build the layered framework crate skeletons in `sw-langtools`, build a hybrid
ISA-scaffolding generator in this repo (`sw-vibe-coding/gen-isa`), and validate
the generator by producing the IBM 1130 quintet (`isa`, `target`, `codegen`,
`asm`, `emulator`) in `sw-comp-history` with round-trip tests passing, asm
round-tripping, and a minimal emulator that runs at least one curated program.

A pilot-frontend hello-world is **out of scope** for this saga (separate future
phase-3 saga).

## Org / repo map (locked; do not relitigate per step)

- **sw-langtools** — `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`,
  `sw-codegen-core`, future `sw-riscv32i-*`.
- **sw-comp-history** — `sw-{ibm1130,cdp1802,s370,s390}-{isa,target,codegen,asm,emulator}`.
- **sw-embed** — existing `sw-cor24-isa` + future `sw-cor24-{target,codegen}` (retrofit, phase 6).
- **sw-vibe-coding/gen-isa** — THIS repo. Generators only. No generated crates live here.

## Generator shape (locked)

Hybrid: skeleton scaffolder writes file/Cargo skeletons for the full quintet;
a small TOML spec drives the mechanical bits (opcode enum, register table,
encode/decode bit-field math, branch constants, format enum, `Architecture`
impl). Codegen patterns, ABI choices, and emulator semantics remain hand-written.

## Step sequence (planned)

1. **decisions-doc** — write `docs/decisions.md` resolving plan.md D1-D6 and
   design.md D1-D8. No code.
2. **spec-format** — design the ISA TOML spec format; document in
   `docs/spec-format.md`; provide sample specs for COR24 (reference cross-check)
   and IBM 1130 (target).
3. **framework-skeletons** — author 5 skeleton crates for `sw-langtools`. Trait
   surfaces from design.md §2-§6. Push 5 sibling repos to the `sw-langtools` org.
4. **scaffolder-mvp** — Rust binary in this repo that emits per-ISA crate
   skeletons (`Cargo.toml`, `lib.rs` stub, `LICENSE`, `COPYRIGHT`, `README.md`,
   `tests/` dir). No spec parsing yet.
5. **scaffolder-codegen** — extend scaffolder to parse the TOML spec and emit
   `opcode.rs`, `register.rs`, `encode.rs`, `decode.rs`, `branch.rs`, `lib.rs`
   (`Architecture` impl).
6. **scaffolder-validation** — dry-run mode + tempdir tests. Cross-check generator
   output against the existing `sw-cor24-isa` (using a COR24 spec). Compile-check
   IBM 1130 generator output.
7. **ibm1130-isa** — generate `sw-ibm1130-isa`, push to `sw-comp-history`.
   Round-trip exhaustive on all opcode shapes; 5 reference vectors from the
   *IBM 1130 Functional Characteristics* manual decode/encode correctly.
8. **ibm1130-target** — generate `sw-ibm1130-target`, push to `sw-comp-history`.
   ABI doc in `docs/abi.md`; `CallingConvention` impl; `RegisterClasses` impl
   (ACC+EXT fixed pair, XR1/XR2 allocatable, XR3 reserved as frame pointer).
9. **ibm1130-codegen** — generate `sw-ibm1130-codegen`, push to `sw-comp-history`.
   Hand-written instruction-selection patterns for IR ops the pilot frontend
   will need (Add, Sub, Load, Store, CondBranch, Return). Snapshot tests.
10. **ibm1130-asm** — generate `sw-ibm1130-asm`, push to `sw-comp-history`.
    `asm → bytes → disasm → asm` round-trips for the curated test set.
11. **ibm1130-emulator** — generate `sw-ibm1130-emulator`, push to
    `sw-comp-history`. Minimal exec semantics covering codegen output.
    Runs at least one curated test program (NOT a frontend hello-world).
12. **postmortem** — `docs/postmortem-1130-bringup.md` in this repo: abstractions
    that changed during 1130 bring-up; lessons for ISA #2.
13. **status-update** — update `docs/status.md` to reflect actual phase progress.

## Exit criteria

- 5 framework crates exist in `sw-langtools` and compile.
- Hybrid scaffolder in this repo produces valid per-ISA crate quintets.
- IBM 1130 quintet exists in `sw-comp-history`; round-trip tests pass; asm
  round-trips; emulator runs at least one curated program.
- `docs/decisions.md`, `docs/spec-format.md`, `docs/postmortem-1130-bringup.md`
  committed in this repo.
- `docs/status.md` reflects reality.

## Out of scope

- BASIC (or any) pilot frontend refactor. Separate future saga (phase 3 in plan.md).
- Hello-world from a language frontend reaching the emulator.
- CDP1802 / RISC-V / S/370 / S/390 bring-up.
- COR24 retrofit (`-target`, `-codegen`). Phase 6 saga.
- crates.io publication.

## Notes for future sessions

- `.agentrail/` is committed alongside each step's code changes.
- Each session does exactly ONE step (per CLAUDE.md).
- If the trait surface in `sw-langtools` needs to change to accommodate IBM 1130
  (likely during steps 7-11), update `sw-langtools` and bump cargo path-dep
  deps as needed. Document the change in the postmortem step.
- Cross-repo dev layout: assume sibling clones; cargo path deps use `../<crate>`.
