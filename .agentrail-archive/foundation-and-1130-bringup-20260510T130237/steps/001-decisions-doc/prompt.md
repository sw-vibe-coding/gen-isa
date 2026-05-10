Step 1 of saga "foundation-and-1130-bringup".

# Goal

Write `docs/decisions.md` in THIS repo (`sw-vibe-coding/gen-isa`) that resolves
the open design decisions blocking code work in this saga. Commit it.

**Do NOT start any code in this step.** This step is documentation only.

# Format

The decisions doc should be 1-3 pages, one section per decision, with each
section stating:

- The question.
- The options considered (1-line each).
- The choice.
- The rationale (1-3 sentences).
- A "revisit if" trigger.

# Decisions to record

## From plan.md §14

- **D1. Workspace vs sibling repos.** Choice: **sibling repos** in their
  respective orgs.
  - `sw-langtools` -> `sw-isa-core`, `sw-target-core`, `sw-tir`, `sw-tir-opt`, `sw-codegen-core`, future `sw-riscv32i-*`
  - `sw-comp-history` -> `sw-{ibm1130,cdp1802,s370,s390}-{isa,target,codegen,asm,emulator}`
  - `sw-embed` -> `sw-cor24-isa` (existing) + `sw-cor24-{target,codegen}` (retrofit)
  - `sw-vibe-coding/gen-isa` -> this repo, generators only
  Rationale: keeps domain boundaries clean (history vs framework vs embedded
  toolchain). Cargo path-deps remain compatible for dev; switch to git deps
  later.
  Revisit if: cross-repo refactoring becomes a major drag or CI for cross-repo
  changes becomes painful.

- **D2. IR shape.** Confirmed: **SSA with block parameters** (per design.md §4
  and §6). Easier to lower than phi nodes; equivalent expressive power.

- **D3. Pilot frontend.** **Deferred.** Out of saga scope. Selected in a future
  phase-3 saga after phase-0 discovery.

- **D4. Existing IR-shaped tools to reuse.** **Deferred** to phase-0 discovery saga.

- **D5. Crate naming.** Confirmed: `sw-isa-core`, `sw-target-core`, `sw-tir`,
  `sw-tir-opt`, `sw-codegen-core`, `sw-{arch}-{isa,target,codegen,asm,emulator}`.

- **D6. Phase 4 ISA.** **Deferred.** Out of saga scope.

## From design.md §11

- **D1. Repo structure.** Same as plan-D1 above.

- **D2. IR shape.** Same as plan-D2 above.

- **D3. SSA construction.** Choice: `IRBuilder` supports **both** alloca-based
  naive emission AND direct-SSA emission. `mem2reg` (in `sw-tir-opt`) cleans
  up the alloca form. Frontends pick whichever style suits them.

- **D4. Distinct address types per ISA.** Choice: **yes**. Newtype wrappers
  (e.g. `WordAddress` for IBM 1130) prevent mixing 1130 word-addresses with
  COR24 byte-addresses. Small ergonomics tax accepted; protects against subtle
  bugs.

- **D5. Custom allocator on IBM 1130.** Choice: **defer**. Start with generic
  linear-scan + aggressive spill. Revisit if codegen quality is unusably bad.
  This is a teaching toolchain, not a perf project (see architecture.md §6.1).

- **D6. ABI doc format.** Choice: **plain markdown** (`docs/abi.md` per
  `-target` crate). No formal TOML descriptors until a tool actually consumes them.

- **D7. Conformance crate.** Choice: **yes, post-second-ISA**. Don't pre-build
  for one ISA.

- **D8. Frontend audit.** **Deferred** to phase-0 discovery saga.

## Saga-specific decisions (record these too)

- **Cross-repo dependency strategy.** In dev, cargo deps use
  `path = "../sw-isa-core"` etc., assuming sibling clone layout under the
  org's local checkout. Once the framework stabilizes (post 3rd ISA, per
  design.md §10), switch to git deps or crates.io paths.

- **Generator scope.** Hybrid: skeleton scaffolder + TOML-spec-driven for
  opcodes/registers/encoding/decoding/branch constants. Codegen patterns,
  ABI choices, and emulator semantics remain hand-written.

- **Saga exit criteria.** Round-trip + reference vectors pass for IBM 1130;
  asm round-trips; emulator runs >=1 curated program. Pilot frontend hello-world
  is OUT (separate future saga).

# What to do this step

1. Create `docs/decisions.md` with the structure above. ASCII-only (per
   `markdown-checker` requirement in `docs/process.md`).
2. `git add docs/decisions.md` and any `.agentrail/` changes.
3. `git commit -m "docs: lock design decisions for foundation+1130 saga"`
4. `agentrail complete` with:
   - `--summary` describing what got recorded
   - `--reward 1`
   - `--actions` describing approach
   - `--next-slug spec-format`
   - `--next-prompt` defining step 2 (design the ISA TOML spec format; document
     in `docs/spec-format.md`; provide sample specs for COR24 cross-check and
     IBM 1130 target; do NOT yet build the parser or generator)
5. STOP. Do not begin step 2.
