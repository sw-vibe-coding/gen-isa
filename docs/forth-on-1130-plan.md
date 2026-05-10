# FORTH on the IBM 1130: Pilot-Frontend Design Doc

> Status: planning doc. Written 2026-05-10 in response to a user
> redirect: replace BASIC with FORTH as the phase-3 pilot frontend
> target on the IBM 1130, leveraging the historical fit (FORTH was
> *first implemented* on a 1130) and the local cor24-forth
> reference. ASCII-only.

## 1. Why FORTH on the 1130 (and why now)

Three reasons it beats BASIC as the pilot frontend choice:

1. **Historical fit, not just historical reference.** Charles H.
   Moore implemented the *first* FORTH on an IBM 1130 in 1968 at
   Mohasco Industries (a 1130 with a 2250 graphic display).
   Picking FORTH on 1130 as our pilot is recreating the language
   on its native machine. BASIC is a fine teaching language but
   has no special bond with the 1130.
2. **Original source survives.** Chuck Moore granted permission
   in May 2020 to publish the recovered source: 645 lines of 1130
   assembly + a 235-line text dump from the 1130 disk, recovered
   from Bob Flanders' email in 2011. Posted at
   `monsonite/1968-FORTH` on GitHub. Implements 28 primitives plus
   a self-extending dictionary -- minimal but complete.
3. **Local reference.** `~/github/sw-embed/sw-cor24-forth` is a
   2600+ line DTC FORTH kernel for COR24 that already implements
   the dictionary, threading model, and word set we'd need. Three
   layered crates (`forth-from-forth`, `forth-in-forth`,
   `forth-on-forthish`) demonstrate the bootstrap pattern. We can
   port this kernel structure to the 1130 instead of designing
   from scratch.

The triangulation is unusual: we have **two reference
implementations on different ISAs plus a clear historical anchor**
to ground the design. Most pilot-frontend choices have at most one.

## 2. Scope (what the saga delivers)

A FORTH crate that:

- Parses FORTH source text (one or more `.fth` files plus the
  REPL).
- Compiles FORTH definitions (`: word ... ;`) to TIR.
- Emits 1130 native code via the existing `sw-ibm1130-codegen` and
  `sw-ibm1130-asm` pipeline.
- Runs on `sw-ibm1130-emulator`, with the standard 28-ish FORTH
  primitives backed by 1130 instructions.
- Demonstrates a true end-to-end story: a `.fth` file goes from
  source to the 1054/console Selectric printer typing
  `HELLO WORLD ` (with the trailing space and `CR` that FORTH's
  `EMIT` sequence naturally produces).

**Out of scope** (deferred to follow-on sagas):

- Block I/O via FORTH's traditional 1024-byte block model.
- Multi-tasking FORTH.
- Redefining the kernel from FORTH itself (full self-hosting).
  Possible later, but pilot scope is "kernel in Rust + assembled
  output, FORTH user code on top".
- Original-1968-EBCDIC character handling. Use ASCII for source
  and `EMIT`; EBCDIC retrofit is the dedicated character-encoding
  saga (`docs/character-encoding-plan.md`).

## 3. Crate layout

Add one crate to the per-ISA family for the **frontend** layer
(parallel to existing `-isa`, `-target`, `-codegen`, `-asm`,
`-emulator`):

- `sw-comp-history/sw-ibm1130-forth` -- FORTH frontend that
  parses, threads, and emits TIR via `sw_tir`. Depends on:
  `sw-tir`, `sw-target-core`, `sw-ibm1130-target`,
  `sw-ibm1130-codegen`, `sw-ibm1130-asm`, `sw-ibm1130-emulator`
  (last for tests / runtime).

The crate name follows the pattern; nothing else in the layered
framework needs to change. (If the pilot frontend later gets
shared between ISAs we can hoist a `sw-forth-frontend` framework
crate then.)

## 4. Implementation strategy (phased)

### Phase 3.-1 -- Close known infrastructure gaps (1 week)

Before any FORTH-specific work, close the asm/isa gaps that block
ingesting historical 1130 source. See Sec 10 below for the full
list. Headline items: BSC long-form mask field (postmortem Sec 4),
literal expressions in operands, BSS / BES / DEC / EBC / DSA / ENT
/ EXT directives.

### Phase 3.0 -- Read & port references (1 week)

- Pull `monsonite/1968-FORTH` into `~/github/sw-comp-history` as a
  read-only reference clone.
- Read `FORTH68asm.txt` end-to-end with Carl Claunch's
  `notes on FORTX assem code.pdf` open. Build a side-by-side index
  from primitives to 1130 instructions.
- Read `~/github/sw-embed/sw-cor24-forth/forth.s` end-to-end. Note
  which primitives are direct-threaded, which are double-cell, and
  the dictionary header layout.
- Decide which kernel structure to follow:
  - **Option A:** Direct port of Moore's 1968 design (28
    primitives, indirect-threaded, EBCDIC characters). Maximum
    historical fidelity. Minimum compiler work.
  - **Option B:** COR24-style DTC kernel (50+ primitives, modern
    structure, ASCII). Faster path to a usable system. Less
    historical fidelity.
  - **Recommended:** **A** for the kernel, **B** for the user
    layer. Stand up the historical 28-primitive kernel first,
    then extend with the COR24 layered approach to add the
    additional primitives (`HERE` / `LATEST` / `WORDS` /
    `INTERPRET` / etc.) on top.

### Phase 3.1 -- Kernel in 1130 assembly (1 week)

- Author `sw-ibm1130-forth/kernel.asm` -- a hand-written 1130
  assembly kernel implementing the 28 historical primitives plus
  whatever support words the user layer needs.
- Register allocation (proposed, mirroring Moore's 1968 choices):
  - `XR1` = workspace pointer (W)
  - `XR2` = data stack pointer (DSP, grows per chosen direction)
  - `XR3` = **NOT used by FORTH** -- reserved for the
    `sw-ibm1130-target` ABI's LIBF transfer-vector base. (This is
    where our historical conventions matter: Moore had no library
    to interop with, so he could use XR3. We do, and we can't.)
  - `IAR` = the 1130's PC, threaded into through `BSI`/`BSC`.
  - `ACC`, `EXT` = arithmetic temporaries; not preserved across
    primitives.
- Instead of XR3, use a memory cell to hold a fourth pointer
  (return-stack pointer / IP) -- explicit per-primitive
  load/store, slower but correct.
- Threading model: **indirect-threaded code (ITC)** to match
  Moore's 1968 design. Each compiled word is a list of word
  addresses; NEXT loads the addressed word's CFA and jumps. ITC
  is more conservative on code size than DTC, which matters on a
  16K-word 1130.
- Assemble via `sw-ibm1130-asm`; the kernel is just another asm
  source file the assembler eats.

### Phase 3.2 -- TIR-emitting frontend (1.5 weeks)

For the layered, modern parts of the system:

- `sw-ibm1130-forth/parser.rs` -- tokeniser + parser for the
  FORTH source language. Handles `: word ... ;`, `IF/ELSE/THEN`,
  `BEGIN/UNTIL`, `DO/LOOP`, immediate words, base prefix (`#`,
  `$`, `%`).
- `sw-ibm1130-forth/compile.rs` -- per-source-token compilation:
  each parsed token becomes either a primitive call (a TIR
  `Call` to the kernel-resident primitive's address) or a
  user-defined word call.
- `sw-ibm1130-forth/emit.rs` -- emit TIR `Function`s representing
  user-defined words; the codegen pipeline (`sw_codegen_core` ->
  `sw-ibm1130-codegen`) lowers them to threaded-code arrays in
  data memory.
- The user FORTH word `: SQ DUP * ;` becomes a TIR function whose
  body is just a list of `Call(DUP)`, `Call(MUL)`, `Return`.

This is where we test whether the slot-based-no-allocator codegen
can carry actual user code. FORTH's stack-machine model is
forgiving: every primitive consumes/produces fixed-size stack
items, so register allocation isn't on the critical path.

### Phase 3.3 -- REPL + integration (0.5 weeks)

- A `cargo run --example forth-repl` reads a `.fth` file or stdin,
  compiles each line, runs it on the emulator, and prints the
  emulator's `console_output` after each line.
- Demo: `examples/hello.fth`:
  ```
  : HI ." HELLO WORLD" CR ;
  HI
  BYE
  ```
- Snapshot tests: assemble, run, assert `console_output` ==
  `"HELLO WORLD\n"`.

### Phase 3.4 -- Postmortem (0.25 weeks)

Standard saga postmortem:
- Did the slot-codegen survive real frontend output?
- Did the BSC mask gap bite?
- What in `sw-langtools` had to change?
- Cross-link from `docs/status.md`.

## 5. Saga outline (for the future agentrail saga)

```
forth-on-1130 (saga)
  step 1  decisions          locking pilot=FORTH; document choices
  step 2  reference-cloning  pull monsonite/1968-FORTH; read kernel
  step 3  kernel-asm         hand-write 28-primitive kernel
  step 4  kernel-tests       primitives execute correctly under emulator
  step 5  parser             FORTH source -> token stream
  step 6  compiler           tokens -> TIR (user words as TIR fns)
  step 7  emit               TIR -> 1130 binaries via codegen + asm
  step 8  repl               REPL example with hello.fth
  step 9  end-to-end-demo    hello.fth -> bytes -> emulator -> console
  step 10 postmortem
  step 11 status-update
```

11 steps, ~4-5 weeks of focused work. Same shape as the 1130
bring-up saga.

## 6. Why this is the right next saga (vs alternatives)

| Alternative | Pro | Con |
| ----------- | --- | --- |
| BASIC pilot first (original plan) | Familiar; lots of frontends | No special 1130 fit; lacks historical anchor |
| ISA #2 next (CDP1802 or RISC-V) | Validates `sw-langtools` against a different shape | We learn nothing about frontend integration; postmortem-Sec-2 risk persists |
| Character encoding next (EBCDIC) | Closes a known gap | Doesn't move the bring-up forward; smaller story |
| **FORTH on 1130** | **Historical fit + reference impls + tests slot-codegen against real user code** | **Doesn't validate framework against ISA #2 yet** |

FORTH-first does **not** preclude doing ISA #2 next: the frontend
isn't ISA-locked once it's emitting TIR. ISA #2 can run after
FORTH lands and we have two ISAs (1130, ISA #2) executing the same
FORTH source.

## 7. Open questions for the saga's decisions step

> **Resolved 2026-05-10 in saga step 1.** See
> [`forth-on-1130-decisions.md`](forth-on-1130-decisions.md) for
> the locked outcomes. The questions below are kept for context;
> the linked doc is the authoritative answer.

These got resolved at saga step 1; listed here so the context
survives:

- **Q1.** ITC vs DTC vs subroutine-threaded? Recommended: ITC for
  historical fidelity, plus a DTC fallback experiment if ITC's
  per-primitive overhead is intolerable on the 1130.
- **Q2.** Block I/O vs no block I/O? Recommended: defer; the
  pilot saga targets keyboard + console only.
- **Q3.** ASCII or EBCDIC source? Recommended: ASCII, with the
  EBCDIC retrofit as a separate saga (per
  `docs/character-encoding-plan.md`).
- **Q4.** Self-hosting goal? Recommended: not in saga scope.
  Reach "FORTH user code reaches the emulator end-to-end" first;
  self-hosting (`forth-on-forthish` style) is a follow-up.
- **Q5.** Which primitive set? Recommended: the historical 28
  for the kernel, plus a layered "extension set" for the
  REPL-side conveniences (`HERE`, `LATEST`, `WORDS`, `BYE`, `:`,
  `;`, `."`, etc.).

## 8. References

### Source survey (saga step 2 deliverables)

- [`sw-ibm1130-forth/docs/moore-1968-survey.md`](https://github.com/sw-comp-history/sw-ibm1130-forth/blob/main/docs/moore-1968-survey.md)
  -- prose findings from reading Moore's kernel + Claunch's
  notes: threading model, register usage, branching idioms,
  character-encoding pipeline, definition syntax, code-
  generation primitive set, asm directives observed.
- [`sw-ibm1130-forth/docs/moore-1968-primitives.md`](https://github.com/sw-comp-history/sw-ibm1130-forth/blob/main/docs/moore-1968-primitives.md)
  -- mechanical side-by-side table covering the kernel
  primitives, the in-FORTH code-generation primitives, the
  high-level FORTH word set, the directives, the conditional-
  branch mnemonics, and the shift sub-ops.

### The historical 1968 source

- [monsonite/1968-FORTH](https://github.com/monsonite/1968-FORTH) --
  the canonical recovered source. `FORTH68asm.txt` (645 lines of
  1130 asm), `FORTH68lst.txt` (235-line FORTH-level dump),
  `FORTH-68_notes.txt`, and Carl Claunch's PDF analysis.
- [Charles H. Moore (Wikipedia)](https://en.wikipedia.org/wiki/Charles_H._Moore)
- [Chuck Moore: The Invention of Forth (HOPL)](https://colorforth.github.io/HOPL.html)
- [Forth.com history page](https://www.forth.com/resources/forth-programming-language/)
- [Retrocomputing forum: First Forth sources, 1968](https://retrocomputingforum.com/t/first-forth-sources-12-pages-1968-for-ibm-1130/1243)
- [ForthHub discussion #63: 1130 FORTH restored](https://github.com/ForthHub/discussion/issues/63)
- [Rescue 1130 blog: 1130 FORTH restoration](http://rescue1130.blogspot.com/2018/03/historical-recreationrestoration-of.html)

### Local reference

- `~/github/sw-embed/sw-cor24-forth` -- 2600-line COR24 DTC FORTH
  kernel + REPL + tests, organised into three layered crates
  (`forth-from-forth`, `forth-in-forth`, `forth-on-forthish`).
  License: MIT (Mike Wright).
- `~/github/sw-embed/sw-cor24-forth/forth.s` -- the kernel,
  prologue at lines 1-50 documents the COR24 register allocation
  (r0=W, r1=RSP, r2=IP, sp=DSP) and the DTC NEXT shape.

### Saga-context cross-links

- `docs/decisions.md` Sec 8 (D3 pilot frontend) -- gets updated
  to point at this doc and resolve toward FORTH.
- `docs/plan.md` phase 3 -- updated to reference this doc.
- `docs/status.md` open-decisions section -- D3 is now
  *resolved* (FORTH); previous D6 (CDP1802 vs RISC-V for ISA #2)
  remains open.
- `docs/postmortem-1130-bringup.md` Sec 5 (slot codegen debt) --
  the FORTH saga is the first real test of whether slot codegen
  carries non-trivial user code; section's recommendation to
  defer the allocator until ISA #3 still stands.

## 10. Assembler features needed for the 1968 source

A real concern: our `sw-ibm1130-asm` accepts a small modern
syntax (24 mnemonics + ORG/EQU/DC/END + colon-suffixed labels +
hex/decimal/binary literals + I/L flags). The historical 1968 IBM
1130 Assembler Language is much richer. Reading the
`monsonite/1968-FORTH` `FORTH68asm.txt` (645 lines of historical
source) before any port will surface the full extension list, but
the obvious gaps are predictable:

### 10.1 Directives we don't have

| Directive | Purpose | Saga step to add |
| --------- | ------- | ---------------- |
| `BSS` / `BES` | Block-Storage Symbol / Block-End Symbol -- reserve N words, label points at start (BSS) or just past end (BES) | step 3 of the FORTH saga (kernel asm needs them) |
| `DEC` | Decimal constant (we have plain numeric literals; historical asm distinguished `DEC 5` from `HEX 0005`) | step 3 |
| `DSA` | Define Storage Address -- emits a 1- or 2-word address constant; analogous to our `DC SYMBOL` but with a different relocation mode | step 3 |
| `EBC` | Emit packed-EBCDIC string literal (Moore's 1968 source uses this; our character-encoding-plan captures the EBCDIC retrofit as a separate saga, so we may stub `EBC` as a synonym for our future `DC.STR` and revisit) | step 3 + character-encoding saga |
| `ENT` / `EXT` | Mark a label as the program's entry point / mark a label as external | step 3 |
| `LIBF` / `CALL` | Pseudo-ops emitting the library/non-library subroutine calls (LIBF emits a 1-word indexed BSI through XR3; CALL emits a 2-word direct BSI). See postmortem Sec 4 (LIBF emission listed as deferred). | LIBF in step 3 only if the kernel uses it; otherwise defer to a follow-on |
| `END` (with operand) | Currently we accept `END` (no operand); historical asm allows `END START_LABEL` to set the load-time entry point | step 3 -- one-line extension |
| `ISS` / `ILS` | Mark interrupt service / level subroutines | defer; FORTH on 1130 doesn't need interrupts |
| `ABS` / `RLD` | Relocation directives | defer; we don't have a real linker |

### 10.2 Expression syntax we don't have

Currently operands are a number or a single symbol. The 1968
source uses arithmetic expressions in operands (`SYM+1`, `*-2`,
`A-B`, etc.). Required for:

- Self-relative jumps and per-word skips inside the kernel.
- Inline length calculations in EBCDIC string literals.
- Computing offsets into dictionary entries.

**Saga-step impact:** literal expressions are already listed as a
deferred item in `docs/postmortem-1130-bringup.md` Sec 9 ("future
work"). The FORTH saga forces the issue. A small Pratt parser
(`+`, `-`, `*`, `/`, parens, `*` = current location counter) is
the right shape. Estimate: half-day of implementation, a day of
tests; lands as a step in the FORTH saga.

### 10.3 Source-layout features

The 1968 source is fixed-column: label cols 1-5, opcode cols 7-9,
operand cols 11-, comment after a tab or column 30. Our parser is
free-form (whitespace-driven). We can either:

- **(a)** Translate the historical source to free-form on the fly
  (small `forth68_to_asm.py` shim) and assemble the translated
  text. Lower risk; doesn't touch the assembler.
- **(b)** Add an optional fixed-column parsing mode to the
  assembler. Higher fidelity; bigger surface.

Recommended: (a). The 1968 source is a few hundred lines; a
one-shot translation script is faster than maintaining two parser
modes.

### 10.4 Mnemonic gaps

The 24-mnemonic spec covers the common 1130 instruction set but
omits a few that FORTH might want:

- `NOP` -- traditionally `SLA 0` (a no-op shift). The asm already
  parses `SLA 0` so this is just a doc note.
- Subroutine sub-ops of `SLA` / `SRA` (`SLT`, `SLC`, `SRT`) --
  encoded by the displacement bits per the spec, not separate
  opcodes. The asm currently parses `SLA N` where N is the count;
  if the kernel needs `SLT` we add a small operand-time
  recognizer that lights up the appropriate displacement bits.
- `BOSC` -- "branch out" variant of BSC for ISS exit. Not needed
  for FORTH (no interrupts in pilot scope), defer.

### 10.5 BSC mask field (Postmortem Sec 4)

**The big one.** The 1968 source uses `BSC` long form with
condition masks intensively, since FORTH branches through them
constantly. Our current ISA spec marks the long-form mask bits as
reserved-zero, so our `Instruction::Long` has no mask field, and
the asm parses `BSC L target, mask` but drops the mask.

This is the first downstream consumer where the BSC-mask gap
*actually bites*. The skip-and-jump idiom (postmortem Sec 4) works
but doubles the instruction count for every conditional branch --
intolerable in a FORTH kernel where threading overhead dominates.

**Decision deferred to saga step 1:** either close the gap in the
`-isa` spec (re-run the scaffolder for `sw-ibm1130-isa`, re-apply
the hand-written modules, update encode/decode/asm/emulator) before
starting the kernel, or live with the doubled branches. The gap-
closing is documented in postmortem Sec 4 with effort estimated
at "half-day, gated on a willingness to bump the shipped
`sw-ibm1130-isa` crate." The FORTH saga is exactly the willingness
event.

### 10.6 Saga step ordering implication

Given Sec 10.1-10.5, the FORTH saga's pre-kernel step list should
be:

```
forth-on-1130 (saga, REVISED)
  step 1  decisions          + lock asm-extension scope (this section)
  step 2  reference-cloning  pull monsonite/1968-FORTH
  step 3  bsc-mask-fix       close the BSC long-form mask gap in isa/asm/emulator
  step 4  asm-extensions     BSS, BES, DEC, EBC, DSA, ENT, EXT, expr parser, END operand
  step 5  kernel-asm         hand-write the FORTH kernel
  step 6  kernel-tests       primitives execute correctly
  step 7  parser             FORTH source -> token stream
  step 8  compiler           tokens -> TIR
  step 9  emit               TIR -> 1130 binaries
  step 10 repl
  step 11 end-to-end-demo
  step 12 postmortem
  step 13 status-update
```

13 steps rather than 11, ~5-6 weeks. Adding two infrastructure
steps (BSC fix, asm extensions) up front reduces churn during the
kernel work and produces a real assembler -- one that can take
historical 1130 source as input.

## 9. What to do now (concrete)

The current `foundation-and-1130-bringup` saga is closed. To act
on this plan:

1. Read this doc end-to-end before opening a new saga.
2. Start a new agentrail saga `forth-on-1130` with the 11-step
   skeleton in Sec 5 above.
3. Step 1 of that saga: pull `monsonite/1968-FORTH` into
   `~/github/sw-comp-history/`, read the kernel, lock in
   ITC/DTC choice (Q1 above) and source-encoding choice (Q3).
4. Step 2: clone-as-reference, read both kernels (Moore's and
   COR24), produce a side-by-side primitive table.
5. Continue per saga skeleton.

Until that saga starts, this doc plus `docs/plan.md` (updated)
captures the intent.
