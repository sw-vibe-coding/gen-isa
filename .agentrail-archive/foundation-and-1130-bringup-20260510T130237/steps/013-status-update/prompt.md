Step 13 of saga 'foundation-and-1130-bringup' -- final saga step.

# Goal

Bring docs/status.md fully up to date to reflect the actual phase
progress. This is a docs-only sweep with no code changes.

# What to do

1. Re-read docs/status.md end-to-end. It has been updated incrementally
   step by step, but several sections were drafted before any code
   landed; verify they reflect the actual outcome.

2. Update the Phase summary table:
   - Phase 0 (discovery) is still 'not started' -- correct.
   - Phase 1 (layered scaffolding) should be 'complete' (all 5 framework
     crates exist and are tested).
   - Phase 2 (first new ISA, IBM 1130) should be 'complete' (all 5 per-
     ISA crates exist; emulator runs curated programs).
   - Phases 3-8 still 'not started' -- correct.

3. Update the Crate-state tables to reflect the final state:
   - Layered scaffolding row: still all 'exists / skeleton built /
     smoke tests' which is accurate.
   - Per-ISA crates row for IBM 1130: all five marked 'exists' with
     crate counts and test counts; that is already current as of
     step 12. Spot-check.
   - COR24 row: still un-retrofitted (phase 6).

4. Update 'Open decisions waiting on user' if any have been resolved
   during the saga; otherwise leave as-is.

5. Add a final Recent changes entry for step 13 itself ('finalised
   docs/status.md to reflect phase 1 + 2 complete').

6. Mark the saga complete in agentrail with --done. No --next-slug.

# Constraints

- Pure docs sweep. No code changes. ASCII-only.
- Keep the file's existing structure and tone.
- If you find inaccuracies that point to actual code problems (rather
  than docs drift), list them in a Recent changes entry as 'discovered
  during status-update sweep' rather than fixing the code in this step.

# Acceptance

- docs/status.md's Phase summary correctly reflects phase 1 + phase 2
  complete.
- All Crate-state rows match the actual repo states.
- Final saga commit pushed.
- agentrail complete --done.