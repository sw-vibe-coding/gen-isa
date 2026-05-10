Step 10 of saga "foundation-and-1130-bringup".

# Goal

Bring up sw-comp-history/sw-ibm1130-asm -- the fourth IBM 1130 crate.
A two-pass assembler that parses 1130 mnemonics and emits 1130
instruction bytes via sw-ibm1130-isa::encode. Round-trips: asm ->
bytes -> disasm -> asm produces a textually equivalent listing.

# Inputs

- sw-comp-history/sw-ibm1130-isa -- Opcode, Reg, Instruction, encode,
  decode, parse_register, addr_mode, branch_cond.
- sw-comp-history/sw-ibm1130-codegen -- emit_asm output is the
  consumer that asm parses (so codegen-produced text is the canonical
  asm shape we must accept).
- sw-comp-history/sw-ibm1130-target/docs/abi.md -- understand inline
  DC parameter shape after BSI calls.
- sw-comp-history/ibm-1130-rs assembler at src/cpu/assembler.rs (MIT,
  free to reference for parser shape; do NOT copy code from S1130).
- gen-isa/docs/porting-guide.md Sec 7 (asm and emulator notes).

# What to do

1. Run gen-isa scaffold for the -asm crate with --spec; keep only
   the sw-ibm1130-asm subdir.

2. Cargo.toml dependencies: sw-isa-core, sw-ibm1130-isa, plus a
   lightweight error type. No external parser dep -- hand-roll a
   lexer + recursive-descent parser; the language is small.

3. Implement two-pass assembly:
   - Pass 1: scan source, build a symbol table mapping labels to
     word addresses; resolve EQU equivalences; track origin (ORG)
     directives.
   - Pass 2: emit instructions. For each line: parse mnemonic, parse
     operands, look up addresses, call sw_ibm1130_isa::encode::encode
     to produce bytes. Handle short vs long form selection by
     displacement range; emit DC directives literally.

4. Support the directives the codegen consumer needs at minimum: ORG,
   EQU, DC, END. Plus all 24 mnemonics in the ISA spec (ld, ldd, sto,
   std, ldx, stx, lds, sts, a, ad, s, sd, m, d, and, or, eor, sla,
   sra, bsc, bsi, mdx, wait, xio).

5. Round-trip test: take a curated asm listing (start with the output
   of sw-ibm1130-codegen's emit_asm for the snapshot test cases),
   assemble to bytes, disassemble via sw_ibm1130_isa::decode, render
   back to asm, assert textual equivalence.

6. Error reporting: parse errors carry source position (line, col)
   and a clear message; tests assert on at least 3 error messages
   for typical mistakes (unknown mnemonic, out-of-range disp,
   undefined label).

7. cargo build / test / clippy -D warnings / fmt --check clean.

8. Move to ~/github/sw-comp-history/sw-ibm1130-asm, git init, first
   commit, gh repo create --public --source=. --push.

9. Update gen-isa/docs/status.md crate-state row.

10. Commit gen-isa changes and agentrail complete with
    --next-slug ibm1130-emulator, --next-prompt defining step 11.

# Constraints

- Hand-rolled parser; no external parser-combinator dep.
- ASCII-only source language (no UTF-8 weirdness).
- Round-trip is the test; if asm -> bytes -> disasm -> asm doesn't
  match for the codegen output, the asm is wrong.
- Do not modify sw-ibm1130-isa or earlier shipped crates unless a
  trait-surface change is forced; document any such change for the
  postmortem.

# Acceptance

- sw-comp-history/sw-ibm1130-asm exists and is pushed.
- All 24 mnemonics + ORG/EQU/DC/END parse and assemble correctly.
- Round-trip test passes for the codegen-emit_asm output of every
  step-9 snapshot test.
- At least 3 negative tests for parse errors with source-position
  reporting.
- gen-isa/docs/status.md updated.