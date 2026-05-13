Step 4 of saga 'forth-on-1130' -- asm extensions for ingesting Moore's 1968 source.

# Goal

Extend sw-ibm1130-asm with the directives, literal forms, mnemonic
aliases, and operand expressions that Moore's FORTH68asm.txt uses
and our current asm doesn't accept. This is the second pre-
translation infrastructure step (after step 3 closed the BSC mask
gap). After this step ships, saga step 5 can perform the actual
translation of Moore's kernel.

# Inputs

- sw-ibm1130-forth/docs/moore-1968-survey.md Sec 8-9 (asm
  directives observed; non-required deferred items).
- sw-ibm1130-forth/docs/moore-1968-primitives.md Sec D-F
  (directives table, conditional-branch mnemonics, shift sub-ops).
- sw-ibm1130-forth/reference/1968-FORTH/FORTH68asm.txt (the
  actual source we need to ingest).
- gen-isa/docs/forth-on-1130-decisions.md Sec 2.6 (asm-extension
  scope decisions: in-scope vs out-of-scope per directive).

# What to do (in sw-ibm1130-asm)

1. Hex literal '/XXX' prefix: tokeniser accepts '/' followed by
   hex digits as a hex literal, equivalent to '0xXXX'. Update
   parser's read_operand_token and parse_number accordingly.

2. Directive additions:
   - 'BSS N' (block storage symbol): reserve N words at the
     current location counter, emit N zero words.
   - 'END LABEL' (end with operand): accept an optional operand;
     stored as the entry-point label in the symbol-table output
     for tooling that wants it.
   - 'ABS' as a no-op directive (declare absolute / non-
     relocatable; we don't relocate, so it's a no-op marker).
   - '// JOB', '// ASM', '*LIST ALL' lines: treat as comments
     (consume the rest of the line; do not error).

3. Conditional-branch mnemonic aliases. Add to mnemonic_to_opcode
   (or a thin wrapper) the macros that expand to BSC + mask:
       B mnem  -> BSC short, mask = 0  (or BSC L when /L given)
       BL      -> BSC long, mask = 0
       BZ      -> BSC long, mask = 0x20 (Z)
       BNZ     -> BSC long, mask = 0x18 (P|N)
       BP      -> BSC long, mask = 0x08 (+)
       BN      -> BSC long, mask = 0x10 (-)
       BNP     -> BSC long, mask = 0x28 (- | Z)
       BOD     -> BSC long, mask = 0x04 (E) -- note: 'OD' is 'odd'
                 in 1130 asm but mask 0x04 = Even in Moore. Need to
                 verify Moore's actual BOD usage; might be BSC short.
   Each emits the underlying BSC instruction with the mask baked
   in (caller does NOT pass a separate mask operand).

4. Shift sub-op mnemonics. SLA / SRA take a count; SLT (Shift
   Left Together, both ACC and EXT) and SRT (Shift Right Together)
   are sub-ops encoded by displacement bits per the FC manual.
   Per the survey, Moore's source uses SRT 1 and SLT 1. Add at
   least these two; check what other sub-ops appear and add as
   needed.

5. Literal expressions in operands. Add a small Pratt parser
   under parse_operand for + / - / * with parenthesisation;
   also accept '*' as the current location counter (LC).
   Examples Moore uses: 'IC+1', '*-2', 'queue+1', 'A(2)'.
   Symbol table lookup is the same; arithmetic combines.

6. Cargo test / clippy -D warnings / fmt --check clean.

7. Round-trip discipline: bytes-equal round-trip still holds for
   sources using the new directives / mnemonics.

8. Negative tests for the new features:
   - /not-hex -> parse error.
   - BSS with non-numeric operand -> error.
   - Expression with undefined symbol -> error with source pos.
   - BZ with operand mask -> error (mnemonic aliases don't accept
     a user-supplied mask).

# Constraints

- All extensions belong in sw-ibm1130-asm. No changes to other
  shipped crates.
- The current 27-test suite must continue to pass.
- ASCII-only.

# Acceptance

- sw-ibm1130-asm accepts /XXX hex literals.
- BSS, END LABEL, ABS, // ..., *LIST ALL all parse correctly.
- BZ/BNZ/BP/BN/BNP/BOD/B/BL mnemonics emit correct masked BSC
  instructions.
- SLT/SRT shift sub-ops encode correctly.
- Operand expressions like 'SYM+1', '*-2' resolve correctly.
- New negative tests cover the obvious user errors.
- All sw-ibm1130-asm tests still pass plus the new ones for
  each feature.
- One commit on sw-ibm1130-asm; saga step 5 (translation) queued.