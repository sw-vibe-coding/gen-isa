# FORTH-on-1130 Saga: Locked Decisions

> Status: locked decisions for the `forth-on-1130` agentrail saga.
> Written 2026-05-10 as step 1 of the saga. Resolves the open
> sub-questions in
> [`forth-on-1130-plan.md`](forth-on-1130-plan.md) Sec 7 and
> picks scope for the assembler-extension work in Sec 10.
>
> Subsequent saga steps treat the choices below as fixed. Revising
> any of them is in-scope only with an explicit commit that updates
> this doc and explains the trigger.
>
> ASCII-only.

## 1. Q1 -- Threading model: ITC vs DTC vs subroutine-threaded

**Locked: indirect-threaded code (ITC).**

Rationale:

- Moore's 1968 1130 FORTH is ITC. Recreating his system on its
  native machine is the saga's whole point; matching the
  threading model is non-negotiable for "this is Moore's FORTH".
- ITC is more code-compact than DTC on a 16K-word 1130. Each
  threaded reference is one word (the address of the word's CFA),
  and CFA dereference happens in NEXT. DTC stores executable code
  inline, which is faster but uses more memory. The 1130's 16K
  word ceiling makes the trade favour ITC.
- The cor24-forth reference is DTC, but for COR24 not 1130. We
  consult its structure (dictionary headers, NEXT shape, REPL
  loop) without porting the threading model.

Subroutine-threaded (each word a real BSI to its primitive)
considered and rejected: BSI's 2-word long form is heavier than
ITC's 1-word reference, and Moore did not use it.

## 2. Q2 -- Block I/O

**Locked: deferred. Pilot saga is keyboard + console only.**

Rationale:

- 1130 block I/O via 2310/2311 disk is genuine work: device
  semantics, sector layout, allocation/free tables, the BLOCK /
  BUFFER / UPDATE / FLUSH word family. None of it is on the
  saga's critical path for "Moore's FORTH runs end-to-end".
- The emulator currently stubs all non-console XIO; bringing up
  disk would also pull in interrupt handling, which is out of
  scope (postmortem Sec 7).
- A future "1130 peripheral simulation" saga is the right home
  for block I/O; the FORTH word set will need to be extended at
  that point with the standard block words.

Source code referencing block I/O during translation: the kernel
is the load-bearing piece; if Moore's source contains BLOCK /
BUFFER words we either translate them as no-ops with a clear log
entry, or skip them with a forward-reference comment pointing at
the deferred saga.

## 3. Q3 -- Source encoding: ASCII or EBCDIC

**Locked: ASCII for the saga; EBCDIC retrofit deferred.**

Rationale:

- `gen-isa/docs/character-encoding-plan.md` already captures the
  intended EBCDIC retrofit as a 4-phase future saga. Anchoring
  this saga on ASCII keeps it inside one cohesive scope and
  doesn't double-couple it to the encoding work.
- The asm `DC.STR` directive that would produce packed-EBCDIC
  literals is a phase-A deliverable of the encoding saga, not
  this one. Until then, FORTH source files use ASCII; FORTH
  string literals (`."`) emit a sequence of numeric `DC` words
  carrying ASCII bytes.
- Moore's 1968 source uses a *custom* EBCDIC variant (per the
  upstream notes), neither plain ASCII nor canonical CP-037. The
  translation step will rewrite each EBCDIC string literal as
  numeric `DC` initialisers preserving the original byte values
  (per `historical/forth68/TRANSLATION-LOG.md` policy), with a
  log entry per literal so the eventual EBCDIC saga can map them
  back.

The character-encoding postmortem item (deferred-feature) stays
deferred; this saga does not close it.

## 4. Q4 -- Self-hosting goal

**Locked: not in saga scope.**

Rationale:

- The saga's exit criterion is "FORTH user code reaches the
  emulator end-to-end". A `.fth` file compiles, runs on the
  emulator, and types output on the 1054 console.
- Self-hosting (rebuilding the kernel itself from FORTH source,
  cor24-forth's `forth-from-forth` / `forth-in-forth` /
  `forth-on-forthish` layered approach) is a substantial
  follow-on. The cor24-forth reference shows it's tractable; we
  defer to a future saga.
- A modest intermediate -- using FORTH source to *extend* the
  kernel (define new colon words at REPL time) -- IS in scope.
  That's the standard FORTH user model and the REPL step (saga
  step 10) delivers it.

If the saga delivers "extend the kernel via FORTH at runtime"
plus the runnable Moore-1968 demo, self-hosting can wait.

## 5. Q5 -- Primitive set

**Locked: 28 historical primitives in the kernel; layered
"extension set" for REPL conveniences.**

Rationale:

- Kernel layer (`historical/forth68/kernel.asm`, translated from
  Moore's `FORTH68asm.txt`): the 28 primitives Moore specified.
  Translation step (saga step 5) preserves identifiers and order;
  any cuts get logged in `TRANSLATION-LOG.md`.
- Extension layer (defined in `core.fth` or similar, loaded by
  the REPL at startup): the conveniences a usable FORTH needs --
  `HERE`, `LATEST`, `WORDS`, `BYE`, `:`, `;`, `."`, `STATE`,
  `BASE`, `,`, `C,`, `ALLOT`, etc. Authored in FORTH using the
  kernel's primitives. Mirrors the cor24-forth pattern.

The exact extension-set list is decided per-as-needed during the
parser/compiler work (saga steps 7-8). What's locked here: there
ARE two layers, and the kernel layer is exactly Moore's 28.

## 6. Asm-extension scope (refines plan.md Sec 10)

The saga's asm-extensions step (saga step 4) ships ONLY what the
kernel translation needs. Each item is in-scope only if Moore's
source uses it; out-of-scope items get punted to a separate
"asm catch-up" saga.

### In scope (likely; confirmed by reading Moore's source in step 2)

- **BSC long-form condition mask field.** Postmortem Sec 4. The
  most consequential gap; FORTH branches through masked BSC
  long form constantly, and the skip-and-jump workaround
  doubles instruction count. Closing it requires re-running
  `gen-isa scaffold` for `sw-ibm1130-isa` (re-applying the
  hand-written modules) and updating asm + emulator. **In
  scope as saga step 3.**
- **Literal expressions in operands.** `SYM+1`, `*-2`, `A-B`.
  Required for self-relative jumps and inline length math
  inside the kernel. Small Pratt parser; ~half-day. **In scope
  as part of saga step 4.**
- **`BSS` / `BES`.** Block storage + block-end symbol. The
  kernel's data area declarations almost certainly use these.
  **In scope as part of saga step 4.**
- **`DEC` / `HEX` typed constants.** Currently we accept plain
  numeric literals; historical asm distinguishes
  `DEC 5`/`HEX 0005`. Either accept both as synonyms for our
  current parser or no-op the directive line. **In scope as
  part of saga step 4 (cheap to add).**
- **`DSA`** (define storage address). Almost equivalent to our
  `DC SYMBOL`. May need a small relocation-mode tweak. **In
  scope as part of saga step 4.**
- **`END LABEL`.** END with an entry-point operand. **In scope
  as part of saga step 4 (one-line extension).**

### Out of scope (deferred to later sagas; not on this saga's path)

- **`EBC` packed-EBCDIC literals.** Stubbed during the
  translation step (per
  `historical/forth68/TRANSLATION-LOG.md`); proper support
  belongs to the character-encoding saga (`character-encoding-
  plan.md` phase B).
- **`LIBF` / `CALL` pseudo-ops.** Historical 1130 subroutine-
  linkage emission. Moore's kernel does not use LIBF for its
  primitives (FORTH is a closed system; no IBM library
  interop). Defer entirely.
- **`ENT` / `EXT`** (entry-point / external markers). Not
  needed without a real linker. Defer.
- **`ISS` / `ILS`** (interrupt subroutine markers). FORTH on
  1130 doesn't use interrupts in pilot scope. Defer.
- **`ABS` / `RLD`** (relocation directives). No real linker.
  Defer.
- **Fixed-column source layout.** Per plan.md Sec 10.3, we
  translate Moore's source to free-form offline. The asm stays
  free-form-only.

If reading Moore's source in saga step 2 surfaces a directive
not on either list above, it lands here as a delta in a follow-up
commit on this doc.

## 7. XR3 deviation from Moore

**Locked: XR3 stays reserved as the LIBF transfer-vector base.
The kernel uses a memory cell where Moore used XR3.**

Rationale:

- Per `gen-isa/docs/abi-linkage.md` and the postmortem Sec 3:
  XR3 is the LIBF transfer-vector base for the IBM 1130 system
  library. Anything that scribbles XR3 silently breaks interop
  with FORTRAN, the Disk Monitor System, or any IBM-supplied
  subprogram.
- Moore's 1968 FORTH was a closed system. He had no library
  interop concerns; he could and did use XR3 for the kernel's
  fourth pointer (per the source's preamble notes).
- We cannot. So our translation reserves a memory cell as the
  fourth pointer (call it `IP` for instruction pointer / return
  stack), and primitive code path-loads from / stores to it
  rather than referencing XR3.

The translation log records every place XR3 is replaced by the
memory-cell idiom. Per-primitive overhead: an extra LD/STO pair
in NEXT and in primitives that touch the IP. Acceptable cost for
ABI compliance.

If a future saga ever decides to relax the XR3 reservation (e.g.
explicitly opt out of all IBM library interop for closed-system
programs), this constraint loosens; until then it's a hard
invariant.

## 8. Translation scope (reaffirms `historical/forth68/` policy)

**Locked: syntactic retargeting only. Step 5 of this saga is the
sole place where Moore's source enters our repo.**

Rationale (re-stating the policy in
`historical/forth68/TRANSLATION-LOG.md` so this saga is honest):

- Algorithm preserved verbatim. Identifier names preserved.
  Primitive count preserved (28). Dictionary structure
  preserved.
- Transformations limited to: column layout (fixed -> free),
  comment character (`*` -> `;`), label suffix (`FOO` ->
  `FOO:`), directive renames (per Sec 6 above), EBCDIC
  literals -> numeric DC (per Sec 3 above), BSC mask encoding
  (depends on saga step 3 outcome), and the XR3 -> memory-cell
  rewrite (Sec 7 above).
- Every non-mechanical transformation is logged in
  `historical/forth68/TRANSLATION-LOG.md`.
- The kernel after translation is a derivative work under
  Moore's May-2020 public-posting permission (per
  `historical/forth68/NOTICE`), not a clean-room rebuild.

If a later saga decides to also produce a clean-room kernel
alongside the translated one (good for "FORTH on 1130 from a
fresh design" comparisons), it would live at
`src/kernel-clean-room.asm` with its own provenance doc. That is
**not** in this saga's scope.

## 9. Cross-links

- [`forth-on-1130-plan.md`](forth-on-1130-plan.md) -- saga plan
  (this doc resolves Sec 7's open questions).
- `~/github/sw-comp-history/sw-ibm1130-forth/historical/forth68/`
  -- the redistribution destination + policy.
- [`postmortem-1130-bringup.md`](postmortem-1130-bringup.md) Sec
  4 (BSC mask gap) and Sec 9 (deferred-feature catalogue) --
  context for saga step 3.
- [`character-encoding-plan.md`](character-encoding-plan.md) --
  why ASCII for now; what the EBCDIC retrofit covers.
- [`abi-linkage.md`](abi-linkage.md) -- why XR3 is reserved.
