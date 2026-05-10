Step 3 of saga 'forth-on-1130' -- close the BSC long-form condition-mask gap.

# Goal

Update sw-ibm1130-isa, sw-ibm1130-asm, and sw-ibm1130-emulator
to support the BSC and BSI long-form condition-mask field. This
is the single biggest infrastructure delta Moore's kernel
exposes; with it closed, the kernel translation in saga step 5
can emit BSC L target, mask and BSI L target, mask directly
matching Moore's source 1:1.

# Inputs

- sw-ibm1130-forth/docs/moore-1968-survey.md Sec 4 (the BSC-mask
  call-site count and mask-bit assignments).
- sw-ibm1130-forth/docs/moore-1968-primitives.md Sec E
  (conditional-branch mnemonic table).
- gen-isa/docs/postmortem-1130-bringup.md Sec 4 (the gap; a
  half-day-effort estimate).
- gen-isa/docs/spec-examples/ibm1130.toml (current spec where
  the long-form 'reserved' bits live).
- sw-ibm1130-isa, sw-ibm1130-asm, sw-ibm1130-emulator (the three
  shipped crates that need updating).

# What to do

1. Update gen-isa/docs/spec-examples/ibm1130.toml: replace the
   long-form 'reserved' field with a 'mask' field exposed for
   BSC and BSI opcodes. Document any opcodes that should still
   reserve those bits to zero.

2. Re-run gen-isa scaffold --spec for sw-ibm1130-isa. Re-apply
   the hand-written addr_mode.rs and branch_cond.rs modules
   (the lib.rs declares them with HAND-ADDED markers).

3. Update Instruction::Long (or add a per-format wrapper) to
   carry the mask byte. Roundtrip tests must continue to pass
   with the wider Instruction.

4. Update sw-ibm1130-asm/src/encode.rs to populate the mask in
   build_long when a BSC or BSI mnemonic + mask operand is
   emitted. Keep the existing mask-as-second-operand syntax;
   it is now load-bearing.

5. Update sw-ibm1130-emulator/src/exec.rs:
   - exec_bsc: read mask from Instruction::Long's new mask
     field. Reconcile the bit assignments to match Moore's
     :POSITIVE/:NEGATIVE/:EQUAL/:EVEN constants
     (P=0x08, N=0x10, Eq=0x20, Even=0x04). Update the doc
     comment that pins the convention.
   - exec_bsi: ditto. BSI long with mask = conditional call
     (call only if mask matches; fall through if not).
   - Update the unit tests in tests/exec_units.rs to cover the
     new mask semantics for BOTH BSC and BSI long form.

6. Update sw-ibm1130-emulator/tests/programs/conditions.asm and
   loops.asm to use BSC L target, mask directly (the natural
   way) rather than the skip-and-jump workaround. The existing
   demos should still pass.

7. Run cargo build / test / clippy -D warnings / fmt --check
   across all three repos. Fix any breakage.

8. Push each repo's update as a single commit.

9. Update sw-ibm1130-forth/docs/moore-1968-survey.md to mark
   Sec 4 (the BSC-mask gap) as 'closed'.

10. agentrail complete with --next-slug asm-extensions,
    --next-prompt defining step 4 (the rest of the directives
    + mnemonics from saga step 2's survey: ABS, /XXX hex, BSS,
    END LABEL, B/BL/BZ/BNZ/BP/BN/BNP/BOD aliases, SLT/SRT
    shift sub-ops, literal expressions in operands).

# Constraints

- Three crates need updating in lockstep (sw-ibm1130-isa, -asm,
  -emulator). Each gets its own commit with the mask change
  scoped to that repo.
- Roundtrip discipline must hold: bytes -> decode -> encode ->
  bytes' equals bytes for all instruction shapes including the
  new masked BSC/BSI.
- Existing demos (math, conditions, loops, strings, hello-world)
  must still pass after the change.
- ASCII-only.

# Acceptance

- ibm1130.toml exposes a 'mask' field for BSC/BSI long form.
- sw-ibm1130-isa decodes/encodes the mask correctly. All
  existing tests pass.
- sw-ibm1130-asm parses 'BSC L target, mask' and emits the
  mask in the encoded bytes.
- sw-ibm1130-emulator's exec_bsc/exec_bsi consume the mask
  with the Moore-authoritative bit assignments. Updated unit
  tests cover the new semantics.
- conditions.asm and loops.asm rewritten to use direct masked
  branches; demos still pass.
- All three repos have clippy/fmt clean and tests passing.
- Step 4 (asm-extensions) queued via --next-slug.