Step 7 of saga 'forth-on-1130' -- fill in the stubbed dictionary and translate the FORTH listing.

# Goal

Move from 'kernel boots' to 'kernel can interpret a FORTH word' by:

1. Filling in the initial dictionary block (upstream lines 519-635 of FORTH68asm.txt) -- 100+ pre-built dictionary entries that the kernel's DO routine walks.

2. Translating FORTH68lst.txt (234 lines) into historical/forth68/listing.fth.

3. Adding a runtime test that injects a small FORTH program into the kernel's input buffer (via direct memory writes), runs the kernel, and observes it parse one or two words successfully.

# Inputs

- reference/1968-FORTH/FORTH68asm.txt lines 519-640 (the
  initial dictionary).
- reference/1968-FORTH/FORTH68lst.txt (the FORTH overlay).
- historical/forth68/kernel.asm (the current translated kernel).
- historical/forth68/TRANSLATION-LOG.md (extend with new entries).
- historical/forth68/RUNTIME-STATUS.md (update with new state).

# What to do

1. Translate upstream lines 519-640 into kernel.asm: the chain of dictionary entries. Each entry is 4 words: name-half-1, name-half-2, code-address, then a blank slot. Replace the current sentinel 'E2' stub with the real chain. The end of the chain is marked by the END directive at the bottom.

2. Translate FORTH68lst.txt into historical/forth68/listing.fth. This is FORTH source, not asm. For now we don't have a way to LOAD .fth files into the running kernel; mark as documentation-only and reference it from runtime-status. A future I/O step will wire it in.

3. Add tests/kernel_interprets_one_word.rs (or extend kernel_runtime.rs) that:
   - Boots the kernel as before.
   - Before stepping, injects a tiny FORTH source like '12 +' into the SECT input area at the kernel's expected position.
   - Sets up workspace pointers so ACCEPT reads from SECT instead of trying to LIBF a disk.
   - Runs for N steps.
   - Asserts: parsed a word; the stack got 0x12 pushed (or whatever).

   This is more ambitious than step 6's boot test; it actually exercises the parse-and-execute path.

4. Update historical/forth68/RUNTIME-STATUS.md and TRANSLATION-LOG.md.

5. Commit and push.

6. agentrail complete with --next-slug postmortem (step 8) or --next-slug io-bringup (step 8 alt: real input/output).

# Constraints

- Don't add asm features unless the runtime trace demands them.
- Don't fix LIBF in this step (still deferred per delta 2.6).
- ASCII-only.
- One main commit on sw-ibm1130-forth; toolchain bugs (if any) get separate commits on the relevant crate.

# Acceptance

- historical/forth68/kernel.asm has the full initial dictionary.
- historical/forth68/listing.fth exists (translated; not necessarily executable yet).
- The new runtime test passes (kernel parses at least one word).
- RUNTIME-STATUS.md and TRANSLATION-LOG.md updated.