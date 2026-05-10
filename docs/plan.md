# Plan: Multi-ISA Toolchain Bring-up

> Status: exploratory draft, not git-tracked. Created 2026-05-08. Resequenced 2026-05-08 to greenfield-first.

## 1. Sequencing strategy: greenfield first, retrofit COR24 last

There are two plausible orders for bringing up the layered architecture:

### Option A: Retrofit COR24 first, then add new ISAs

1. Build scaffolding (`sw-isa-core` etc.).
2. Retrofit COR24 to validate the layering against the only ISA we know.
3. Add IBM 1130, CDP1802, RISC-V I32, S/370, S/390.

**Pros:** the first end-to-end pipeline runs against a known-working ISA; less new-ISA risk during the design phase.

**Cons (decisive):**

- COR24's quirks (3-bit register fields, byte-addressed memory, ROM-based encoding, pipelined branch base = PC+4) bake into the trait surface. When ISA #2 (IBM 1130) doesn't share these properties, the abstraction strains.
- The existing COR24 toolchain *already works*. Refactoring it before the design has been pressure-tested introduces risk for zero immediate user-visible benefit.
- We commit to a design before we've seen its hardest case.

### Option B: Greenfield first — start with IBM 1130, retrofit COR24 last

1. Build *minimal* scaffolding alongside IBM 1130.
2. Bring up CDP1802 and (probably) RISC-V I32. Iterate on the trait surface freely.
3. After the design has survived 2–3 new ISAs, retrofit COR24.

**Pros (decisive):**

- The most **abstraction-stressing** ISA (IBM 1130: word-addressed memory + single accumulator — see `architecture.md §6.1` for the explicit scale and why this is *not* the same as "absolute hardest") drives the design from day 1.
- COR24 keeps working **untouched** during experimentation. Zero risk to the existing toolchain.
- By the time COR24 is retrofitted, the trait surface is stable; the retrofit is mechanical.
- Each new ISA pushes back on the design. The design is forged against real diversity, not theoretical possibilities.

**Cons:**

- No reference implementation in the same workspace during initial scaffolding. Mitigation: IBM 1130 IS the reference; the 1130 crate and the scaffolding crates are co-developed.
- No COR24 path through the new pipeline until very late. Mitigation: COR24's existing direct-emit toolchain keeps running; nothing breaks.

**Decision: Option B.** This plan is sequenced accordingly.

## 2. Sequencing rationale (specific to Option B)

- **Layering and IBM 1130 are co-developed.** The trait surface in `sw-isa-core` etc. is iterated alongside the 1130 implementation. Any 1130 quirk that doesn't fit the abstraction is a signal to fix the abstraction, not work around it. (See `architecture.md §6.1` for why 1130 specifically — it scores high on the abstraction-disruption axis, which is what phase 2 needs to validate. CDP1802 is a defensible alternative; S/370 and S/390 are *absolutely* harder but stress different axes that are better tackled later.)
- **Pilot frontend lands on IBM 1130, not COR24.** The pilot frontend's job is to prove the IR + codegen design. COR24 isn't reachable through the new pipeline yet, but the *existing* COR24 compilers continue to work via direct emission. No regression.
- **2nd new ISA validates the layering.** If 1130 + CDP1802 (or 1130 + RISC-V I32) share a stable trait surface, the design is mature.
- **3rd ISA is the green-light for retrofitting COR24.** By this point the surface won't change much; retrofitting COR24 is a mechanical port.

## 3. Phase 0 — Discovery (must complete before phase 4)

**Owner:** repo lead. **Duration:** 1–2 days of audit, no code.

Tasks:

- **0.1.** For each compiler in the toolchain (APL, BASIC, C, FORTH, FORTRAN, Lisp, Pascal, PL/SW, Smalltalk, SNOBOL4, sws, tuplet, …), record:
  - Repo path and entry point.
  - Where the "emit COR24 assembly" boundary currently is. (Function? Module? Fully scattered?)
  - Whether there is an internal IR already, or if AST goes straight to assembly.
  - Test surface for verifying the compiler still works after a refactor.
- **0.2.** Categorise each compiler:
  - **A.** Has an IR boundary; emit-`sw-tir`-instead is mechanical.
  - **B.** No IR but parser-driven emission with a clear boundary; IR insertable with moderate effort.
  - **C.** Tightly coupled to COR24; no clean IR boundary.
- **0.3.** Pick the **pilot frontend** (default suggestion: BASIC).
- **0.4.** Write findings to `docs/other-isas/discovery.md`.

Deliverable: `discovery.md` listing each compiler's category and refactor estimate.

## 4. Phase 1 — Layered scaffolding (minimal)

**Owner:** repo lead. **Duration:** 1–2 weeks.

Build *only what IBM 1130 needs* in the scaffolding crates. Resist building speculative methods. Iteration during phase 2 will expand the surface.

### 4.1 `sw-isa-core` (skeleton)

- `Architecture` trait with the methods 1130 actually exercises: `decode`, `encode`, `disassemble`, type associations, constants.
- `address::AddressType`, `endian::Endian`, `format::FormatInfo`, `format::Length`, `register::RegisterId`, `Mnemonic`.
- Conformance test scaffolding (parameterised over `A: Architecture`).

### 4.2 `sw-target-core` (skeleton)

- `Target` trait with `Arch`, `CallConv`, `type_width`, `STACK_GROWS`, `POINTER_BITS`.
- `CallingConvention` and `RegisterClasses` traits.
- `PrimType` enum.

### 4.3 `sw-tir` (skeleton)

- `Module`, `Function`, `Block`, `Op`, `Terminator`, `Type`.
- `IRBuilder` supporting alloca-based emission.
- Pretty-printer.
- Text-format parser (for golden tests).

### 4.4 `sw-tir-opt` (skeleton)

- Pass driver.
- Constant folding, DCE, mem2reg.

### 4.5 `sw-codegen-core` (skeleton)

- `Backend` trait, `Object<T>` types.
- Generic linear-scan register allocator.
- Branch relaxation driver.
- Frame layout helper.
- Asm pretty-printer.

### Exit criteria for phase 1

- All five scaffolding crates compile.
- IBM 1130 work in phase 2 can begin without scaffolding being a blocker.
- Anything not exercised by 1130 is **deferred** (do not pre-design for ISA #N).

## 5. Phase 2 — IBM 1130 bring-up (co-developed with phase 1)

**Owner:** repo lead. **Duration:** 4–8 weeks (overlaps phase 1).

Follow `porting-guide.md`. Expect to push back on `sw-isa-core` and friends as 1130-specific issues surface.

- **2.1.** `sw-ibm1130-isa` — opcodes, encoding, registers, branch ranges, `Architecture` impl. Round-trip and reference-vector tests pass.
- **2.2.** `sw-ibm1130-target` — invented ABI documented in `docs/abi.md`; type widths; register classes (with ACC+EXT pair).
- **2.3.** `sw-ibm1130-codegen` — instruction selection patterns for every IR Op the pilot frontend emits; snapshot tests.
- **2.4.** `sw-ibm1130-asm` — assembler.
- **2.5.** `sw-ibm1130-emulator` — minimal emulator covering executing what the pilot frontend produces.

### Exit criteria for phase 2

- `sw-isa-core` / `sw-target-core` / `sw-codegen-core` are stable enough that further ISAs can plug in without forcing changes.
- IBM 1130 emulator runs the pilot frontend's "hello world".
- Postmortem in `docs/other-isas/postmortem-ibm1130.md` documenting any abstraction-level adjustments made during bring-up.

## 6. Phase 3 — Pilot frontend on the IBM 1130

**Owner:** language maintainer (with assistance). **Duration:** 5–6 weeks. **Detailed design:** [`docs/forth-on-1130-plan.md`](forth-on-1130-plan.md).

**Pilot frontend choice:** **FORTH**, not BASIC. Three reasons:

- Historical fit: Charles H. Moore implemented the *first* FORTH on an IBM 1130 in 1968. Choosing FORTH on 1130 is recreating the language on its native machine.
- Original source available: Moore granted permission in 2020 to publish the recovered 1968 source ([`monsonite/1968-FORTH`](https://github.com/monsonite/1968-FORTH); 645 lines of 1130 asm + 235-line FORTH dump; 28 primitives + self-extending dictionary).
- Local reference: `~/github/sw-embed/sw-cor24-forth` is a 2600-line DTC FORTH for COR24 with three layered crates -- the structure ports cleanly to the 1130 register set.

BASIC remains an alternate; if the FORTH saga uncovers a blocker, BASIC is the fallback (and could ship later as a second pilot).

- **3.-1.** Close known infrastructure gaps before any FORTH-specific work: BSC long-form condition mask in `sw-ibm1130-isa` (postmortem Sec 4); literal expressions and historical directives (BSS/BES/DEC/EBC/DSA/ENT/EXT) in `sw-ibm1130-asm`. See `docs/forth-on-1130-plan.md` Sec 10 for the full extension list.
- **3.0.** Pull `monsonite/1968-FORTH` as a read-only reference; produce a side-by-side primitive table comparing Moore's 1968 kernel and the COR24 reference.
- **3.1.** Hand-write the 1130 FORTH kernel in 1130 assembly (28 historical primitives + dictionary structure). Assemble through `sw-ibm1130-asm`; run primitives under `sw-ibm1130-emulator`.
- **3.2.** Author `sw-ibm1130-forth` Rust crate: parser + tokeniser + compiler that emits TIR `Function`s for user-defined words and threads them via the kernel's NEXT.
- **3.3.** Wire the pipeline end-to-end: `.fth` source → parser → TIR → `sw-tir-opt` → `sw-ibm1130-codegen` → `sw-ibm1130-asm` → emitted bytes → `sw-ibm1130-emulator` → 1054/console output.
- **3.4.** Demo: a `.fth` file like `: HI ." HELLO WORLD" CR ; HI BYE` ends with `HELLO WORLD` typed on the captured console buffer.
- **3.5.** Document refactor pattern in `docs/other-isas/frontend-refactor.md` for use in phase 7.

### Exit criteria for phase 3

- `sw-comp-history/sw-ibm1130-forth` exists and is pushed.
- A FORTH source file compiles, runs on the emulator, and prints to the captured 1054/console buffer end-to-end.
- BSC mask gap closed (or explicitly documented as why-deferred-still); historical directives accepted by the assembler so future 1130-flavoured pilot work has a real on-ramp.
- Refactor pattern documented for use in phase 7 (when other frontends move to TIR).

The COR24 toolchain remains fully functional via the existing direct-emit path. Pilot frontend keeps both paths during transition; COR24 path is removed only after phase 8.

## 7. Phase 4 — Second new ISA (validates layering)

**Owner:** repo lead or designated. **Duration:** ~6 weeks once phase 2 is done.

Choose **one of**:

- **CDP1802** — exercises the PC-via-R(P) quirk and a tight 16×16 register file. Recommended because it's the most architecturally distant from 1130.
- **RISC-V I32** — register-rich, well-specified, fastest path to a "second working ISA". Good if time pressure favours speed over design diversity.

Default recommendation: **CDP1802** first (forces the design to handle two architecturally weird ISAs back-to-back). RISC-V can follow as phase 5.

Follow `porting-guide.md`. Track any abstraction-level changes in postmortem.

### Exit criteria for phase 4

- 2nd ISA's pipeline runs the pilot frontend's "hello world".
- Trait surfaces in `sw-isa-core` / `sw-target-core` / `sw-codegen-core` have **not changed** since end of phase 2 (or any changes are minor and well-justified).
- If the surfaces did change substantially, do **not** proceed to phase 5; revisit the design.

## 8. Phase 5 — Third new ISA (RISC-V I32 if not already done)

**Duration:** ~6.5 weeks.

By this point the layering should be stable. RISC-V I32 is the lowest-risk third option and pays off as a sanity baseline against a modern, register-rich ISA.

If CDP1802 was phase 4, RISC-V is phase 5. If RISC-V was phase 4, CDP1802 is phase 5.

### Exit criteria for phase 5

- 3rd ISA pipeline runs the pilot frontend's "hello world".
- Trait surfaces unchanged from end of phase 4.
- **Green light to retrofit COR24 (phase 6).**

## 9. Phase 6 — COR24 retrofit

**Owner:** repo lead. **Duration:** 2–3 weeks.

By now the design is mature; retrofit is mechanical.

- **6.1.** Add `impl sw_isa_core::Architecture for Cor24` in `sw-cor24-isa`. Keep all existing public types unchanged for backward compatibility.
- **6.2.** Create `sw-cor24-target` documenting the existing COR24 ABI used by the C compiler today. (Audit cc source to extract.)
- **6.3.** Create `sw-cor24-codegen`. Initially this is a port of the existing C compiler's emitter — same patterns, plumbed through the new layering.
- **6.4.** Tests:
  - All existing emulator tests still pass.
  - All existing assembler tests still pass.
  - Pilot frontend now produces COR24 binaries via the new pipeline; output matches (or is documented-equivalent to) the existing direct-emit output.

### Exit criteria for phase 6

- COR24 is reachable through the new pipeline.
- Pilot frontend can target COR24, IBM 1130, ISA #4, and ISA #5 from the same source.
- Existing COR24 toolchain has had **zero regressions**.

## 10. Phase 7 — Refactor remaining frontends (parallel)

**Owners:** per-language maintainers. **Duration:** parallelisable; each frontend ~1–4 weeks.

For each frontend not in category C from phase 0, in any order:

- Apply the refactor pattern from `frontend-refactor.md`.
- Verify existing test suite passes via the new pipeline (on COR24 *and* at least one new ISA).
- Add the frontend to the conformance matrix in `status.md`.

Frontends in category C are deferred; they remain COR24-only via the legacy direct-emit path until refactored.

## 11. Phase 8 — Remaining ISAs (S/370, S/390, others)

**Duration:** S/370 ~14 weeks; S/390 ~6.5 weeks (extends S/370).

S/370 and S/390 are the heaviest ISAs in scope (Axis B in `architecture.md §6.1`). Tackle after the layering has been validated against three new ISAs plus retrofitted COR24. By that point the format dispatch in `sw-isa-core` and `sw-target-core` will have been refined enough to accommodate:

- S/370's instruction-format zoo (RR, RX, RS, RSI, SI, SS, S, E, I, …).
- S/370's 24-bit and 31-bit addressing modes (target-mode toggle in `sw-target-core`).
- S/390's **access registers** for cross-address-space addressing — likely requires a `sw-target-core` extension for "address-space mode". This is the only known case where the trait surface may need a non-trivial new concept introduced *after* phase 5.
- S/390's ESAME 64-bit mode — modelled as a separate `Target` impl alongside the 32-bit one, sharing the underlying `Architecture`.

S/390 reuses S/370's opcode space substantially; bring it up as an extension. If the access-register design proves too disruptive to retrofit cleanly, isolate it in a separate `sw-s390-system` crate rather than bending `sw-target-core` for one ISA.

## 12. Phase dependencies (compact)

```
Phase 0 (discovery)
   ↓
Phase 1 (scaffolding) ── co-develops with ── Phase 2 (IBM 1130)
                                                ↓
                                             Phase 3 (pilot frontend on 1130)
                                                ↓
                                             Phase 4 (CDP1802 or RISC-V I32)
                                                ↓
                                             Phase 5 (the other one)
                                                ↓
                                             Phase 6 (COR24 retrofit)
                                                ↓
                                ┌───────────────┴───────────────┐
                                ▼                               ▼
                           Phase 7                          Phase 8
                  (other frontends, parallel)        (S/370, S/390, …)
```

Phases 7 and 8 may overlap.

## 13. Risks and mitigations (post-Option B)

| Risk | Mitigation |
| ---- | ---------- |
| IR design doesn't survive ISA #2 | Phase 4 exit criterion explicitly requires trait stability; if it changes substantially, pause and reassess. |
| Discovery (phase 0) reveals most frontends are category C | Cut scope: pick the 3–4 refactorable frontends; defer the rest. Adjust PRD success criteria. |
| 1130 codegen quality is poor due to single accumulator | Accept it for now; this is a teaching toolchain, not a perf project. Custom allocator can land later. |
| Pilot frontend has regressions on COR24 (legacy path) during refactor | Pilot keeps both paths until phase 6 completes. Drop legacy only when new path is verified equivalent. |
| Trait surface ossifies before COR24 retrofit | Phase 5 explicitly tests "third ISA fits without changes". If COR24 doesn't fit during phase 6, that's a critical signal — fix the abstraction, don't paper over it. |
| Calendar drift | Track per-ISA hours in `status.md`; if a phase exceeds 2× estimate, pause and reassess. |

## 14. Decisions to make before phase 1

These block the start of code. Resolve via discussion:

- **D1.** Workspace vs sibling repos for new layered crates? (See design.md §11 D1.)
- **D2.** SSA-with-block-params IR design confirmed? (design.md §11 D2.)
- **D3.** Pilot frontend choice — BASIC, or another? (Phase 0.3 may answer this.)
- **D4.** Are there existing IR-shaped tools in the COR24 ecosystem we should learn from / reuse before building `sw-tir`?
- **D5.** Crate naming: `sw-tir` and `sw-codegen-core`, or different prefix?
- **D6.** Phase 4: CDP1802 or RISC-V I32 first? Recommendation: CDP1802 (architectural diversity wins).

Phase 0 discovery may surface additional questions; capture them in `discovery.md`.

## 15. What this plan is NOT

- It is not a schedule with calendar dates. Durations are rough.
- It does not guarantee every frontend supports every ISA — depends on phase 0 discovery.
- It is not a public commitment until the user approves the PRD.
- It does not assume backwards compatibility with vendor toolchains for historical ISAs (IBM 1130, S/370, S/390 etc.). We invent our own ABIs.
