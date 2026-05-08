# PRD: Multi-ISA Toolchain Retargeting

> Status: exploratory draft, not git-tracked. Created 2026-05-08.

## 1. Background

The COR24 ecosystem includes a family of language compilers and runtime artefacts:
APL, BASIC, C, FORTH, FORTRAN, Lisp, Pascal, PL/SW, Smalltalk, SNOBOL4, sws, tuplet, and others. Today these target the COR24 ISA. We want to **port the entire toolchain to additional target ISAs** with minimal per-language, per-ISA work.

`sw-cor24-isa` is the canonical-truth crate for the COR24 ISA. It is one piece of a larger picture; the ISA description alone is not what makes porting hard — the *codegen* and *ABI* descriptions are.

## 2. Primary goal

**Make porting the toolchain to a new ISA close to "implement two thin crates" rather than "rewrite each compiler".**

Concretely: when a new ISA is added (e.g. IBM 1130), the work to bring up *every* language in the toolchain is bounded by:

- A `sw-{arch}-isa` crate (opcodes, encoding) — small, mechanical.
- A `sw-{arch}-target` crate (ABI, register classes, calling convention) — small, opinionated.
- A `sw-{arch}-codegen` crate (lower IR to instructions) — the real work, but bounded.
- Zero per-language work *for languages that already emit the shared IR*.

## 3. Target ISAs (priority order)

1. **IBM 1130** — first port; word-addressed 16-bit, single accumulator. Stress-tests the abstraction at the "does it survive a non-register-rich ISA" boundary.
2. **RCA CDP1802** — 8-bit; exercises the "any register can be the PC" quirk and a tight register file.
3. **RISC-V I32** — modern, register-rich, well-specified; sanity baseline.
4. **IBM S/370** — variable-length multi-format encoding, rich machine state.
5. **IBM S/390** — extends S/370; should reuse most of the S/370 stack.

## 4. Goals

- **G1.** Layered architecture: `sw-isa-core` (description), `sw-target-core` (ABI), `sw-codegen-core` (IR + lowering scaffolding), `sw-tir` (target-independent IR), and per-ISA crates that plug in.
- **G2.** Each language frontend emits a single shared IR (`sw-tir`). Frontends are written **once**, not per-ISA.
- **G3.** Mid-end optimisations (constant folding, dead code, basic inlining, peephole on IR) are **target-independent**.
- **G4.** Per-ISA work is concentrated in `sw-{arch}-{isa,target,codegen}`. ISA-specific tools (assembler, emulator, disassembler) reuse the `-isa` crate but are otherwise free to take the shape that suits the ISA.
- **G5.** `sw-cor24-isa`'s public API does not break for existing consumers when retrofitted to implement `sw-isa-core`.
- **G6.** A new ISA's bring-up sequence (ISA → target → codegen → asm → emu → "hello world" from at least one language) is documented and reproducible.

## 5. Non-goals

- **NG1.** Replacing or unifying the language frontends themselves. APL stays APL; Pascal stays Pascal. Only their *backend* changes.
- **NG2.** A universal IR rich enough to express every quirk of every language losslessly. `sw-tir` is pragmatic, not academic.
- **NG3.** Cross-ISA execution, JIT, dynamic recompilation, or heterogeneous binaries.
- **NG4.** ABI compatibility with vendor toolchains (e.g. matching IBM's S/370 OS linkage). We define our own ABIs and document them.
- **NG5.** Cycle-accurate or microarchitectural modelling.
- **NG6.** Floating-point, vector, or system-mode instructions in the *first* iteration of any ISA. Add later if a frontend needs them.

## 6. User stories

### As a Pascal compiler maintainer
Today my compiler emits COR24 assembly. Once the toolchain layering lands, I refactor my compiler **once** to emit `sw-tir` IR. After that, when IBM 1130 support lands I do **nothing**: my Pascal compiler retargets automatically because the 1130 codegen consumes the same IR.

### As a new-ISA porter (e.g. bringing up CDP1802)
I implement three small crates: `sw-cdp1802-isa`, `sw-cdp1802-target`, `sw-cdp1802-codegen`. I run the existing test suite for any one frontend (say BASIC) against my codegen and watch the failures shrink. When BASIC works, every other language that emits IR works too.

### As a frontend author of a brand-new language (e.g. tuplet)
I implement IR emission once. I get COR24, IBM 1130, CDP1802, S/370, S/390, and RISC-V I32 support for free.

### As a multi-ISA disassembler / doc-site author
I depend only on `sw-isa-core` and walk every implementor of `Architecture`. I get pretty-printed disassembly for every supported ISA without ISA-specific code.

## 7. Success criteria

- **S1.** Layered crates exist: `sw-isa-core`, `sw-target-core`, `sw-codegen-core`, `sw-tir`. Each `0.x`, no third-party deps beyond optional `serde`.
- **S2.** `sw-cor24-{isa,target,codegen}` exist, all current COR24 frontends emit `sw-tir` IR (one frontend at a time is acceptable), and the resulting binaries run on the COR24 emulator and FPGA. **No regressions versus current direct-emission compilers.**
- **S3.** `sw-ibm1130-{isa,target,codegen}` exist. At least one frontend (BASIC is a reasonable choice — small language, modest IR features) produces an executable that runs on an IBM 1130 emulator.
- **S4.** Adding the **third** ISA (CDP1802) takes < 2 calendar weeks of focused work for a contributor familiar with the codebase.
- **S5.** A round-trip property test passes for every (frontend × ISA) pair: source compiles → IR → assembly → bytes → disassembly → matches the assembly the codegen emitted.

## 8. Constraints and risks

- **C1. Single-accumulator ISAs.** IBM 1130 and CDP1802 lack a register file rich enough for naive linear-scan allocation. The codegen scaffolding must accommodate "spill almost everything" allocation strategies. Mitigation: explicitly design `sw-codegen-core` register-allocator to support custom allocators per target.
- **C2. Word-addressed memory.** The 1130 addresses memory in 16-bit words, not bytes. The IR's pointer arithmetic and struct layout must be parameterised by the target's `AddressUnit`. Many language semantics (C in particular) assume byte addressing — this needs care.
- **C3. ABI is invented for historical ISAs.** No vendor reference exists for "the calling convention for our 1130 toolchain". This is a feature (we control it) but each ABI must be documented up-front before any codegen lands; otherwise frontends can't target it.
- **C4. Existing compilers may not be IR-shaped.** If the COR24 compilers today emit assembly directly, refactoring each to emit `sw-tir` is a per-language effort. Quantifying this is **a discovery task that must precede the plan** (see plan.md §3).
- **C5. Premature universalism.** Designing IR for "every language we'll ever write" is a trap. **Mitigation:** IR is driven by the *concrete needs of the next language to retarget*, not speculation.
- **C6. Quirky ISAs leak quirks into IR.** CDP1802's PC-via-R(P), 1130's accumulator-pair multiply, S/370's privileged ops. **Mitigation:** quirks stay in codegen; IR sees only the abstract operation. Codegen does whatever lowering is needed, however ugly.

## 9. Out of scope (for now)

- IBM 1401, PDP-8, 6502, Z80, Z80000, 68k, x86. Interesting; not in scope.
- 64-bit RISC-V or RISC-V extensions beyond I32.
- Floating-point, vector, atomics — until a frontend needs them.
- A linker. Each codegen emits a flat object format; a linker is a separate concern.
- Debugging info (DWARF or equivalent).

## 10. Approval gate

Before any new code lands, the user reviews:

- This PRD
- `architecture.md`
- `design.md`
- `plan.md`

Discovery task in plan.md §3 (audit existing compilers' backend shape) **must** complete before phasing decisions are finalised.
