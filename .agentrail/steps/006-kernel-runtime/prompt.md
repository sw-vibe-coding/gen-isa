Step 6 of saga 'forth-on-1130' -- run the translated kernel on the emulator.

# Goal

Take historical/forth68/kernel.asm assembled bytes, load into sw-ibm1130-emulator memory at ORG=0x900, set IAR to START, run for N steps, and observe the kernel's behaviour. Establish a runtime test harness that future translation refinements can use to catch regressions.

# Inputs

- sw-ibm1130-forth/historical/forth68/kernel.asm (the translated kernel).
- sw-ibm1130-forth/historical/forth68/TRANSLATION-LOG.md (lists the known stubs + deferrals; runtime tests should respect them).
- sw-ibm1130-emulator (the executor).
- sw-ibm1130-asm (the assembler).

# What to do

1. Add a sw-ibm1130-forth integration test (tests/kernel_runtime.rs) that:
   - Assembles historical/forth68/kernel.asm via sw-ibm1130-asm.
   - Allocates a 4096-word emulator memory; loads bytes at byte offset 0x1200 (= word 0x900, the ORG of the kernel).
   - Sets state.iar = START (resolve via the asm's symbol table).
   - Runs N steps with a guard (max_steps = 1000? 10000?).
   - Asserts: emulator didn't error out, IAR is in the expected range, no unrecoverable trap was hit. NOT a 'kernel works' test -- just 'kernel doesn't crash on N steps from START'.

2. Iterate on stubs that the kernel hits during run-to-N-steps. Each iteration: identify the issue, fix in kernel.asm (with a TRANSLATION-LOG entry), or note as a known runtime issue.

3. Produce a runtime-status doc: historical/forth68/RUNTIME-STATUS.md listing what works, what hits a stub, and what's actively broken.

4. Commit and push the test + any kernel.asm fixes.

5. agentrail complete with --next-slug listing-translation (step 7: translate FORTH68lst.txt) or --next-slug dictionary-table (step 7 alt: fill in the stubbed dictionary).

# Constraints

- No changes to sw-ibm1130-asm / -isa / -emulator unless the runtime trace reveals a genuine bug. Surface any such finding as its own commit on that crate.
- Respect the stubs documented in TRANSLATION-LOG.md. Don't try to fix BLOCK / PUT / PRINT in this step.
- ASCII-only.

# Acceptance

- sw-ibm1130-forth/tests/kernel_runtime.rs exists and passes.
- historical/forth68/RUNTIME-STATUS.md exists with the observed behaviour.
- Any genuine bugs found in our toolchain are filed with separate commits on the affected crate.