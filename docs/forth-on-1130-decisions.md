# FORTH-on-1130 Saga: Forced Deltas from Moore's 1968 Design

> Status: locked-deltas record for the `forth-on-1130` agentrail
> saga. Written 2026-05-10 as step 1.
>
> **Framing (important).** We are not designing a FORTH and we
> are not making architectural decisions. We are recreating
> Charles H. Moore's 1968 FORTH for the IBM 1130 -- *that* FORTH,
> not any FORTH, not a new FORTH. The architecture, threading
> model, primitive set, dictionary structure, identifier choices,
> and word semantics all live in Moore's source.
> See [`historical/forth68/NOTICE`](https://github.com/sw-comp-history/sw-ibm1130-forth/blob/main/historical/forth68/NOTICE)
> for the redistribution provenance.
>
> What this doc records: the **forced deltas** between Moore's
> environment and ours -- places where our toolchain literally
> can't run Moore's source as-is, and we have to do something. Each
> delta is justified by an infrastructure constraint, not a design
> preference, and includes a path back to alignment with Moore.
>
> Saga steps treat the deltas below as fixed. Revising any of them
> requires an explicit commit that updates this doc and explains
> the trigger.
>
> ASCII-only.

## 1. What we honor (Moore's, unchanged)

These are not decisions; they are facts inherited from Moore's
1968 source. The translation step (saga step 5) preserves them
verbatim.

- **Threading model: indirect-threaded code (ITC).** Moore's
  choice in 1968. We use ITC because the kernel is his.
- **Primitive set: the 28 historical primitives.** Same names,
  same semantics, same order. Translation does not add or
  remove.
- **Dictionary structure.** Moore's header layout (link, flags,
  name, CFA) preserved.
- **Register usage at the kernel level: XR1 = workspace pointer
  (W), XR2 = data stack pointer (DSP).** Carried over directly.
- **NEXT shape, primitive ABI, return-stack discipline.** All
  Moore's; we copy.
- **Identifier names.** Every label, every directive, every
  comment glyph: preserved through the translation. (Comment
  *character* changes; see Sec 3.)
- **Self-hosting status: not self-hosting.** Moore's 1968 system
  was a kernel + a small block of FORTH source loaded from disk;
  it didn't rebuild itself. Our recreation matches.

There are no decisions to make about any of the above. The saga
honors them.

## 2. Forced deltas (this is what's actually decided)

The cases below are **infrastructure-forced**, not preferential.
Each is a place where our toolchain genuinely cannot run Moore's
source as-is, and we patch the difference.

### 2.1 ASCII source (Moore had EBCDIC)

**Force:** our toolchain has no EBCDIC support yet.

Moore's source uses a custom EBCDIC variant -- packed-byte
character storage, alphabetical-sort-friendly, neither
ASCII-compatible nor canonical CP-037. Our `sw-ibm1130-asm`
parses ASCII; our codegen has no `DC.STR`-style packed-string
directive yet; our emulator's XIO console handler types raw
bytes (treats them as ASCII for display).

**Delta:** during the translation (saga step 5) each EBCDIC
string literal is rewritten as a numeric `DC` sequence carrying
the *original byte values* (preserving Moore's alphabetisation
through the same byte ordering); the literals' ASCII renderings
appear only in the translated comments. The translation log
(`historical/forth68/TRANSLATION-LOG.md`) records each rewrite
so a future EBCDIC saga can map them back faithfully.

**Path back:** the character-encoding saga
(`docs/character-encoding-plan.md` phase A + B) lands EBCDIC
tables and a `DC.STR` directive. When that saga is done, the
1968 FORTH translation can be re-run to keep the literals
EBCDIC-native, matching Moore's environment exactly. This delta
is therefore **temporary and well-bounded**.

### 2.2 XR3 reservation (Moore used XR3 freely)

**Force:** our ABI reserves XR3 as the LIBF transfer-vector
base. See `docs/abi-linkage.md` and postmortem Sec 3.

Moore's 1968 FORTH was a closed system; he had no IBM library
interop concerns and used XR3 as a kernel-internal auxiliary
register. We can't, because any code that scribbles XR3 silently
breaks interop with FORTRAN, the Disk Monitor System I/O
subroutines, and any IBM-supplied subprogram. The XR3
reservation is enforced by `sw-ibm1130-target`'s
`RegisterClasses::reserved()` and `CallingConvention` impls and
by defensive smoke tests there.

**Delta:** the translation (saga step 5) replaces every XR3
reference in Moore's kernel with a memory-cell load/store. The
cell is named in the data area (likely as `IP_CELL` or similar)
and accessed via direct LD/STO. Per-primitive overhead: an
extra LD/STO pair in NEXT and in primitives that touch the
fourth pointer.

**Path back:** none. This delta is **permanent** for as long as
we want library-interop compatibility. A future "closed-system
mode" saga that explicitly opted out of all IBM library interop
could revisit, but that's not on any current roadmap.

### 2.3 Free-form source columns (Moore's was fixed-column)

**Force:** our `sw-ibm1130-asm` parses free-form source.

Moore's 1968 source is fixed-column: label cols 1-5, opcode
cols 7-9, operand cols 11-, comment after col 30. Our parser is
whitespace-driven; column positions are irrelevant.

**Delta:** the translation reformats source to free-form,
preserves all text content and order, and adds colon suffixes
to label definitions (our parser requires `FOO:` not `FOO`).
Mechanical; not logged per-line.

**Path back:** none planned. We picked free-form for the asm
in saga step 10 of the previous saga; reverting would not help
anyone. This delta is **permanent and trivial**.

### 2.4 Comment character: `;` instead of `*`

**Force:** our `sw-ibm1130-asm` recognises `;` as the comment
delimiter, not the historical 1130 asm's `*`.

**Delta:** mechanical search-and-replace in the translation.
Not logged per-line.

**Path back:** none planned; `;` is conventional in modern asm.
**Permanent and trivial.**

### 2.5 BSC long-form condition mask field

**Force:** our `sw-ibm1130-isa` spec at saga step 7 of the
previous saga marked the BSC long-form mask bits as
reserved-zero. Postmortem Sec 4.

Moore's 1968 source uses `BSC` long form with condition masks
intensively (FORTH branches through them constantly). Our asm
parses `BSC L target, mask` but currently drops the mask; our
emulator treats `BSC L` as unconditional.

**Delta:** saga step 3 closes the gap before any translation
work happens. Procedurally:

1. Update `gen-isa/docs/spec-examples/ibm1130.toml` to expose
   the long-form's currently-reserved 7 bits as a per-opcode-
   conditional `mask` field.
2. Re-run `gen-isa scaffold --spec` for `sw-ibm1130-isa`;
   re-apply the hand-written modules (`addr_mode.rs`,
   `branch_cond.rs`).
3. Update `Instruction::Long` (or add a per-format wrapper) to
   carry the mask.
4. Update `sw-ibm1130-asm`'s `encode.rs` to populate the mask.
5. Update `sw-ibm1130-emulator`'s `exec_bsc` to read the mask
   from its proper home.
6. Add tests covering each masked-condition case.

After step 3 is complete, the translation in step 5 emits
masked BSC long form directly, matching Moore's source 1:1.

**Path back:** N/A; this delta closes itself. After saga step
3, this row goes away. We will note in the postmortem that we
should've shipped the mask field in the original spec.

### 2.6 Asm directives Moore uses but we don't have

**Force:** our `sw-ibm1130-asm` ships `ORG`, `EQU`, `DC`, `END`
only. Historical 1130 asm is richer.

The exact list is determined by reading Moore's source in saga
step 2. The plan-doc Sec 10 catalogues likely candidates;
saga step 1 (this doc) commits to which subset is **in scope**
for saga step 4:

| Directive | In scope (saga step 4)? | Reason |
| --------- | ----------------------- | ------ |
| `BSS`, `BES` | Yes | Block-storage; data-area declarations. |
| `DEC`, `HEX` | Yes (cheap) | Typed numeric constants. |
| `DSA` | Yes | Address constant; near-equivalent to our `DC SYMBOL`. |
| `END LABEL` | Yes (one-line) | Entry-point operand. |
| `EBC` | No (stub) | Packed-EBCDIC literals; deferred to encoding saga (Sec 2.1 above). Translation rewrites each `EBC` as numeric `DC` sequence per the EBCDIC delta. |
| `LIBF`, `CALL` | No | Library-call pseudo-ops. Moore's kernel is closed-system; no LIBF emission. Defer to a future I/O-interop saga. |
| `ENT`, `EXT` | No | Entry/external markers; need a real linker first. |
| `ISS`, `ILS` | No | Interrupt-subroutine markers; FORTH on 1130 is non-interrupt-driven in pilot scope. |
| `ABS`, `RLD` | No | Relocation; need a real linker first. |

Plus literal expressions in operands (`SYM+1`, `*-2`, parens): in
scope. Small Pratt parser; ~half-day. Required for self-relative
jumps inside the kernel.

If reading Moore's source surfaces a directive not in either
column above, this section gets a follow-up commit before saga
step 4 starts.

### 2.7 Block I/O semantics

**Force:** our emulator stubs all non-console XIO. No disk
device.

If Moore's source uses block I/O (BLOCK/BUFFER/UPDATE/FLUSH),
those primitives need device support that we don't have. Saga
step 2 (reading the source) confirms whether this delta is
load-bearing.

**Delta (conditional on what step 2 finds):**

- **If the kernel doesn't reference block words:** no delta.
  Move on.
- **If the kernel does reference block words:** translate them
  with no-op primitive bodies and a clear comment + log entry
  pointing at the deferred I/O-saga. Programs that try to call
  them at runtime get a no-op (data stack untouched, no harm
  done); programs that don't touch them work.

**Path back:** the future "1130 peripheral simulation" saga
brings up disk + paper tape + card reader + the rest of XIO. At
that point the no-op stubs are filled in.

## 3. What's NOT a decision in this saga

For clarity, listing things that might look like decisions but
aren't:

- "ITC vs DTC vs subroutine-threaded" -- not a decision.
  Moore picked ITC; we use ITC.
- "Self-hosting" -- not a decision. Moore wasn't self-hosting;
  we're not self-hosting.
- "Primitive set / which primitives to ship" -- not a decision.
  Moore's 28; we use 28.
- "Direct port vs clean-room rebuild" -- not a decision.
  Translation, per `historical/forth68/` policy.

If the user encounters a question that feels like one of these,
the answer is "look at Moore's source." If it isn't there, it
isn't a question this saga answers.

## 4. Scope summary (one paragraph)

This saga recreates Moore's 1968 FORTH for the IBM 1130 by
translating his source into our `sw-ibm1130-asm` syntax,
shipping it under his May-2020 public-posting permission with
attribution. Our toolchain has six infrastructure deltas from
Moore's environment: ASCII (not EBCDIC, temporary), XR3
reservation (permanent), free-form columns (trivial), `;`
comments (trivial), BSC mask-field absence (closed in saga
step 3), and missing directives (added in saga step 4). After
those deltas are applied, the translation produces a kernel
that runs end-to-end on `sw-ibm1130-emulator` and prints to the
captured 1054 console -- "Moore's 1968 FORTH on our toolchain",
not "a FORTH we built that resembles Moore's".

## 5. Cross-links

- [`forth-on-1130-plan.md`](forth-on-1130-plan.md) -- saga plan
  (Sec 7 redirects here for the locked answers).
- [`postmortem-1130-bringup.md`](postmortem-1130-bringup.md) Sec
  4 (BSC mask gap) and Sec 9 (deferred-feature catalogue) --
  context for delta 2.5.
- [`character-encoding-plan.md`](character-encoding-plan.md) --
  the encoding saga that closes delta 2.1.
- [`abi-linkage.md`](abi-linkage.md) -- why XR3 is reserved
  (delta 2.2).
- `~/github/sw-comp-history/sw-ibm1130-forth/historical/forth68/`
  -- redistribution destination + policy + translation log.
