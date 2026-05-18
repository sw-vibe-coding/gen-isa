Step 9 of saga 'forth-on-1130' -- restore the factor-of-2 byte-address arithmetic that Moore had and the translation dropped.

# Goal

Add multiplication operator to sw-ibm1130-asm (supports 'N*M' in operand expressions) and restore the 2*WORD, 2*SECT+74, 2*SECT+642+72, 2*PRN+2, 2*PRN+74 byte-address constants in kernel.asm that the saga step 5 translation dropped (TRANSLATION-LOG LC.5). With those constants restored, the kernel's workspace pointers will hold true byte-addresses and FETCH/DEPOSIT's word-vs-byte arithmetic will index correctly.

After landing, extend tests/kernel_parses.rs's inject test to assert on observable parser behaviour (workspace[0] gets the converted FORTH internal code for the injected char).

# What to do

1. Add multiplication operator to sw-ibm1130-asm:
   - parser.rs: add Operand::Multiply { lhs: i64, rhs: Box<Operand> } variant.
   - split_multiply helper analogous to split_offset; finds '*' at position > 0 (so '*-N' is still parsed as LC + offset, not as multiplication).
   - parse_operand_text dispatch order: try split_offset first (offset wraps everything), then split_multiply, then the atomic forms (number / symbol / *).
   - encode.rs and symtab.rs: resolve recursively. lhs is a const, rhs resolves like a sub-expression.
   - tests: '2*WORD' = 2 * (address of WORD); '2*WORD+5' = 2*WORD + 5; '*' still LC; '*-2' still LC-2.

2. Update kernel.asm to restore Moore's byte-address constants:
   - W1 first slot: 'DC WORD_SYM' -> 'DC 2*WORD_SYM'.
   - START's 'LDX L 3, SECT_BASE' -> 'LDX L 3, 2*SECT+74' (the saga-step-5 translation collapsed 2*SECT+74 to SECT_BASE).
   - C1 / C2 / C3 init values: restore '2*SECT+642+72', etc.
   - DP0 / DP / DP1: restore 2*PRN+2, 2*PRN+74.
   - Remove the SECT_BASE label if it's no longer needed.
   - Run kernel_parses.rs's inject test; if workspace[0] now picks up the converted char, assert on it.

3. Update RUNTIME-STATUS.md and TRANSLATION-LOG.md (mark LC.5 as RESOLVED).

4. Commit the asm changes separately from kernel/translation changes.

5. agentrail complete with --next-slug postmortem.

# Constraints

- ASCII-only.
- All existing 8 kernel-side tests must continue to pass.
- All 46 asm tests must continue to pass.
- Toolchain bugs (if any new ones surface) get separate commits on the affected crate.

# Acceptance

- sw-ibm1130-asm has Multiply operator with tests.
- kernel.asm uses 2*X arithmetic where Moore did.
- inject test asserts on workspace[0] having the converted FORTH code value (0x0A for 'A').
- All previously-passing tests still pass.
- RUNTIME-STATUS.md and TRANSLATION-LOG.md updated.