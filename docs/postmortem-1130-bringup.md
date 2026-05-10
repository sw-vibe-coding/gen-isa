# Postmortem: IBM 1130 Bring-up (Saga `foundation-and-1130-bringup`)

> Status: postmortem record. Written 2026-05-10 at the close of the
> 1130-bring-up saga, while the friction is fresh. Cross-references
> point at the per-step commits and the per-crate code; this doc is
> the index, not a replacement for reading them.

ASCII-only.

## 1. Summary

The saga delivered the IBM 1130 quintet (`-isa`, `-target`,
`-codegen`, `-asm`, `-emulator`) under `sw-comp-history`, plus the
hybrid scaffolder (`gen-isa`) and the layered framework crates
(`sw-langtools`). Every saga exit criterion is met: the framework
crates compile, the scaffolder produces valid per-ISA quintets, the
1130 round-trips through every layer, the assembler round-trips, and
the emulator runs five curated programs to halt.

Headline shifts during bring-up:

- **Step 8 ABI was revised mid-step-9** after research against
  bitsavers showed the invented ABI conflicted with historical 1130
  conventions (XR3 is the LIBF transfer-vector base, not a frame
  pointer). See `docs/abi-linkage.md` for the research and Section 3
  below for the implications.
- **BSC long-form condition mask** lives in the historical 1130
  first-word reserved bits, which our ISA spec marked
  reserved-zero. The asm parses the `mask` operand but currently
  drops it; the emulator treats `BSC L` as unconditional. Section 4
  below details the gap and the conditional-branch idiom we adopted
  to work around it.
- **No register allocator and no linker.** Codegen uses a naive
  "ValueId N -> memory slot N" model and emits placeholder addresses
  with fixups; nothing wires the fixups up to a linker yet.
  Section 5 below.
- **Saga added a +1 bonus deliverable mid-flight:** demo programs
  illustrating math/conditions/loops/strings/hello-world. The
  hello-world demo prints "HELLO, WORLD!" via XIO to a captured
  console buffer (1054/Selectric area), which led to a small
  emulator extension. Section 7.

The trait surfaces in `sw-langtools` did not change during 1130
bring-up. That's a useful data point about the design's quality but
a deceptive one: see Section 2 for caveats before relying on it.

## 2. Trait-surface evolution in `sw-langtools`

The expected outcome of this saga (per `docs/porting-guide.md` Sec
11) was that the first ISA would force several trait-surface
revisions. **It did not.** No struct or trait method in
`sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`, or
`sw-codegen-core` was changed during steps 7-11.

Why this is real:

- The trait surface stayed small and shape-only. Each crate's
  `lib.rs` is under 100 lines; there is little surface to break.
- The 1130's hardest peculiarities (word-addressed memory,
  accumulator-only arithmetic, no hardware stack pointer, BSI
  store-IAR call shape) were already on the radar from
  `architecture.md` Sec 6.1's disruption-axis ranking.
- We accepted impedance at the **codegen** layer rather than
  pushing back into framework traits: the slot model
  (Section 5), the BSC kludge (Section 4), and the dropped LIBF
  mask (Section 4) are all codegen/asm/emulator workarounds.

Why this is deceptive:

- The 1130 didn't exercise several axes. It has no FPU; we never
  asked the framework to model FP regs. It has no SIMD or vectors.
  It uses single-word instructions (ish); we never tested
  multi-word formats like S/370 SS.
- Some "didn't change" results from punting: we have no
  `LinkerHook` trait in `sw-codegen-core` and no symbol-table type
  in `sw-tir` because nothing produces real linkable objects yet.
  When a real linker arrives, expect framework changes.
- The `Architecture::decode` signature returns `(Instruction,
  bytes_consumed)`. That worked for the 1130's two-format encoding
  but feels too narrow for variable-length ISAs that need decoder
  state (S/370's MVCL, RISC-V compressed). The first ISA that
  needs decoder state will force a change.

**Postmortem stance:** count the trait surface as **load-bearing
but unproven beyond the 1130 disruption profile.** ISA #2 (CDP1802
or RISC-V I32, depending on `decisions.md` D6) is what tells us
whether the design generalises.

## 3. ABI revision (mid-step-9)

The step-8 invented ABI made two errors:

1. **Used XR3 as a frame pointer.** XR3 is reserved by the
   historical 1130 standard library as the LIBF transfer-vector
   base; the loader sets it once at program start and every library
   subroutine assumes it stable for the program lifetime. Any
   codegen that scribbles XR3 silently breaks interop with FORTRAN,
   the Disk Monitor System I/O subroutines, and any
   IBM-supplied subprogram.
2. **Defined a separate stack pointer (XR2) and frame pointer
   (XR3).** Real 1130 software uses fixed-size activation records
   addressed by a single base register; recursion was handled by
   convention or not at all. Our two-register model added complexity
   the architecture didn't need.

The fix (committed on `sw-ibm1130-target` after research):

- XR3 -> `RegisterClasses::reserved()`, never named in
  `CallingConvention`.
- XR2 -> single frame-base / "logical SP". `frame_pointer()`
  returns `None`.
- `caller_saved` = `{ACC, EXT, XR1}`, `callee_saved` = `{XR2}`.

Research notes that drove this live in `gen-isa/docs/abi-linkage.md`.
Defensive smoke tests in `sw-ibm1130-target/tests/smoke.rs` assert
that XR3 is reserved and never appears in any `CallingConvention`
slot, so a future contributor "freeing up" XR3 immediately fails
its CI.

**Lesson:** invented ABIs need a paranoia check against historical
listings *before* downstream layers (codegen, asm, emulator) are
poured on top. The cost of catching this in step 9 was small (one
follow-on commit on `sw-ibm1130-target`). The cost of catching it
in step 11 (after the emulator and demos cement assumptions) would
have been substantially worse.

## 4. The BSC long-form condition-mask gap

The 1130's BSC instruction tests an accumulator/flag condition
mask. In the long form, that mask lives in 6 bits of the **first**
word's "reserved" area; the second word holds the branch target.

Our ISA spec at saga step 7 marked those 6 bits as reserved-zero,
because the original sample spec did and we didn't catch it. As a
result:

- `sw-ibm1130-isa::Instruction::Long` has no `mask` field.
- `sw_ibm1130_isa::encode::encode_long` always writes 0 to the
  reserved bits.
- The asm parses `BSC L target, mask` but drops the `mask`.
- The emulator's BSC-long handler treats every long-form BSC as
  unconditional.

To get conditional branches, the asm and demos use the
**skip-and-jump** idiom that 1130 software actually used:

```
   BSC mask          ; short form; if mask matches ACC state, skip
                     ;   the next instruction
   BSC L target, 0   ; unconditional jump (mask = 0)
```

The pair effects "branch unless the masked condition matched". This
is structurally identical to how real 1130 code looks; the BSC L
mask field would only be needed if we wanted single-instruction
conditional branches, which our codegen doesn't currently emit.

**Future work to close the gap (out of saga scope):**

- Update `gen-isa/docs/spec-examples/ibm1130.toml` to expose the
  reserved-7-bits area as a per-opcode-conditional `mask` field
  (BSC and MDX use it; most other opcodes leave it zero).
- Re-run `gen-isa scaffold --spec` for `sw-ibm1130-isa` and
  re-apply the hand-written modules (`addr_mode.rs`,
  `branch_cond.rs`).
- Update `Instruction::Long` (or add a per-format wrapper) to
  carry the mask.
- Update asm `encode.rs` to populate the mask.
- Update emulator `exec_bsc` and `exec_mdx` to read the mask from
  its proper home.

Estimated effort: half-day, gated on a willingness to bump the
shipped `sw-ibm1130-isa` crate. Not urgent: the skip-and-jump idiom
keeps codegen and demos working.

## 5. Slot-based codegen with no allocator

`sw-ibm1130-codegen`'s `Backend::lower_module` produces lowered
instructions, but there is no register allocator and no linker:

- Each TIR `ValueId(N)` is bound to memory word slot `N` in the
  function's frame. ACC is the single computational register;
  every op spills to its slot.
- Long-form addresses in emitted instructions are slot indices,
  not real frame offsets. A future allocator pass needs to rewrite
  them to real offsets.
- Inter-function references (calls, return-address slots) emit
  placeholder `0` addresses plus `Fixup` records. Nothing
  resolves the fixups yet.

This is **demo-quality codegen**: the snapshot tests assert that
the lowering shape is right, not that the emitted bytes would run
on real hardware. To get a working pipeline we need:

- A register allocator. `sw-codegen-core::regalloc::LinearScan` is
  a stub; a real linear-scan allocator would walk TIR live-ranges
  and assign physical registers, with spills to frame slots.
- A linker / object writer. `sw-codegen-core::Object<T>` carries
  emitted functions and fixups; nothing currently resolves the
  fixups across functions or globals.
- A frame-size pass. Currently `frame_size` is just "max ValueId +
  1"; a real pass would size the frame based on live spills,
  saved registers, outgoing arg slots, and any callee-saved
  state.

**Postmortem stance:** the slot model was the right call for the
saga (gets a story end-to-end without 2-3 more weeks of allocator
work) but it's a debt the next saga must clear before any pilot
frontend can target the 1130 for real.

## 6. Asm syntax decisions

The two-pass assembler in `sw-ibm1130-asm` made several decisions
that future contributors should know about:

- **Labels require a trailing colon.** First attempt used "label
  in column 1" detection (1130 traditional format), but that stole
  mnemonics like `DC` or `A` when written without leading
  whitespace. Switched to colon-suffix on day one.
- **`I` and `L` flag tokens require lookahead.** First attempt
  consumed `I`/`L` greedily, which broke `STO L I` (intended: STO
  long form to label `I`) by treating `I` as the indirect flag.
  Fix: only consume an `I`/`L` token as a flag if there is at
  least one more token after it on the line. This still lets
  legacy-style `BSC I L 0x42` (indirect long, address 0x42) parse
  correctly.
- **Auto-promote short-to-long is OFF.** Short-form displacement
  out of range produces an error; the user must write `L`
  explicitly. Predictable assembly output beats clever sizing.
- **`BSC L target, mask`** parses but drops the mask -- see
  Section 4.
- **Numeric literals support decimal, hex (`0xNN` / `0XNN`), and
  binary (`0bNN` / `0BNN`).** No expressions yet (no `SYM+1`).
  Adding expressions would need a tiny pratt parser; deferred
  because no demo currently needs them.
- **Round-trip discipline:** the contract is text -> bytes ->
  text' -> bytes' = bytes, not text == text'. Disassembly never
  recovers symbols (we never had them in the byte stream); it
  emits numeric literals. Tests that need text equality should
  go through the `bytes -> bytes` form.

## 7. Emulator scope

`sw-ibm1130-emulator` covers what step-9 codegen and the demos need
and explicitly leaves the rest:

- **Implemented:** Load/Store/LoadDouble/StoreDouble/LoadIndex/
  StoreIndex, all four arith + double-precision pair ops, And/Or/
  Xor, SLA/SRA (arithmetic), BSC short skip-on-condition, BSC long
  unconditional, BSI store-IAR, MDX short increment-and-skip-on-
  zero, Wait halt. XIO with a single-device console output channel.
- **Stubbed as no-op:** LoadStatus, StoreStatus, XIO with any
  area or function other than `(CONSOLE, WRITE)`. Programs that
  use these don't crash but don't get the historical effect.
- **Authoritatively pinned:** the BSC condition-mask bit
  layout (`Z=0x01`, `-=0x02`, `+=0x04`, `E=0x08`, `C=0x10`,
  `O=0x20`); two's-complement sign tests for `+`/`-`; carry
  semantics on Add/Sub.
- **Not implemented:** real device subsystem (1442 card reader/
  punch, 1132/1403 printer, disk channel, paper tape), interrupt
  levels, ILSs, ISSs, sense functions, multi-cycle timing, any
  notion of cycle count or clock.

Demo programs in `tests/programs/` (math, conditions, loops,
strings, hello.asm) double as integration tests; runnable
counterparts in `examples/` print source, hex dump, post-run
memory, and the per-demo result.

## 8. Lessons for ISA #2

For the next ISA bring-up (CDP1802 or RISC-V I32 depending on
`decisions.md` D6), in rough priority order:

1. **Cross-check the ABI against historical listings before
   step 9.** The mid-step-9 revision cost a follow-on commit; it
   would have been free in step 8 if we'd opened the bitsavers
   manuals first. Add a "historical conventions reality check"
   sub-step to the per-ISA template after `-target`.
2. **Look at the ISA spec's "reserved" fields skeptically.**
   Reserved on the architecture often means "used by some opcodes
   for sub-op selection." The BSC mask gap is the cleanest
   example. Per-opcode mask/sub-op fields belong in the ISA spec
   from the start.
3. **Ship one runnable end-to-end demo per saga, not just unit
   tests.** The emulator step's demo programs (and especially the
   hello-world over XIO) made the bring-up feel done in a way that
   "13 unit tests pass" never quite does. Add a "minimal end-to-
   end demo" sub-step to every per-ISA saga template.
4. **The slot/no-allocator codegen worked. Don't try to build a
   real allocator until ISA #3.** A real allocator that has only
   seen the 1130 will bake 1130-isms into the framework. Bring up
   two more ISAs against the slot model first.
5. **Use the asm's round-trip discipline as the contract.** Any
   future asm work (literal expressions, packed-string
   directives) should preserve `bytes == bytes'`.
6. **Conditional branches: pick the idiom early.** The 1130's
   skip-and-jump idiom is verbose but transparent. RISC-V has
   single-instruction conditional branches. CDP1802 has both. Pin
   the lowering idiom in the porting guide before step 9 so the
   codegen pattern doesn't rediscover it.
7. **Keep XIO/IO as a pluggable subsystem.** The emulator's
   `console_output: Vec<u8>` is a one-device hack. ISA #2 might
   want a serial port or a different device shape; the next
   refactor should swap the buffer for a small `Devices` trait
   the emulator carries.
8. **`agentrail` flow held up well; the 8-step + bonus shape
   matched real working pace.** Continue the pattern: scaffolding
   step, then `-isa`, `-target`, `-codegen`, `-asm`, `-emulator`,
   then a postmortem. Bonus polish (demos, EBCDIC, etc.) belongs
   off-saga or in follow-on sagas, not jammed into the bring-up
   timeline.

## 9. Future work (deferred during this saga)

Captured here so the next saga can pick the priorities:

- **LIBF emission and transfer-vector linker pass.** Required for
  any real 1130 software interoperability.
- **Register allocator** (Section 5).
- **Linker / object writer** (Section 5).
- **BSC long-form mask** (Section 4).
- **Character encoding** -- EBCDIC vs ASCII vs Hollerith / per-
  device codes. Plan in `gen-isa/docs/character-encoding-plan.md`.
- **Real device subsystem on the emulator** -- card reader, line
  printer, disk channel, interrupts.
- **Pilot frontend** -- BASIC (or whatever `decisions.md` D3
  resolves) lowering source through TIR through codegen to the
  emulator end-to-end. Phase 3 in `plan.md`, separate saga.
- **COR24 retrofit** -- bring `sw-cor24-isa` under the new
  layering. Phase 6 in `plan.md`, separate saga.
- **Proper `WAIT L` / `WAIT mask` handling.** Currently `WAIT L
  100` errors with "no long form"; the historical 1130 used long-
  form WAIT for some operator-checkpoint patterns. Niche but
  documentable.
- **Asm literal expressions** (`SYM+1`, `LABEL-OTHER`).
- **Branch relaxation** in `sw-codegen-core::branch`.
- **Frame layout pass** in `sw-codegen-core::frame`.

## 10. References

- Per-saga commits on:
  - https://github.com/sw-comp-history/sw-ibm1130-isa
  - https://github.com/sw-comp-history/sw-ibm1130-target
  - https://github.com/sw-comp-history/sw-ibm1130-codegen
  - https://github.com/sw-comp-history/sw-ibm1130-asm
  - https://github.com/sw-comp-history/sw-ibm1130-emulator
- gen-isa supporting docs:
  - `docs/decisions.md` (D1-D8 + saga directional)
  - `docs/abi-linkage.md` (ABI research)
  - `docs/character-encoding-plan.md` (deferred 4-phase plan)
  - `docs/cor24-validation.md` (scaffolder validation)
  - `docs/porting-guide.md` (per-ISA template)
  - `docs/spec-format.md`, `docs/spec-examples/*.toml`
- IBM 1130 manuals on bitsavers:
  - `C26-5929-4` Subroutine Library
  - `C26-3717` Disk Monitor System
  - `GA26-5881` Functional Characteristics
  - `C20-1642-0` FORTRAN Programming Techniques
- MIT-licensed reference implementations cloned locally:
  - `sw-comp-history/ibm-1130-rs` (CPU + assembler reference)
  - `softwarewrighter/demo-ibm-1130-system` (peripheral reference)
- `softwarewrighter/S1130` is read-only reference; do **not** copy
  code from it (no top-level LICENSE).
