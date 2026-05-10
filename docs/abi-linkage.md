# IBM 1130 Subroutine Linkage: Research Notes

> Status: research record, ASCII-only. Created 2026-05-09 during saga
> step 9 (`ibm1130-codegen`) after the user redirected to verify the
> step-8 invented ABI against historical practice. The findings here
> drive the ABI revision in
> `sw-comp-history/sw-ibm1130-target/docs/abi.md`.

## 1. Why this document exists

Step 8 (`ibm1130-target`) shipped an ABI that was **invented**: I picked
XR3 as a frame pointer and XR2 as a logical stack pointer based on
abstract reasoning. The user pointed out that bitsavers has actual IBM
1130 software listings and asked for a reality check before step 9
codegen poured concrete on those choices.

The reality check found the invented ABI was **wrong on XR3** in a way
that would silently break interop with any IBM-supplied subroutine.
This document captures what 1130 software actually did so future
sessions don't relitigate it.

## 2. The three linkage mechanisms

The 1130 instruction `BSI` (Branch and Store IAR) is the universal
subroutine call. On top of `BSI`, IBM software used three different
linkage idioms that codegen needs to know about.

### 2.1 CALL linkage (Type III / IV "non-relocatable" subprograms)

Two-word call. Subroutine entry word stores the return address.

```
       BSI   L NAME       ; long form; BSI writes return addr to NAME
       DC    PARAM1       ; in-line parameter (1 word per param)
       DC    PARAM2
NEXT:  ...                ; subroutine returns control here

NAME:  DC    *-*          ; entry word; BSI writes return addr here
       ...                ; body uses PARAM1 = NAME+1, etc
       BSC   I NAME       ; return: indirect branch through entry word
```

`BSC I NAME` reads the return-address word at `NAME`, adds the
parameter count to it (callee-known constant), and jumps. Subroutines
return control to the instruction *after* the last parameter, not
immediately after the BSI.

**Source**: ibm1130.net "Programming Tips and Techniques"; Wikipedia
on IBM 1130.

### 2.2 LIBF linkage (Type I / II library subprograms)

One-word call through a transfer vector based at XR3.

```
       BSI   3 disp       ; short form, tag=3, disp = vector entry index
```

The loader builds a per-program transfer vector. Each vector entry is
three words:

```
       DC    *-*          ; return-address slot (BSI writes here)
       B     L NAME       ; long branch into the routine body
```

LIBF saves space when a routine is called many times (one word per
call instead of two for CALL). The cost is the extra vector entry and
the requirement that XR3 hold the vector base for the lifetime of the
program.

**Source**: ibm1130.net; Wikipedia ("Library routines are addressed
through index register XR3").

### 2.3 ISS basic calling sequence (Subroutine Library, p. 6)

I/O subroutines use a stylised LIBF with a fixed parameter shape:

```
       LIBF  NAME             ; e.g. LIBF CARD1
       DC    Control parameter ; 4 hex digits: function, device id, ...
       DC    I/O area          ; address of caller's buffer
       DC    Error subroutine  ; address of error handler (optional)
```

"Unless otherwise specified, the subroutine returns control to the
instruction immediately following the last parameter."

ISSs internally save and restore ACC, Extension, all index registers,
Carry, and Overflow (Subroutine Library, p. 2 "ISS Operation").

**Source**: IBM 1130 Subroutine Library, C26-5929-4, p. 6 "Basic ISS
Calling Sequence".

## 3. Register conventions

Synthesised from C26-5929-4 (subroutine library) and FORTRAN compiler
behaviour.

| Reg  | Convention                                     | Source                              |
| ---- | ---------------------------------------------- | ----------------------------------- |
| ACC  | caller-saved; arg/return for scalars           | universally trashed by ISS/LIBF     |
| EXT  | caller-saved; high half of 32-bit value        | trashed alongside ACC               |
| XR1  | caller-saved; FORTRAN parameter pointer        | DM1 ILSs save+restore (p. 2)        |
| XR2  | **unused by FORTRAN; available to user**       | DM2 ILSs additionally save (p. 2)   |
| XR3  | **reserved as LIBF transfer-vector base**      | "addressed through XR3" (Wikipedia) |
| IAR  | hardware program counter                       | written by BSI                      |

ISSs internally preserve ACC, Extension, all XRs, Carry, Overflow
(C26-5929-4 p. 2). ILSs save ACC, Extension, XR1, Carry, Overflow on
DM1; DM2 ILSs additionally save XR2 (p. 2). Critically, **XR3 is
never modified by ILS/ISS housekeeping** -- it is global state set up
by the loader and assumed stable by every LIBF caller.

The user's error subroutine (when one is supplied via the third
parameter of an ISS call) returns via `BSC I` (not `BOSC`) so the
interrupt level stays on; this is a quirk of the error-handling path
not relevant to general codegen.

## 4. FORTRAN-specific details (not directly load-bearing for our
codegen, but useful context)

- The FORTRAN compiler builds a per-subprogram parameter table and
  emits a call to a runtime helper called `SUBIN` as the first
  instruction in the routine body. `SUBIN` patches the address fields
  of every parameter reference at runtime to the caller-supplied
  argument addresses.
- Parameters can be coded in-line after the BSI **or** placed in XR1
  and XR2; the compiler picks based on the call site.
- This level of runtime self-modification is well beyond what our
  toolchain needs to emit, but it explains why the FORTRAN library
  routines are so flexible about their parameter shapes.

**Source**: ibm1130.net; matches descriptions in
C20-1642-0_1130_FORTRAN_Programming_Techniques.

## 5. Implications for our ABI

The invented step-8 ABI gets two things wrong:

1. **XR3 cannot be a frame pointer.** XR3 is the LIBF transfer-vector
   base; if our codegen ever wants to interoperate with any IBM
   library subroutine (or with FORTRAN code, or with the disk monitor
   I/O subroutines), XR3 must be left alone for the lifetime of the
   program. The cleanest way to enforce this in the trait surface is
   to put XR3 in `RegisterClasses::reserved()` and never name it
   anywhere else.

2. **There is no separate SP.** The 1130 has no hardware stack pointer
   and no historical software convention defined one in a way the
   compiler community converged on. FORTRAN simply did not have a
   stack -- it used a fixed activation record per subprogram, no
   recursion. Our ABI should follow suit: a single "frame base"
   register (XR2 is the only sensible choice -- ACC/EXT are arithmetic,
   XR1 is the param pointer, XR3 is reserved) holds the activation
   record's base, and frames are fixed size at compile time.

Corrected role assignment:

| Reg  | Role                                       | Saved by  |
| ---- | ------------------------------------------ | --------- |
| ACC  | first scalar arg + scalar return           | caller    |
| EXT  | high word of 32-bit pair                   | caller    |
| XR1  | scratch + parameter-list pointer in callee | caller    |
| XR2  | frame base (locals, spills)                | callee    |
| XR3  | LIBF transfer-vector base -- **RESERVED**  | loader    |
| IAR  | PC                                         | hardware  |

The `CallingConvention` trait requires both `stack_pointer` and
`frame_pointer`. With no separate SP in the historical idiom, the
clean encoding is:

- `stack_pointer() = XR2` (the frame base, used as both)
- `frame_pointer() = None` (no distinct FP; activation records are
  fixed-size and addressed by XR2 alone)

This keeps the trait honest: codegen can't accidentally use a
different register for SP vs FP because the ABI says they coincide.

## 6. What our codegen should emit

For now (step 9 codegen scope), the relevant calling-sequence shape
for *outgoing* calls from our generated code is the **CALL linkage**
(Section 2.1). LIBF requires a transfer-vector setup pass that
belongs to a linker step we have not yet built; CALL is self-contained
and is what step-9 snapshot tests will exercise.

If and when our toolchain wants to call IBM library subprograms
directly, we will revisit and add LIBF emission. For greenfield code
paths this is unnecessary.

## 7. Open question for the postmortem (step 12)

The C26-5929 manual's pages 5 and 8 describe a more nuanced
"Test function" call pattern (the `LIBF NAME / DC ctrl / OP / OP`
shape that branches to LIBF+2 vs LIBF+3 based on completion). This
is an I/O-subsystem idiom and not part of the general ABI; recording
it here only so a future self knows where to look when implementing
async I/O or device polling in the emulator (step 11).

## 8. Sources

- [IBM 1130 Subroutine Library, C26-5929-4 (1966)](http://bitsavers.org/pdf/ibm/1130/subroutines/C26-5929-4_1130_Subroutine_Library_1966.pdf)
  -- pages v, 1-9 (Introduction, Interrupt Service Subroutines,
  ILS/ISS Operation, Basic ISS Calling Sequence, Core Storage
  Locations).
- [IBM 1130 FORTRAN Programming Techniques, C20-1642-0](http://bitsavers.org/pdf/ibm/1130/lang/C20-1642-0_1130_FORTRAN_Programming_Techniques.pdf)
  -- referenced for FORTRAN runtime conventions; not directly
  excerpted here.
- [ibm1130.net "Programming Tips and Techniques"](https://ibm1130.net/DM2/ProgTipsAndTechniques.html)
  -- complementary explanation of CALL vs LIBF.
- [Wikipedia: IBM 1130](https://en.wikipedia.org/wiki/IBM_1130)
  -- summary of register conventions; cross-checks against the
  manuals.
- [bitsavers IBM 1130 subroutines directory](http://bitsavers.org/pdf/ibm/1130/subroutines/)
  -- the canonical archive for all subroutine-library manuals.
