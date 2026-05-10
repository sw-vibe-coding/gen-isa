Step 9 of saga "foundation-and-1130-bringup".

# Goal

Bring up sw-comp-history/sw-ibm1130-codegen -- the third IBM 1130
crate. Implements sw_codegen_core::Backend for the IBM 1130: hand-written
instruction selection lowering each TIR op to native instruction(s).

# Inputs

- ~/github/sw-langtools/sw-codegen-core/src/lib.rs -- the trait surface
  (Backend, plus regalloc/branch/frame/asm sub-modules).
- ~/github/sw-langtools/sw-tir/src/lib.rs -- TIR op set.
- ~/github/sw-comp-history/sw-ibm1130-isa -- Opcode, Reg, Instruction,
  Architecture impl.
- ~/github/sw-comp-history/sw-ibm1130-target/docs/abi.md -- the invented
  ABI; codegen must obey it (ACC = arg + return, ACC+EXT 32-bit pair,
  XR1 scratch, XR2 = SP, XR3 = FP, stack grows down).
- ~/github/sw-comp-history/sw-ibm1130-target/src/{abi,classes}.rs -- the
  CallingConvention and RegisterClasses impls to consume.
- gen-isa/docs/porting-guide.md Sec 6 (codegen organisation).

# What to do

1. Run gen-isa scaffold for the -codegen crate with --spec; keep only
   the sw-ibm1130-codegen subdir from the tempdir output.

2. Cargo.toml dependencies: sw-codegen-core, sw-tir, sw-isa-core,
   sw-ibm1130-isa, sw-ibm1130-target (path deps).

3. Implement sw_codegen_core::Backend in src/lib.rs:
   - struct Ibm1130Backend
   - type Target = sw_ibm1130_target::Ibm1130Target
   - lower(tir_function) -> Vec<sw_ibm1130_isa::Instruction> via the
     dispatch in src/select.rs.

4. Hand-write per-IR-op lowering in src/lower/:
   - arith.rs: Add (A short-form), Sub (S), Mul (M -> ACC+EXT pair),
     Div (D -> ACC quotient, EXT remainder).
   - mem.rs: Load (LD short or long depending on disp range), Store
     (STO).
   - cmp_branch.rs: Icmp (S then BSC with computed condition mask),
     CondBranch (BSC short for nearby targets, long otherwise),
     Branch (BSC unconditional).
   - call.rs: Call (BSI long; emit caller arg block in memory after the
     BSI per the ABI), Return (BSC indirect through return-address
     slot), prologue/epilogue per docs/abi.md Sec 8.

5. Snapshot tests with insta in tests/snapshot.rs covering at minimum:
   - i16 add: lower add(a, b) -> a + b
   - i16 load: lower load(ptr) -> ld
   - i16 store: lower store(ptr, val) -> sto
   - i16 cmp+cond branch: lower icmp eq + br_if
   - call+ret: lower a leaf function f(x: i16) -> i16 { ret x }
   Each snapshot is the disassembled instruction sequence.

6. cargo build / test / clippy -D warnings / fmt --check clean.

7. Move to ~/github/sw-comp-history/sw-ibm1130-codegen, git init, first
   commit, gh repo create --public --source=. --push.

8. Update gen-isa/docs/status.md crate-state row.

9. Commit gen-isa changes and agentrail complete with --next-slug
   ibm1130-asm, --next-prompt defining step 10.

# Constraints

- Hand-written selection patterns; do NOT reach for table-driven
  matchers (Burg/etc) -- per porting-guide Sec 6 we wait until 3+ ISAs
  are stable.
- Generated crate must build/test/clippy/fmt clean.
- ASCII-only in any docs files added.
- Do not modify sw-ibm1130-isa or sw-ibm1130-target (they are shipped).
- If the trait surface in sw-codegen-core needs to change, update
  sw-codegen-core in sw-langtools and document the change for the
  postmortem (saga step 12).

# Acceptance

- sw-comp-history/sw-ibm1130-codegen exists and is pushed.
- Backend implemented for the IBM 1130 covering Add, Sub, Load, Store,
  Icmp, CondBranch, Branch, Call, Return.
- Snapshot tests for at least the 5 IR-op flows above.
- gen-isa/docs/status.md updated.