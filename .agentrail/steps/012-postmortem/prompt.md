Step 12 of saga "foundation-and-1130-bringup".

# Goal

Write gen-isa/docs/postmortem-1130-bringup.md capturing the trait-
surface changes, design decisions revisited, and lessons learned
during steps 7-11 of the IBM 1130 bring-up.

# Inputs

- gen-isa/docs/decisions.md -- original decisions; some have shifted.
- gen-isa/docs/abi-linkage.md -- mid-saga ABI revision research.
- gen-isa/docs/character-encoding-plan.md -- deferred design for
  EBCDIC/ASCII/Hollerith handling.
- The 5 IBM 1130 crates (sw-ibm1130-{isa,target,codegen,asm,emulator})
  and their commit histories on the sw-comp-history org.
- gen-isa/docs/status.md -- Recent changes section captures what
  shipped per step.

# What to do

1. Author docs/postmortem-1130-bringup.md (ASCII-only) with sections:
   a. Summary -- what was built; key decisions made.
   b. Trait-surface evolution -- what in sw-langtools changed during
      bring-up (anything?). For 1130 we kept the trait surfaces;
      document why and what we punted.
   c. ABI revision -- what step-8 got wrong, what fixed it, what
      future readers should know about XR3 = LIBF base.
   d. BSC condition-mask gap -- the long-form mask field that our
      ISA spec marked reserved-zero; impact on codegen/emulator and
      what to do for ISA #2.
   e. Slot-based codegen with no allocator -- why we shipped a
      naive ValueId-to-memory-slot model, and what the proper
      register allocator step looks like later.
   f. Asm syntax decisions -- colon-suffixed labels, I/L flag
      lookahead, deferred features (literal expressions, codegen
      inline-arg blocks).
   g. Emulator scope -- what we left as no-op (XIO, status word
      ops), and why those are fine to defer.
   h. Lessons for ISA #2 -- 5-10 concrete recommendations for the
      next ISA bring-up: things we'd do differently, things to keep,
      where to budget more time.

2. Cross-link the postmortem from gen-isa/docs/status.md (Recent
   changes section) and from any per-crate README that has a
   'limitations' section.

3. Run agentrail complete with --next-slug status-update and a
   short --next-prompt for step 13 (the final status sweep).

# Constraints

- ASCII-only.
- No code changes in this step -- pure docs.
- Do not edit any earlier shipped crate's source. If you discover
  something that should be a code change, list it in the
  postmortem under 'Future work' rather than fixing it here.

# Acceptance

- gen-isa/docs/postmortem-1130-bringup.md exists with the 8 sections
  named above.
- Cross-links from status.md.
- gen-isa changes committed.