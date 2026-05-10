# gen-isa

Generator and design hub for a multi-ISA toolchain retargeting effort.
This repo is **generators only** -- no compiled libraries, no per-ISA
code, no frontend code. Generated artifacts live in their respective
organisations.

## What this repo contains

- `docs/` -- design docs (PRD, architecture, design, plan,
  porting-guide, decisions, spec-format), the 1130 bring-up
  postmortem, the FORTH-on-1130 plan, and the character-encoding
  plan.
- `docs/spec-examples/` -- worked TOML specs for COR24 and IBM 1130.
- `.agentrail/` -- agent saga record (durable, tracked in git).
- `src/` and `Cargo.toml` -- the `gen-isa` Rust binary that scaffolds
  per-ISA crate quintets (`-isa`, `-target`, `-codegen`, `-asm`,
  `-emulator`) from a TOML spec, with mechanical Rust code emission
  for the `-isa` crate's opcode/register/encode/decode/branch/lib
  modules.

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

## Sagas

Work is tracked as agentrail sagas. Each session does exactly one
step per `CLAUDE.md`.

- **`foundation-and-1130-bringup`** (complete, 2026-05-10). 13
  steps; delivered phase 1 (layered scaffolding in `sw-langtools`)
  and phase 2 (IBM 1130 quintet in `sw-comp-history` + 5 demo
  programs running on the emulator including a 1054/console
  hello-world). Postmortem at `docs/postmortem-1130-bringup.md`.
- **`forth-on-1130`** (planned, not yet started). Phase-3 pilot
  frontend: bring up FORTH on the 1130, leveraging the historical
  fact that the *first* FORTH was implemented on a 1130 by Charles
  Moore in 1968. Plan at `docs/forth-on-1130-plan.md`.

Current state: see `docs/status.md`.

## Status

`0.1.x`. Phase 1 + phase 2 complete; phase 3 (FORTH on 1130) is
planned and ready to start as a separate saga. See
`docs/status.md` for the live phase-by-phase state.

## License

MIT.
