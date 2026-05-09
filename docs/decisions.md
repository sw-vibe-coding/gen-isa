# Design Decisions: foundation-and-1130-bringup saga

> Status: locked. Created 2026-05-08. Resolves the open decisions in
> `docs/plan.md` Sec 14 and `docs/design.md` Sec 11 that block code work in
> this saga. Future sagas may add or revise decisions; existing entries here
> stand unless their "Revisit if" trigger fires.

This document is the authoritative answer for every decision question that
must be answered before code lands in the saga `foundation-and-1130-bringup`.
Each section states the question, the options considered, the choice, the
rationale, and a "revisit if" trigger.

---

## 1. Repo layout (plan.md D1, design.md D1)

**Question.** Workspace vs sibling repos for the layered crates?

**Options considered.**
- A. Single Cargo workspace under one repo.
- B. Sibling repos, one per crate, distributed across themed orgs.
- C. Hybrid: workspace for framework, siblings for per-ISA.

**Choice.** B -- sibling repos in themed orgs:

- `sw-langtools` -> `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`,
  `sw-codegen-core`; future `sw-riscv32i-*`.
- `sw-comp-history` -> `sw-{ibm1130,cdp1802,s370,s390}-{isa,target,codegen,asm,emulator}`.
- `sw-embed` -> existing `sw-cor24-isa` and future `sw-cor24-{target,codegen}` (retrofit).
- `sw-vibe-coding/gen-isa` -> this generator repo. No generated code lives here.

**Rationale.** Org boundaries match domain boundaries (history vs framework
vs embedded toolchain vs methodology). Future readers find things by topic.
Cargo path-deps remain compatible with sibling clones for dev; git deps are
the migration target once the framework stabilises.

**Revisit if.** Cross-repo refactoring becomes a major drag, or per-repo CI
across coordinated breaking changes becomes painful enough to justify the
single-workspace cost.

---

## 2. IR shape (plan.md D2, design.md D2)

**Question.** What shape should `sw-tir` take?

**Options considered.**
- A. SSA with phi nodes.
- B. SSA with block parameters.
- C. Three-address non-SSA.
- D. Stack machine.

**Choice.** B -- SSA with block parameters.

**Rationale.** Block parameters are easier to lower to register-pair moves
than phi nodes and are equivalent in expressive power. SSA in general gives
the optimisation passes and register allocator the cleanest input. A stack
machine would be simpler for frontends but harder for codegen, which is
where the per-ISA work concentrates -- we optimise for the side that is
multiplied across ISAs.

**Revisit if.** A frontend's emission shape makes block-parameter SSA
demonstrably awkward, or a planned target proves hard to lower from
block-parameter SSA.

---

## 3. SSA construction (design.md D3)

**Question.** Do frontends produce SSA directly, or does a post-pass convert?

**Options considered.**
- A. Direct SSA emission only.
- B. Alloca-based naive emission; `mem2reg` cleans up.
- C. Both, frontend's choice.

**Choice.** C -- `IRBuilder` supports both. Frontends pick whichever fits
their internal shape. `mem2reg` in `sw-tir-opt` cleans up the alloca form.

**Rationale.** Some frontends (e.g. those that already track lifetimes
explicitly) produce SSA naturally; others (parser-driven naive emitters)
find alloca-then-`mem2reg` less invasive. Supporting both is a small
builder-API addition with no consumer cost.

**Revisit if.** `mem2reg` becomes a bottleneck, or the dual-mode builder
proves error-prone.

---

## 4. Distinct address types per ISA (design.md D4)

**Question.** Should each ISA define its own address newtype, or share a
generic `u64`/`usize`?

**Options considered.**
- A. Generic `u64` everywhere.
- B. Newtype per ISA (`WordAddress` for 1130, `ByteAddress` for COR24, ...).

**Choice.** B -- newtype per ISA.

**Rationale.** Word-addressed (IBM 1130) and byte-addressed (most others)
memory must not be silently mixed. The newtype ergonomics tax is small;
the correctness gain is large. The `AddressType` trait in `sw-isa-core`
provides the shared API.

**Revisit if.** Cross-ISA tooling (e.g. a unified disassembler) finds the
newtype distinctions painful to bridge.

---

## 5. Custom allocator on IBM 1130 (design.md D5)

**Question.** Generic linear-scan with aggressive spill, or a 1130-specific
allocator?

**Options considered.**
- A. Generic linear-scan plus aggressive spill.
- B. Custom allocator that knows ACC+EXT pair semantics.

**Choice.** A initially. Defer B until measured codegen quality demands it.

**Rationale.** This is a teaching toolchain, not a perf project (see
`architecture.md` Sec 6.1). Aggressive spill on a single-accumulator ISA
produces correct code even if slow. Writing a custom allocator before any
codegen exists is premature. `RegisterClasses::fixed_pairs()` in
`sw-target-core` already declares ACC+EXT, so the generic allocator can
refuse to split the pair.

**Revisit if.** Generated code is so spill-heavy it cannot fit in the
1130's 4K-word memory for non-trivial programs, or if frontend authors
complain that the output is unreadable.

---

## 6. ABI documentation format (design.md D6)

**Question.** Plain markdown per `-target` crate, or formal TOML/structured
descriptors?

**Choice.** Plain markdown (`docs/abi.md` per `-target` crate).

**Rationale.** No tool currently consumes a structured ABI descriptor. We
will not invent the format until a consumer exists. Markdown is human-first;
when a tool needs the data, it can be extracted then.

**Revisit if.** A tool (linker, debugger, FFI generator) needs to read the
ABI programmatically.

---

## 7. Conformance crate (design.md D7)

**Question.** Build a workspace-level conformance test crate now, or defer?

**Choice.** Defer until the second ISA exists.

**Rationale.** A conformance crate exists to test `(Arch, Target, Backend)`
triples uniformly. With one ISA there is no uniformity to test. Building
it prematurely bakes in 1130-specific assumptions.

**Revisit if.** When the second ISA's `-isa` + `-target` + `-codegen`
crates exist, build the conformance crate then.

---

## 8. Pilot frontend selection (plan.md D3)

**Status.** Deferred. Out of this saga's scope.

**Rationale.** This saga ends at "IBM 1130 quintet exists and tests pass".
No frontend integration yet. Pilot selection happens in a future phase-3
saga after phase-0 discovery (`plan.md` Sec 3) categorises each compiler.

**Revisit if.** Phase-0 discovery completes and the saga sequencing changes.

---

## 9. Existing IR-shaped tools to reuse (plan.md D4)

**Status.** Deferred to phase-0 discovery saga.

**Rationale.** Phase-0 discovery audits each existing compiler's backend
shape. That audit surfaces any IR-shaped intermediate representations
already in the ecosystem worth reusing or learning from. Doing the audit
mid-1130-bringup would derail focus.

**Revisit if.** Phase-0 saga completes and finds reusable IR.

---

## 10. Crate naming (plan.md D5)

**Question.** Confirm crate-name prefixes.

**Choice.** Confirmed:

- Framework: `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`,
  `sw-codegen-core`.
- Per ISA: `sw-{arch}-{isa,target,codegen,asm,emulator}`.

**Rationale.** Matches the existing `sw-cor24-isa` precedent. Hyphenated,
prefix-namespaced, role-suffixed. Easy to grep, easy to predict.

**Revisit if.** Crates.io publication forces a name conflict.

---

## 11. Phase 4 ISA selection (plan.md D6)

**Status.** Deferred. Out of this saga's scope.

**Rationale.** Phase 4 is the second new ISA after IBM 1130. This saga
ends at the end of phase 2 (IBM 1130 done). Phase 4 selection (CDP1802
vs RISC-V I32) belongs in a future phase-4-prep saga and benefits from
1130 postmortem findings.

**Revisit if.** Postmortem after this saga changes the recommendation.

---

## 12. Frontend audit (design.md D8)

**Status.** Deferred to phase-0 discovery saga.

**Rationale.** Same as decision 9. Phase-0 discovery is a separate saga
that produces `docs/other-isas/discovery.md` per `plan.md` Sec 3.

**Revisit if.** Phase-0 saga produces results that affect the framework
trait surface.

---

## 13. Cross-repo dependency strategy (saga-specific)

**Question.** How do framework crates and per-ISA crates depend on each
other across sibling repos?

**Options considered.**
- A. Cargo path deps assuming sibling clone layout.
- B. Git deps from day 1.
- C. Crates.io publication.

**Choice.** A for dev (`path = "../sw-isa-core"`). Migrate to B (or C)
once the framework stabilises (post-3rd-ISA per `design.md` Sec 10).

**Rationale.** Path deps make iteration fast; the trait surface will
change during 1130 bring-up and we want zero-friction edits. Git deps
and crates.io add publish-cycle tax that hurts iteration. The framework's
policy already forbids crates.io until three ISAs ship end-to-end.

**Revisit if.** A contributor cannot conveniently maintain sibling clones,
or CI needs a stable cross-repo dep graph before the third ISA.

---

## 14. Generator scope (saga-specific)

**Question.** What does the hybrid generator in `gen-isa` produce, and
what remains hand-written?

**Choice.**

- Spec-driven (via TOML spec): `opcode.rs`, `register.rs`, `encode.rs`,
  `decode.rs`, `branch.rs`, `format.rs` (when needed), `lib.rs`
  (`Architecture` impl).
- Skeleton-only (TODO holes): `Cargo.toml`, `LICENSE`, `COPYRIGHT`,
  `README.md`, `tests/` directory, `-target` ABI / `CallingConvention` /
  `RegisterClasses`, `-codegen` instruction-selection patterns, `-asm`
  parser, `-emulator` exec semantics.

**Rationale.** The mechanical bits (opcode tables, encoding bit-fields)
are high-leverage to generate from a spec. The judgemental bits (ABI,
codegen patterns, emulator semantics) cannot meaningfully be specified
and remain hand-written. The hybrid keeps the spec format small enough
to write by hand.

**Revisit if.** A spec-format extension would clearly automate a
hand-written piece (e.g. an exec-semantics DSL that is actually viable).

---

## 15. Saga exit criteria (saga-specific)

**Choice.** This saga is done when:

- Five framework crate skeletons exist in `sw-langtools` and compile.
- The hybrid scaffolder in this repo produces valid per-ISA crate quintets.
- The IBM 1130 quintet exists in `sw-comp-history` with:
  - Round-trip tests passing exhaustively for all opcode shapes.
  - Five reference vectors from *IBM 1130 Functional Characteristics*
    decode and re-encode correctly.
  - Asm `asm -> bytes -> disasm -> asm` round-trips.
  - Emulator runs at least one curated test program (no frontend
    integration).
- `docs/decisions.md`, `docs/spec-format.md`,
  `docs/postmortem-1130-bringup.md` committed in this repo.
- `docs/status.md` reflects reality.

**Out of scope.** Pilot frontend hello-world (separate future saga);
CDP1802 / RISC-V / S/370 / S/390 bring-up; COR24 retrofit; crates.io
publication.

**Revisit if.** Bring-up reveals an exit criterion is unmeetable as
stated, or a deferred item turns out to be an actual blocker.

---

## 16. HLASM-grade ambition for `sw-ibm1130-asm`

**Question.** What grade of assembler do we ship for IBM 1130?

**Options considered.**
- A. Basic line assembler (mnemonic + operands, no symbols).
- B. HLASM-grade with macros, conditional assembly, full directive set.
- C. **Simple HLASM subset first; grow as justified**.

**Choice.** C. Initial scope: labels, `ORG`, `EQU`, `DC`, `DS`/`BSS`/`BES`,
expressions (with `*` current-address), forward references via two-pass.
Macros, conditional assembly (`AIF`/`AGO`), and `SETA`/`SETB` are deferred
until a real consumer needs them.

**Rationale.** Full HLASM is a multi-month effort for a single-ISA
assembler; we don't have a consumer yet that justifies it. Simple subset
covers everything the IBM 1130 emulator's curated test programs need
plus the foreseeable codegen output. Add features when they pay for
themselves. `softwarewrighter/S1130/docs/Assembler.md` is the design
reference (read-only -- do not borrow code per memory rule).

**Revisit if.** A frontend or test program needs macros / conditional
assembly to express something in a non-painful way.

---

## 17. HLASM-in-HLASM bootstrap is a parallel future saga

**Status.** Recorded for direction; not in this saga's scope.

**Decision.** Once `sw-ibm1130-asm` (Rust) and `sw-ibm1130-emulator` ship
end-to-end, a separate saga authors HLASM source for the same simple
subset, assembles it via the Rust HLASM, runs it on the emulator, and
verifies parity. Provides a self-hosted reference and a forcing function
for HLASM correctness.

**Rationale.** Self-hosting is a valuable correctness property and a
classic compiler-bootstrap pattern. But it requires the Rust HLASM to be
working first; doing both in parallel inside one saga creates dependency
cycles and slows both. Sequencing them keeps each saga simple.

**Revisit if.** A use case forces the order to change.

---

## 18. FPGA execution augments the emulator (not replaces it)

**Status.** Recorded for direction; not in this saga's scope.

**Decision.** A future `sw-ibm1130-fpga` (or similarly named) crate may
provide FPGA-board-backed execution alongside `sw-ibm1130-emulator`. Both
share `sw-ibm1130-isa` (decode / encode / disassemble) and
`sw-ibm1130-target` (ABI). The `Backend` from `sw-codegen-core` does not
care which executor consumes its output. The emulator stays as a
software-only execution path forever; FPGA is an alternative path,
chosen at run-time or by build-time feature flag.

**Rationale.** Two backends keep us honest -- if FPGA execution diverges
from emulator, that's a bug we want to catch. Sharing `-isa` and
`-target` means FPGA support doesn't fork the toolchain.

**Revisit if.** FPGA execution exposes ISA-level constraints (e.g.
timing-sensitive operations) that don't fit the existing trait surface.

---

## 19. IBM 1130 encoding: real per Functional Characteristics, not toy

**Question.** Adopt the simplified 16-bit encoding from
`sw-comp-history/ibm-1130-rs` (bits 15-12 opcode, 11-8 modifier, 7-0
address) or implement the real 1130 Short / Long form encoding?

**Choice.** **Real encoding** per *IBM 1130 Functional Characteristics*
(GA26-5881). Short form: 16-bit, F-bit dispatch, 8-bit displacement.
Long form: 32-bit, indirect bit, 16-bit address. F bit at bit 5
discriminates.

**Rationale.** Project goals are historic-faithful reimplementations.
Toy encoding diverges from real 1130 binaries and breaks any future
integration with original software (DMS subroutines, sample programs,
disk images from `softwarewrighter/demo-ibm-1130-system`). Cost is
~1 week of careful encode/decode + reference vectors; payoff is
authenticity and emulator-level interoperability with reference
material.

**Revisit if.** Functional Characteristics turns out to require something
we cannot express in our `Architecture::Instruction` shape.

---

## 20. Do not borrow code from `softwarewrighter/S1130`

**Status.** Already enforced via session memory; documented here for
future readers.

**Decision.** `softwarewrighter/S1130` has no top-level LICENSE file
(it is a fork the user does not control). Use it as a **read-only
reference** for opcode tables, instruction semantics, assembler
directive coverage, and structural sanity checks. Do **not** copy
or port code from it. Numeric test vectors derived from the IBM 1130
*Functional Characteristics* manual are fine to re-derive (they
authoritatively belong to the manual, not S1130) -- but do not lift
S1130's test files directly.

For actual code reuse, use the two MIT-licensed user-controlled
projects: `sw-comp-history/ibm-1130-rs` (CPU + simple assembler,
~70-80% reusable) and `softwarewrighter/demo-ibm-1130-system`
(peripheral simulation, when needed).

**Revisit if.** S1130 gains a clear LICENSE that the user
controls / accepts.
