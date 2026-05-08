Step 3 of saga "foundation-and-1130-bringup".

# Goal

Create the five framework crate skeletons in the `sw-langtools` GitHub
org. Each is a separate sibling repo. Each compiles standalone. Trait
surfaces follow `docs/design.md` Sec 2-6. Cross-crate deps use
`path = "../<crate>"` assuming sibling clones at
`~/github/sw-langtools/<crate>`.

The five crates:

1. `sw-isa-core` -- Architecture trait, address/endian/format/register
   primitives. `no_std`. (design.md Sec 2)
2. `sw-target-core` -- Target trait, CallingConvention, RegisterClasses,
   PrimType. Depends on sw-isa-core. `no_std`. (design.md Sec 3)
3. `sw-tir` -- Module/Function/Block/Op/Terminator/Type, IRBuilder
   (alloca + direct-SSA), pretty-printer, text-format parser.
   Depends on neither sw-isa-core nor sw-target-core. (design.md Sec 4)
4. `sw-tir-opt` -- pass driver + skeleton passes (constant fold, DCE,
   mem2reg). Depends on sw-tir. (design.md Sec 5)
5. `sw-codegen-core` -- Backend trait, Object types, regalloc framework,
   branch relaxation, frame layout, asm pretty-printer. Depends on
   sw-isa-core, sw-target-core, sw-tir. (design.md Sec 6)

# What "skeleton" means here

- Compiles cleanly: `cargo build` and `cargo test` exit 0.
- Trait surfaces present (signatures match design.md Sec 2-6 closely;
  small deviations OK if motivated).
- Method bodies may be `todo!()` or simplest-possible stubs. Don't
  over-implement -- step 4-6 (scaffolder + IBM 1130 work) will exercise
  these and force iteration.
- Per-crate `LICENSE` (MIT, matching the user's other repos), `COPYRIGHT`,
  `README.md` (one paragraph + crate role + status), `Cargo.toml` with
  `version = "0.1.0"`, `edition = "2024"`, MIT license, optional `serde`
  feature.
- Tests directory exists with at least one trivial test per crate (so
  `cargo test` actually runs something).

# Cross-crate deps

In each consumer's `Cargo.toml`:

```toml
[dependencies]
sw-isa-core = { path = "../sw-isa-core" }
```

Document this convention in each crate's README so a future contributor
knows to clone all five as siblings.

# Repo creation

For each crate:

1. `mkdir -p ~/github/sw-langtools/<crate> && cd ~/github/sw-langtools/<crate>`
2. `cargo init --lib --name <crate>`
3. Author `Cargo.toml`, `LICENSE`, `COPYRIGHT`, `README.md`, `src/lib.rs`,
   submodule files, `tests/<basic>.rs`.
4. `git init && git add -A && git commit -m "init: <crate> skeleton"`
5. `gh repo create sw-langtools/<crate> --public --source=. --push`

# Constraints

- ASCII-only in all .md files. Run `markdown-checker -f <file>` per file.
- Rust 2024 edition.
- Zero `cargo clippy --all-targets --all-features -- -D warnings` warnings
  per crate.
- Zero `cargo fmt --check` differences.
- `no_std` for sw-isa-core and sw-target-core (per design.md Sec 2 and 3).
- All trait method bodies that aren't trivially `todo!()` must be
  motivated by something the IBM 1130 work in steps 7-11 will exercise
  -- avoid speculative methods (per design.md Sec 1 tenet 4).

# Decisions to honour

From `docs/decisions.md`:

- D2: SSA with block parameters (sw-tir).
- D3: IRBuilder supports both alloca-naive and direct-SSA (sw-tir).
- D4: Distinct address-type newtypes per ISA (sw-isa-core defines the
  trait; concrete types live in `sw-{arch}-isa`).
- D6: ABI doc lives in markdown (`docs/abi.md` per `-target` crate;
  not here).

# Acceptance

- 5 repos exist on GitHub at `https://github.com/sw-langtools/<crate>`.
- 5 directories exist locally at `~/github/sw-langtools/<crate>/`.
- In each, `cargo build && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --check` all exit 0.
- Cross-deps resolve via path; pushing a single crate without the
  others would break, but local sibling layout works.
- This repo (`gen-isa`) gets a commit recording what was created. No
  generated crate code lives in this repo (per decisions.md Sec 1 -- 
  this repo is generators only).

# What to do this step

1. Read `docs/decisions.md` and `docs/design.md` Sec 2-6 carefully.
2. Read `docs/spec-format.md` for context on what the per-ISA crates
   will look like once generated; the framework crates' trait surfaces
   must accommodate them.
3. Create the 5 repos as described above. Order: sw-isa-core first
   (no deps); sw-target-core and sw-tir in parallel (one dep each); 
   sw-tir-opt (depends on sw-tir); sw-codegen-core last (depends on
   the other three).
4. Verify each crate's local build/test/clippy/fmt cleanly.
5. Push each to `sw-langtools` via `gh repo create`.
6. In `gen-isa`, record the work in `docs/status.md` (Recent changes
   entry; update the crate-state table).
7. `git add docs/status.md` (and any other gen-isa docs touched).
8. `git commit -m "framework-skeletons: 5 sw-langtools crates created"`
   in gen-isa.
9. `agentrail complete` with:
   - `--summary` describing what got created.
   - `--reward 1`.
   - `--actions` describing approach.
   - `--next-slug scaffolder-mvp`
   - `--next-prompt` defining step 4 (Rust binary in this repo
     `gen-isa/src/bin/scaffold.rs` or similar that, given an ISA name
     and output directory, emits the 5-crate quintet skeleton:
     `Cargo.toml`, `LICENSE`, `COPYRIGHT`, `README.md`, `src/lib.rs`
     stub, `tests/` dir. No spec parsing yet -- that's step 5. Test
     against tempdir; verify generated quintet compiles).
10. STOP.

# What NOT to do

- Don't write any code in this repo (`gen-isa`) beyond doc updates.
- Don't pre-implement codegen or regalloc; just trait shapes + stubs.
- Don't push to crates.io. Per design.md Sec 10, no publish until 3
  ISAs ship.
- Don't run `agentrail complete` until all 5 repos pass local
  build/test/clippy/fmt and are pushed.
