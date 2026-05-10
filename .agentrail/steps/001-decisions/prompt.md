Step 1 of saga 'forth-on-1130' -- decisions (docs only).

# Goal

Lock the open sub-decisions Q1-Q5 from gen-isa/docs/forth-on-1130-plan.md
Sec 7, plus the asm-extension scope from Sec 10. Capture in
gen-isa/docs/forth-on-1130-decisions.md so subsequent saga steps
can rely on these as fixed.

# Inputs

- gen-isa/docs/forth-on-1130-plan.md (the plan; especially Sec 7
  open questions and Sec 10 assembler features needed).
- gen-isa/docs/postmortem-1130-bringup.md (BSC mask gap, slot
  codegen debt, lessons for the next saga).
- gen-isa/docs/character-encoding-plan.md (EBCDIC vs ASCII; this
  saga can lean on the deferral.)
- gen-isa/docs/abi-linkage.md (XR3 = LIBF base; constrains FORTH
  register allocation -- can't use Moore's XR3=auxiliary scheme
  verbatim).
- ~/github/sw-comp-history/sw-ibm1130-forth/historical/forth68/NOTICE
  + TRANSLATION-LOG.md + README.md (the redistribution policy
  already in place).

# What to do

1. Author gen-isa/docs/forth-on-1130-decisions.md. Sections:
   a. Q1 -- ITC vs DTC vs subroutine-threaded? Recommend ITC
      for historical fidelity to Moore's 1968 design.
   b. Q2 -- Block I/O? Defer; pilot saga is keyboard + console
      only.
   c. Q3 -- ASCII or EBCDIC source? ASCII for now; align with
      character-encoding-plan.md retrofit later.
   d. Q4 -- Self-hosting goal? Not in saga scope; reach
      end-to-end-runs-on-emulator first.
   e. Q5 -- Primitive set? Historical 28 in the kernel; layered
      'extension set' for REPL conveniences (HERE, LATEST, WORDS,
      BYE, : , ;, ., ", etc.).
   f. Asm-extension scope -- which Sec-10 items are required
      (in scope) vs. punt-able (out of scope) for this saga.
   g. XR3 deviation -- reconfirm Moore used XR3 as auxiliary;
      we cannot, because XR3 is reserved as LIBF base by our
      ABI. Document the substitute (memory cell holding the IP
      / fourth pointer; explicit per-primitive load/store).
   h. Translation scope -- syntactic-retargeting only (per
      historical/forth68/TRANSLATION-LOG.md policy); the
      translation step (saga step 5) is the sole place where
      Moore's source enters our repo. Reaffirm.

2. Cross-link the new doc from forth-on-1130-plan.md (Sec 7
   open questions) and from sw-ibm1130-forth's
   docs/references.md.

3. Commit gen-isa changes. No code changes in this step.

4. agentrail complete with --next-slug reference-cloning, summary
   describing what's locked, and --next-prompt for step 2:
   side-by-side primitive table comparing Moore's 1968 kernel and
   the cor24-forth reference, written into gen-isa/docs (or
   sw-ibm1130-forth/docs depending on what fits better).

# Constraints

- Pure docs. No code, no scaffolding work.
- ASCII-only.
- One commit (or two: docs commit + agentrail bookkeeping commit
  per CLAUDE.md flow).

# Acceptance

- gen-isa/docs/forth-on-1130-decisions.md exists with all 8
  subsections above.
- Cross-links updated.
- Committed and pushed.
- Step 2 (reference-cloning) queued via --next-slug.