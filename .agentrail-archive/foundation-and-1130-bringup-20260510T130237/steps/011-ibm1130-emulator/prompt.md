Step 11 of saga "foundation-and-1130-bringup".

# Goal

Bring up sw-comp-history/sw-ibm1130-emulator -- the fifth IBM 1130
crate. Minimal IBM 1130 emulator that executes a curated test
program end-to-end. Covers the instruction subset that
sw-ibm1130-codegen produces.

# Inputs

- sw-comp-history/sw-ibm1130-isa -- Architecture, decode, Reg,
  Opcode, Instruction.
- sw-comp-history/sw-ibm1130-asm -- assembles test programs; we use
  it as the input vehicle for emulator inputs.
- sw-comp-history/sw-ibm1130-codegen -- Backend; emit_asm output
  feeds into the asm to produce test program bytes.
- sw-comp-history/sw-ibm1130-target/docs/abi.md -- ABI; emulator
  initialises XR2 (frame base/SP) and respects XR3 = LIBF base
  reservation (treats XR3 as program-lifetime invariant).
- sw-comp-history/ibm-1130-rs cpu/executor.rs (MIT, free to
  reference for execution semantics; do NOT copy code from S1130).
- gen-isa/docs/porting-guide.md Sec 7 (asm and emulator notes).

# What to do

1. Run gen-isa scaffold for the -emulator crate; keep only the
   sw-ibm1130-emulator subdir.

2. Cargo.toml dependencies: sw-isa-core, sw-ibm1130-isa,
   sw-ibm1130-asm (dev-dep, for tests).

3. Implement minimal CPU state in src/state.rs: ACC, EXT, XR1, XR2,
   XR3 as 16-bit registers; IAR (PC); flags Carry, Overflow.
   Provide Default for a fresh-CPU state.

4. Implement memory in src/memory.rs: 16-bit-word-addressed memory
   (default 4096 words = 8 KB), word-read and word-write API.

5. Implement instruction execution in src/exec.rs: a step()
   function that decodes the next instruction at IAR, executes it,
   and advances IAR. Handle:
   - Memory ops: Load, LoadDouble, Store, StoreDouble, LoadIndex,
     StoreIndex (XR1/XR2/XR3 read+write).
   - Arith: Add, AddDouble, Subtract, SubtractDouble, Multiply
     (M -> ACC,EXT pair), Divide (D -> ACC quotient, EXT remainder).
   - Logical: And, Or, ExclusiveOr.
   - Shift: SLA / SRA (count from disp bits).
   - Branch: BSC short (skip-if-condition), BSC long (branch with
     condition mask), BSI (subroutine call: write IAR to target,
     branch to target+1), MDX (modify index, with skip-if-zero).
   - Control: Wait (set halt flag).
   - I/O: XIO -- can stub for now (no-op + log).

6. Run-to-halt loop in src/lib.rs: while not halted, step(). Tests
   load programs, run-to-halt, assert on memory + register state.

7. Curated programs (in tests/programs.rs or similar):
   - simple-add: ld arg1; a arg2; sto result; wait. Verify ACC and
     memory after run.
   - simple-loop: a counter, mdx-loop, until counter is zero. Verify
     XR1 and the iteration count.
   - call-and-return: a tiny subroutine using BSI L NAME and BSC I
     NAME, invoked from a main. Verify end state.
   At least one of these programs should execute to completion via
   the emulator and produce the expected result -- the saga's exit
   criterion ("runs at least one curated test program") is met.

8. cargo build / test / clippy -D warnings / fmt --check clean.

9. Move to ~/github/sw-comp-history/sw-ibm1130-emulator, git init,
   first commit, gh repo create --public --source=. --push.

10. Update gen-isa/docs/status.md crate-state row.

11. Commit gen-isa changes and agentrail complete with
    --next-slug postmortem, --next-prompt defining step 12.

# Constraints

- Minimal exec semantics covering codegen's output, NOT a full 1130
  emulator. Defer multi-cycle timing, all I/O devices, the disk
  channel, interrupts, ILSs/ISSs.
- BSC condition-mask semantics need to be pinned down; this is
  where they get authoritative. Document the chosen mapping in
  src/exec.rs comments and update sw-ibm1130-codegen's cmp_mask if
  needed.
- ASCII-only in any docs.

# Acceptance

- sw-comp-history/sw-ibm1130-emulator exists and is pushed.
- At least one curated assembly program runs to halt and produces
  the expected memory+register state.
- gen-isa/docs/status.md updated.