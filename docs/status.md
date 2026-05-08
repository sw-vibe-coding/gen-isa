# Status: Multi-ISA Toolchain Bring-up

> Status: exploratory draft, not git-tracked. Created 2026-05-08. Update as work progresses.

## Last updated

2026-05-08 — initial document set drafted (architecture, prd, design, plan, porting-guide, status). Sequencing decision: **greenfield-first, retrofit COR24 last** (see `plan.md §1`). Disruption-axis scale added to `architecture.md §6.1`. Saga `foundation-and-1130-bringup` initialised; decisions doc committed (`docs/decisions.md`); IBM 1130 reference implementations linked from `porting-guide.md` Sec 1. No code written yet.

## Phase summary

Phases per `plan.md` (resequenced 2026-05-08).

| Phase | Description                                              | State          |
| ----- | -------------------------------------------------------- | -------------- |
| 0     | Discovery: audit existing compilers' backend shape       | not started    |
| 1     | Layered scaffolding (minimal, co-developed with phase 2) | not started    |
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
| `sw-isa-core`      | no     | designed      | none  | design.md §2                           |
| `sw-target-core`   | no     | designed      | none  | design.md §3                           |
| `sw-tir`           | no     | designed      | none  | design.md §4                           |
| `sw-tir-opt`       | no     | sketch        | none  | design.md §5                           |
| `sw-codegen-core`  | no     | designed      | none  | design.md §6                           |

### Per-ISA crates

| ISA       | `-isa`                | `-target` | `-codegen` | `-asm`            | `-emulator`      |
| --------- | --------------------- | --------- | ---------- | ----------------- | ---------------- |
| COR24     | **exists** (this repo, not yet `Architecture`-impl'd) | no | no | exists (separate) | exists (separate) |
| IBM 1130  | no                    | no        | no         | no                | no               |
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

## Update protocol

After every meaningful change:

1. Update the affected row(s) in this file.
2. Bump the "Last updated" date at the top.
3. Add a one-line entry to "Recent changes".

Do not let this file drift; if it's stale, the rest of the docs become unreliable.
