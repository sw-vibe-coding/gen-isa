# gen-isa

Generator and design hub for a multi-ISA toolchain retargeting effort.
This repo is **generators only** -- no compiled libraries, no per-ISA
code, no frontend code. Generated artifacts live in their respective
organisations.

## What this repo contains

- `docs/` -- design docs (PRD, architecture, design, plan,
  porting-guide, decisions, spec-format) and the agentrail saga state.
- `docs/spec-examples/` -- worked TOML specs for COR24 and IBM 1130.
- `.agentrail/` -- agent saga record (durable, tracked in git).
- (Step 4 onward) a Rust binary that scaffolds per-ISA crate quintets
  from a TOML spec.

## What this repo does NOT contain

- Per-ISA crates (`sw-{arch}-{isa,target,codegen,asm,emulator}`) --
  those live in `sw-comp-history` (historic ISAs) and `sw-embed`
  (existing COR24).
- Framework crates (`sw-isa-core`, `sw-target-core`, `sw-tir`,
  `sw-tir-opt`, `sw-codegen-core`) -- those live in `sw-langtools`.

## Org map

| Org | Hosts |
|-----|-------|
| `sw-langtools` | Framework: `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`, `sw-codegen-core`; future `sw-riscv32i-*`. |
| `sw-comp-history` | Historic ISA implementations: `sw-{ibm1130,cdp1802,s370,s390}-{isa,target,codegen,asm,emulator}`. |
| `sw-embed` | `sw-cor24-isa` (existing) and future `sw-cor24-{target,codegen}` retrofit. |
| `sw-vibe-coding/gen-isa` | This repo. Generators and design hub. |

## Saga

Work is tracked as an agentrail saga
(`foundation-and-1130-bringup`). Each session does exactly one step
per `CLAUDE.md`. See `.agentrail/plan.md` for the 13-step plan and
`.agentrail/steps/` for individual step prompts.

Current state: see `docs/status.md`.

## Status

`0.1.x`. Phase 1 (layered scaffolding) skeletons are in
`sw-langtools`; phase 2 (IBM 1130 bring-up) starts after the
scaffolder lands.

## License

MIT.
