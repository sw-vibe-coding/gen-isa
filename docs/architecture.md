# Architecture: Multi-ISA Toolchain Retargeting

> Status: exploratory draft, not git-tracked. Created 2026-05-08.

## 1. The real problem

The original framing — "describe each ISA in a Rust crate" — is a small piece of the goal. The actual goal is **port the COR24 language toolchain to additional ISAs with bounded per-ISA effort**. The toolchain includes APL, BASIC, C, FORTH, FORTRAN, Lisp, Pascal, PL/SW, Smalltalk, SNOBOL4, sws, tuplet, and others. Multiplying (12 languages × 5 new ISAs) by hand is a 60-port tax we want to reduce to **5 ports** — one per ISA, shared across all languages.

That requires more than ISA description crates. It requires a **layered architecture** in which most of the toolchain is target-independent, and per-ISA work is concentrated in a small, well-defined surface.

## 2. Layered architecture

```
                 ┌─────────────────────────────────────────────────┐
                 │ Language frontends (one per language, written   │
   target-       │ once): APL, BASIC, C, FORTH, FORTRAN, Lisp,     │
   independent   │ Pascal, PL/SW, Smalltalk, SNOBOL4, sws, tuplet  │
   (write once,  └────────────────────────┬────────────────────────┘
    use for                                │ emit
    every ISA)                             ▼
                 ┌─────────────────────────────────────────────────┐
                 │ sw-tir — Target-Independent Representation      │
                 │ (typed three-address SSA-ish IR + value/type    │
                 │ system + linkage metadata)                      │
                 └────────────────────────┬────────────────────────┘
                                           │ optimised by
                                           ▼
                 ┌─────────────────────────────────────────────────┐
                 │ sw-tir-opt — IR-level optimisations             │
                 │ (constant fold, DCE, mem2reg, basic inlining,   │
                 │ peephole). All target-independent.              │
                 └────────────────────────┬────────────────────────┘
                                           │ consumed by
                                           ▼
                 ┌─────────────────────────────────────────────────┐
                 │ sw-codegen-core — codegen scaffolding           │
                 │ (instruction-selection framework, register      │
                 │ allocator interface, branch relaxation, label   │
   target-       │ fixup, pretty-printer, object emission).        │
   independent   │ Generic over A: sw-target-core::Target.         │
                 └────────────────────────┬────────────────────────┘
                  ╔════════════════════════ A: Target boundary ════╗
   per-ISA       ╔════════════════════════════════════════════════╗
   (one set      ║ sw-{arch}-codegen — instruction selection       ║
   per new       ║ patterns, lowering rules, target-specific       ║
   ISA)          ║ peepholes. Implements sw-codegen-core::Backend. ║
                 ║                                                  ║
                 ║ sw-{arch}-target — calling convention, register  ║
                 ║ classes, type widths, alignment, frame layout.   ║
                 ║ Implements sw-target-core::Target.               ║
                 ║                                                  ║
                 ║ sw-{arch}-isa — opcodes, encoding, registers,    ║
                 ║ branch ranges. Implements sw-isa-core::          ║
                 ║ Architecture. (This is the cor24-isa shape.)     ║
                 ╚══════════════════════════════════════════════════╝
                                           │ also feeds
                                           ▼
                 ┌─────────────────────────────────────────────────┐
                 │ sw-{arch}-asm — assembler (per ISA)              │
                 │ sw-{arch}-emulator — emulator (per ISA)          │
                 │ sw-{arch}-disasm — disassembler (per ISA, or     │
                 │ unified via sw-isa-core trait surface)           │
                 └─────────────────────────────────────────────────┘
```

**The boxed area in the middle is the "porting boundary".** Bringing up a new ISA means writing the three crates inside the double border (`sw-{arch}-{isa,target,codegen}`). Everything above is shared across all ISAs; everything below (asm, emulator) is ISA-specific tooling that's separate from the language toolchain.

## 3. Why this layering pays off

| If we did this... | We'd pay this cost per new ISA            |
| ----------------- | ----------------------------------------- |
| Each compiler emits target asm directly (today's state) | Rewrite **every** compiler backend per ISA. (12 langs × 5 ISAs = 60 backends.) |
| Shared IR, per-ISA codegen (proposed)                   | One codegen per ISA. (5 codegens covers all 12 languages.) |

The cost of the layering: a one-time refactor of each existing compiler to emit `sw-tir` instead of COR24 assembly. That refactor is a per-language cost paid **once**, after which every ISA is free.

## 4. What stays in `sw-cor24-isa`'s spirit

`sw-cor24-isa` is small, dependency-light, no-runtime, easy-to-read. The **`sw-{arch}-isa`** layer preserves all of that:

| Module        | Responsibility                                                  |
| ------------- | --------------------------------------------------------------- |
| `opcode.rs`   | `Opcode` enum, `From<u8>`, `format()`, `mnemonic()`, decoded form |
| `register.rs` | register name table + parser                                    |
| `branch.rs`   | branch-offset constants, reachability helpers                   |
| `encode.rs`   | encoding helpers (operands → bytes)                             |
| `decode.rs`   | decoding helpers (bytes → operands)                             |
| `lib.rs`      | re-exports + `impl sw_isa_core::Architecture`                   |

This shape transfers cleanly to every planned ISA. The differences are confined to the contents of each module, not the module structure.

## 5. The two new layers above `-isa`

### `sw-{arch}-target`

Describes the ISA in its **role as a compiler target**, not as a hardware specification:

- Calling convention: argument registers, return register, caller/callee-saved sets.
- Stack: growth direction, frame-pointer convention, alignment.
- Type widths: `int`, `long`, `pointer`, `size_t` mapped to bytes/words.
- Address-unit awareness: byte-addressed (most ISAs) vs word-addressed (IBM 1130).
- Register classes for the allocator: which registers are GPR-allocatable, which are reserved, which form fixed pairs (1130's ACC+EXT).

Calling conventions and ABIs are **invented** for historical ISAs (no vendor reference). This crate is where the invention is documented.

### `sw-{arch}-codegen`

Lowers `sw-tir` IR to native instructions. This is where the real per-ISA work lives:

- Instruction-selection patterns (IR op → ISA op or sequence).
- Target-specific peepholes.
- Branch relaxation (using `sw-{arch}-isa`'s branch-range constants).
- Function prologue / epilogue templates.
- Lowering of "abstract" IR ops to whatever the ISA can do (e.g. 1130 multiply via ACC+EXT pair, CDP1802 increment-via-INC vs separate ADD).

Most of the *scaffolding* (register-allocation framework, label-fixup, asm pretty-printer) lives in `sw-codegen-core`. Per-ISA codegen plugs into it.

## 6. Per-ISA differences that the layering must accommodate

| Concern              | COR24 | IBM 1130 | CDP1802 | S/370 | S/390 | RISC-V I32 |
| -------------------- | ----- | -------- | ------- | ----- | ----- | ---------- |
| Word size            | 24-bit data, byte ops | 16-bit | 8-bit | 32-bit | 32 → 64-bit (ESAME) | 32-bit |
| Address unit         | byte | **word** | byte | byte | byte | byte |
| Endianness           | byte stream | big | big | big | big | little |
| Register file        | 8 × 24-bit | ACC + EXT + 3 XR | 16 × 16-bit + P/X/D | 16 GPR + 4 FPR + PSW | 16 GPR + 4 FPR + 16 AR + PSW | 32 × 32-bit (x0=0) |
| Instr length         | 1/2/4 bytes | 1/2 words | 1–3 bytes | 2/4/6 bytes | 2/4/6 bytes | 4 bytes |
| Length determined by | first byte | F bit | first byte | top 2 bits of byte 0 | top 2 bits of byte 0 | fixed |
| Encoding mechanism   | ROM lookup | bit fields | bit fields | bit fields, multi-format | bit fields, multi-format | bit fields |
| Register-rich for codegen? | yes | **no** | tight | yes | yes | yes |
| PC quirks            | pipeline +4 | none | **R(P) is the PC** | PSW holds PC + flags | PSW holds PC + flags | none |
| Address-space model  | flat | flat | flat | 24/31-bit | **multiple via access regs, ESAME 64-bit** | flat |

## 6.1 The "ISA disruption" scale

"Hardest ISA" depends on which axis you pick. Three different axes matter, and they don't agree:

### Axis A — Abstraction-surface disruption (does it force changes to `sw-isa-core` / `sw-target-core`?)

Ranks ISAs by how many baseline assumptions they violate that a generic ISA crate would naively make. **This is the axis the first-new-ISA decision should be made on**, because the whole point of phase 2 is to forge a trait surface that accommodates non-COR24 properties.

| ISA       | Address unit ≠ byte | Reg file < 8 GPRs | PC isn't a fixed register | Multi-format encoding | Multiple address spaces | **Score** |
| --------- | ------------------- | ----------------- | ------------------------- | --------------------- | ----------------------- | --------- |
| COR24     | no                  | 8 GPRs (OK)       | no (just pipelined)        | no (length only)      | no                      | 0         |
| IBM 1130  | **yes (word)**      | **yes (ACC+ext)** | no                        | no (length only)      | no                      | **2**     |
| CDP1802   | no                  | 16 GPRs (OK)      | **yes (R(P))**            | no                    | no                      | **1**     |
| RISC-V I32| no                  | 32 GPRs (OK)      | no                        | yes (R/I/S/B/U/J)     | no                      | 1         |
| S/370     | no                  | 16 GPRs (OK)      | no                        | **yes (10+ formats)** | partial (24/31-bit)     | **2**     |
| S/390     | no                  | 16 GPRs (OK)      | no                        | yes                   | **yes (access regs)**   | **2**     |

### Axis B — Absolute implementation effort (calendar weeks)

Ranks by total effort to ship `-isa` + `-target` + `-codegen` + `-asm` + `-emulator`. Driven by ISA size, instruction count, and execution-semantics complexity. **Not** the axis to pick the first ISA on — bigger ISA ≠ better validation of the abstraction.

| ISA       | Op count (~) | System mode | Effort (rough) |
| --------- | ------------ | ----------- | -------------- |
| CDP1802   | ~90          | none        | **lowest**     |
| RISC-V I32| ~40 base     | none        | low            |
| IBM 1130  | ~30          | minimal     | low–medium     |
| S/370     | ~200+        | full        | **high**       |
| S/390     | ~600+        | **very full** (DAT, AR mode, ESAME, IRB, …) | **highest** |

### Axis C — Codegen-quality difficulty (will the generic linear-scan allocator produce decent code?)

Ranks by how much per-target codegen acrobatics are needed for acceptable output.

| ISA       | Difficulty | Why                                                         |
| --------- | ---------- | ----------------------------------------------------------- |
| RISC-V I32| **easy**   | Register-rich, regular formats, every IR op maps cleanly    |
| COR24     | easy       | 8 GPRs, simple formats                                      |
| S/370     | medium     | Format dispatch, but register-rich enough                   |
| S/390     | medium     | Same as S/370, plus optional 64-bit value lowering          |
| CDP1802   | **hard**   | Tight register file, R(P) selection forces extra moves      |
| IBM 1130  | **hardest**| Single accumulator forces aggressive spill; ACC+EXT pair    |

### Reading the scale

- **For "validate the abstraction":** prefer ISAs scoring 2 on Axis A. **IBM 1130, S/370, and S/390 all score 2**, but their disruptions hit different parts of the abstraction:
  - 1130 disrupts `AddressUnit` (which lives in `sw-isa-core`) and `RegisterClasses` / codegen design.
  - S/370 disrupts the format-dispatch design (`Format` trait, decode/encode multi-format).
  - S/390 disrupts the address-space and target-mode design (access registers, ESAME).
- **For "ship something quickly":** prefer Axis B low. CDP1802 or RISC-V.
- **For "exercise per-ISA codegen complexity":** Axis C high. 1130 or CDP1802.

### Why IBM 1130 is recommended for phase 2

1130 hits both Axis A (score 2) and Axis C (hardest). Its disruptions land in the parts of `sw-isa-core` and `sw-target-core` that are easiest to bake the wrong assumption into early — `AddressUnit`, register class design, allocator constraints. Locking those down against 1130 protects every subsequent ISA, including the byte-addressed register-rich ones.

It's **not** the absolute hardest ISA. S/390 is. But S/390 is hard along Axis B (massive op count, DAT, system mode, ESAME) — that's effort, not abstraction-surface validation. Bringing up S/390 first would mean spending months on emulator semantics before ever knowing whether the trait surface accommodates word-addressed memory.

### Defensible alternative: CDP1802 first

CDP1802 is also a reasonable phase-2 choice. It's smaller (faster), 8-bit (forces I16/I32 lowering early), and exercises the PC-via-R(P) quirk that the trait surface might silently assume away. The trade-off is that it doesn't stress `AddressUnit` (the 1130's strongest contribution).

A possible compromise: **start CDP1802 in phase 2 and IBM 1130 in phase 4**, on the theory that an early shipped ISA builds confidence faster. This swaps speed for breadth-of-disruption-coverage. The default plan keeps 1130 first; user may direct otherwise.

### Where S/370 and S/390 fit

Both score 2 on Axis A but high on Axis B. Defer to phase 8 (after 1130, CDP1802, RISC-V, COR24 retrofit). By that point:

- The trait surface has been validated against three new ISAs.
- The format-dispatch design (refined during S/370 bring-up) inherits a stable foundation.
- S/390's access-register and address-space-mode features can either be modelled at the `-target` layer (different `Target` impls for different modes) or deferred to a separate `sw-s390-system` crate. Decide during phase 8.

The S/390 access-register mechanism is genuinely unique among planned ISAs and may force one further extension to `sw-target-core` (an "address-space mode" notion). Better to discover this on a mature foundation.

## 6.2 What the layering specifically does for each disruption

- Putting `AddressUnit` in `sw-isa-core` so the IR can size pointers correctly per target — covers the IBM 1130 word-address case.
- Letting `sw-{arch}-target` declare its own register-allocation rules (1130 declares ACC + EXT as a fixed pair, CDP1802 declares P-selectable PC behaviour) — covers register-poor and PC-quirk cases.
- Letting `sw-{arch}-codegen` do whatever instruction-selection acrobatics the ISA demands — covers everything else, including S/370 multi-format encoding.
- Leaving "address-space mode" (S/390 access registers, ESAME 64-bit toggle) as an open extension to revisit during phase 8 — explicitly NOT pre-designed.

## 7. Repo layout

```
sw-embed/
  sw-tir/                 ← shared IR + types + linkage metadata
  sw-tir-opt/             ← target-independent passes
  sw-isa-core/            ← Architecture trait, address/endian/format prims
  sw-target-core/         ← Target trait, calling convention, register classes
  sw-codegen-core/        ← Backend trait, regalloc framework, branch relaxation

  sw-cor24-isa/           ← exists; retrofit to impl sw-isa-core
  sw-cor24-target/        ← new
  sw-cor24-codegen/       ← new
  sw-cor24-asm/           ← exists
  sw-cor24-emulator/      ← exists
  sw-cor24-cc/            ← exists; retrofit to emit sw-tir

  sw-ibm1130-isa/         ← new
  sw-ibm1130-target/      ← new
  sw-ibm1130-codegen/     ← new
  sw-ibm1130-asm/         ← new
  sw-ibm1130-emulator/    ← new

  sw-cdp1802-{isa,target,codegen,asm,emulator}/
  sw-s370-{isa,target,codegen,asm,emulator}/
  sw-s390-{isa,target,codegen,asm,emulator}/   ← target/codegen depend on s370 where shared
  sw-riscv32i-{isa,target,codegen,asm,emulator}/

  sw-{lang}/              ← APL, BASIC, C, FORTH, FORTRAN, Lisp, Pascal,
                            PL/SW, Smalltalk, SNOBOL4, sws, tuplet, …
                            Each emits sw-tir.
```

## 8. Boundaries and what each layer must NOT do

**`sw-isa-core` MUST NOT contain:**
- Execution semantics; ABI; calling convention; type widths.
- I/O, peripherals, interrupts, traps.
- IR types, codegen scaffolding, register allocator.

**`sw-target-core` MUST NOT contain:**
- ISA descriptions (those live in `sw-isa-core`).
- Codegen logic (that's `sw-codegen-core`).
- IR types (that's `sw-tir`).

**`sw-codegen-core` MUST NOT contain:**
- Any per-ISA pattern. (Patterns live in `sw-{arch}-codegen`.)
- IR-level optimisations. (Those live in `sw-tir-opt`.)

**`sw-tir` MUST NOT contain:**
- ISA-specific IR ops. The IR is target-independent. If a language needs a target-specific intrinsic, it goes through a generic `intrinsic("name", args)` op that codegen lowers per-target.

**Each `sw-{arch}-isa` crate MUST NOT depend on:**
- The corresponding emulator, assembler, transpiler, target, or codegen.
- `std`, ideally — keep it `no_std`-friendly, like `sw-cor24-isa` is today.

## 9. Risks the layering doesn't solve

- **Some ISAs may be too register-poor for our IR's assumptions.** A naïve mem2reg pass produces IR with many simultaneously-live virtual registers; the 1130 has effectively one. Mitigation: codegen can degrade gracefully via aggressive spilling, but performance will suffer. This is acceptable for a teaching toolchain.
- **Existing compilers may not be IR-shaped today.** If they emit COR24 assembly directly, retargeting is gated on a per-language refactor. The plan must include a discovery task to assess this before sequencing ISA work.
- **Inventing ABIs for historical ISAs is design-by-fiat.** Other people's tooling for the same ISAs won't interoperate. We accept this for our toolchain; documenting our ABIs clearly minimises confusion.

## 10. Open questions (pushed to design.md and plan.md)

- Trait surface for `sw-isa-core`, `sw-target-core`, `sw-codegen-core` — see design.md.
- Which IR shape: SSA, three-address, stack-machine, hybrid? — see design.md §6.
- What is the current shape of each compiler's backend? — see plan.md §3 (discovery).
- Does `sw-isa-core` live in this repo as a workspace member, or as a sibling repo? — see design.md §11.
