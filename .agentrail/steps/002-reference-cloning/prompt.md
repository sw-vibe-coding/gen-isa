Step 2 of saga 'forth-on-1130' -- read Moore's source and produce a side-by-side primitive table.

# Goal

Read Moore's 1968 FORTH source carefully and produce a
side-by-side primitive table. The table is the input to saga step
3 (BSC mask fix) and step 4 (asm extensions): we won't know what
to fix until we know what Moore's kernel actually uses.

# Inputs

- ~/github/sw-comp-history/sw-ibm1130-forth/reference/1968-FORTH/
  (locally cloned monsonite/1968-FORTH; gitignored). Contains:
    FORTH68asm.txt        -- 645-line 1130 asm source
    FORTH68lst.txt        -- 235-line FORTH-level disk dump
    FORTH-68_notes.txt    -- implementation notes
    notes on FORTX assem code.pdf -- Carl Claunch's analysis
    README.md             -- upstream provenance
- ~/github/sw-embed/sw-cor24-forth/forth.s and
  ~/github/sw-embed/sw-cor24-forth/forth-from-forth/kernel.s --
  the COR24 DTC FORTH structural reference.

# What to do

1. Read FORTH68asm.txt end-to-end with the Carl Claunch PDF open
   for context. Catalogue every primitive (Moore's 28 plus any
   support words) with: name, body's first ~5 lines summarised,
   register usage, branch usage (does it call BSC long w/ mask?
   short skip? BSI?).

2. Read FORTH68lst.txt to see what FORTH source level Moore
   shipped on top of the kernel. Note any words defined there
   that the saga's REPL extension layer should match.

3. Cross-reference against cor24-forth's primitives to see which
   are the same, which are different, and which are COR24-only
   conveniences.

4. Author sw-ibm1130-forth/docs/moore-1968-primitives.md (in the
   sw-ibm1130-forth repo, NOT in gen-isa) with a side-by-side
   table:
       | Primitive | Moore 1968 | cor24-forth | Notes |
       |-----------|------------|-------------|-------|
   Plus a section listing every directive Moore's source uses
   (so saga step 4 can confirm the asm-extension scope from
   step 1's Sec 2.6).

5. Author sw-ibm1130-forth/docs/moore-1968-survey.md (also in
   sw-ibm1130-forth) with prose findings: how Moore structured
   the kernel, register conventions actually used (vs. notes
   suggest), branch idioms, EBCDIC literal usage, anything
   surprising.

6. Update the BSC-mask-fix saga step (step 3 in
   forth-on-1130-plan.md) with a concrete count of BSC-long-mask
   call sites in Moore's source. This is the data that justifies
   doing the fix at all.

7. Commit and push the new docs in sw-ibm1130-forth. Update
   gen-isa side: cross-link from forth-on-1130-plan.md to the
   new survey docs.

8. agentrail complete with --next-slug bsc-mask-fix.

# Constraints

- Pure docs / reading work. No code changes.
- Do NOT copy Moore's source verbatim into either of the new
  docs. Quote sparingly with attribution; the sw-ibm1130-forth
  repo's redistribution policy says verbatim copying lives only
  in historical/forth68/ (under the translation step), not in
  survey docs.
- ASCII-only.
- Treat reference/1968-FORTH/ as read-only; do not modify it.

# Acceptance

- sw-ibm1130-forth/docs/moore-1968-primitives.md exists with the
  side-by-side primitive table, complete for Moore's 28 + any
  support words.
- sw-ibm1130-forth/docs/moore-1968-survey.md exists with the
  prose findings.
- gen-isa/docs/forth-on-1130-plan.md cross-linked to both.
- Step 3 (bsc-mask-fix) queued via --next-slug.