Step 5 of saga 'forth-on-1130' -- translate Moore's 1968 kernel into our asm syntax.

# Goal

Produce sw-ibm1130-forth/historical/forth68/kernel.asm by
translating reference/1968-FORTH/FORTH68asm.txt into the modern
sw-ibm1130-asm syntax. Also translate FORTH68lst.txt into
historical/forth68/listing.fth. Per the saga's policy
(historical/forth68/NOTICE + TRANSLATION-LOG.md): preserve Moore's
algorithm, primitive set, identifier names, and dictionary
structure verbatim; only syntactic retargeting allowed.

# Inputs

- reference/1968-FORTH/FORTH68asm.txt (645 lines).
- reference/1968-FORTH/FORTH68lst.txt (234 lines).
- reference/1968-FORTH/FORTH-68_notes.txt (Claunch's analysis).
- historical/forth68/TRANSLATION-LOG.md (policy template).
- historical/forth68/NOTICE (redistribution provenance).
- gen-isa/docs/forth-on-1130-decisions.md Sec 2 (forced deltas).

# What to do

1. Author historical/forth68/kernel.asm by translating
   FORTH68asm.txt line-by-line. Allowed transformations
   (per Sec 2 of the decisions doc):
   - Column layout: fixed -> free-form.
   - Comment character: trailing positional comments -> ; on
     same line. * line-prefix becomes a ; comment OR is dropped
     (depending on whether the next significant thing is a
     directive line we now accept like //, *LIST -- the asm
     handles those as comments).
   - Label suffix: 'FOO' (def-site) -> 'FOO:' (def-site).
     Reference sites stay bare.
   - Hex literals: /XXX preserved (the asm accepts it natively now).
   - BSC mask field: '4Cnn' generations -> 'BSC L addr, 0xnn'
     using the actual mask bits. Same for BSI long form.
   - XR3 references: each rewritten as a memory-cell load/store
     against a new symbol IP_CELL (per delta 2.2). Log each
     rewrite in TRANSLATION-LOG.md per-occurrence.
   - EBCDIC string literals: rewritten as a sequence of numeric
     DC initialisers carrying the original byte values. Each
     literal logged in TRANSLATION-LOG.md.
   - Conditional-branch mnemonics (B/BL/BZ/BNZ/BP/BN/BNP/BOD)
     preserved as-is; asm accepts them as aliases.
   - Shift sub-ops (SLT/SRT) preserved.
   - * (LC), SYM+N expressions preserved.

2. Author historical/forth68/listing.fth by translating
   FORTH68lst.txt similarly. The FORTH listing is denser; same
   rules apply. The output goes into the FORTH source file the
   REPL will load.

3. Update historical/forth68/TRANSLATION-LOG.md:
   - Fill in the upstream-pin section (commit hash of
     monsonite/1968-FORTH used for the translation, date,
     translator).
   - Per-line transformation log for every NON-MECHANICAL
     transformation (XR3 rewrites, EBCDIC literal expansions,
     any directive substitution, any place we punted on a
     line as 'see future-saga'). Mechanical transformations
     (column layout, * -> ;, label colons) are described once
     in the policy section, not logged per occurrence.

4. Verify the translated kernel.asm assembles under
   sw-ibm1130-asm:
   cargo build --manifest-path sw-ibm1130-asm/Cargo.toml
   sw-ibm1130-asm assemble path/to/kernel.asm
   (Or write a small integration test in sw-ibm1130-forth that
   does the assemble call and asserts byte-count or first-few-
   bytes.)

5. Commit and push the translated artifacts (kernel.asm,
   listing.fth, TRANSLATION-LOG.md) to sw-ibm1130-forth.

6. agentrail complete with --next-slug kernel-tests defining
   step 6 (run the assembled kernel on the emulator; assert it
   doesn't crash; assert specific primitives execute correctly).

# Constraints

- Algorithm preserved verbatim. No 'fixes' or 'cleanups' to
  Moore's source.
- Every non-mechanical transformation logged in TRANSLATION-LOG.md.
- ASCII-only (the existing crates' policy).
- Do not modify any sw-ibm1130-* shipped crate; the asm
  extensions from saga step 4 are sufficient.
- If a line cannot be translated faithfully (e.g. it uses a
  feature we deliberately defer per delta 2.6 or 2.7), comment
  it out with a clear '; STUB: deferred to saga X' marker and
  log it in TRANSLATION-LOG.md.

# Acceptance

- sw-ibm1130-forth/historical/forth68/kernel.asm exists and
  assembles cleanly under sw-ibm1130-asm.
- sw-ibm1130-forth/historical/forth68/listing.fth exists.
- TRANSLATION-LOG.md filled with the upstream pin and the
  per-line non-mechanical transformations.
- One commit on sw-ibm1130-forth; saga step 6 (kernel-tests)
  queued via --next-slug.

# Out of scope

- Running the kernel on the emulator (that is saga step 6).
- Any new asm features (step 4 was the last asm extension).
- Translating supporting prose, Carl Claunch's PDF, or the
  upstream README.