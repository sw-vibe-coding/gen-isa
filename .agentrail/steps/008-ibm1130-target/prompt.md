Step 8 of saga "foundation-and-1130-bringup".

# Goal

Bring up `sw-comp-history/sw-ibm1130-target` -- the second IBM 1130
crate. Implements `sw_target_core::Target` for the IBM 1130: ABI,
calling convention, register classes, type widths.

# Inputs

- `~/github/sw-langtools/sw-target-core/src/lib.rs` -- the trait surface
  (`Target`, `CallingConvention`, `RegisterClasses`, `PrimType`,
  `StackDirection`).
- `~/github/sw-comp-history/sw-ibm1130-isa` -- ISA description with
  `Opcode`, `Reg`, `Instruction`, `Architecture` impl. The target crate
  binds to this.
- `gen-isa/docs/spec-examples/ibm1130.toml` -- has the register table
  with classes (`acc`, `ext`, `xr`, `fp`, `pc`) and the `[[fixed_pair]]`
  declaration for ACC+EXT.
- `gen-isa/docs/porting-guide.md` Sec 5 -- ABI invention guidance.
- The ABI is **invented** -- IBM did not publish a calling convention
  for the 1130; we define our own.

# What to do

1. **Run the scaffolder** for the `-target` crate with `--spec`. Even
   though `-target` content is judgement-based, the scaffolder still
   emits Cargo.toml, LICENSE, README, src/lib.rs stub, src/abi.rs,
   src/classes.rs, src/types.rs, docs/abi.md, tests/smoke.rs.
   ```
   gen-isa scaffold --slug ibm1130 --display-name "IBM 1130" \
       --type-name Ibm1130 --out <tempdir> \
       --framework-path $HOME/github/sw-langtools \
       --spec docs/spec-examples/ibm1130.toml --force
   ```
   Use the `-target` subdirectory only; the `-isa` subdirectory in the
   tempdir overlaps the already-pushed real one and should be discarded.

2. **Hand-write the ABI**. Author `docs/abi.md` covering:
   - **Argument passing**: how the first N args are passed. Suggested:
     ACC for first scalar arg; remainder via fixed memory slots near
     the routine entry (1130 has no register-rich convention).
   - **Return value**: ACC for scalars; ACC+EXT pair for 32-bit
     `long`-style returns.
   - **Caller-saved vs callee-saved**: pick a sensible split.
     Suggestion: ACC, EXT, XR1 caller-saved; XR2 callee-saved; XR3
     reserved (frame pointer).
   - **Stack**: grows down. Frame pointer = XR3. Word-aligned (1130
     memory is word-addressed; stack alignment is 1 word = 1 address
     unit).
   - **Frame layout**: documented diagram (saved registers, locals,
     spill area, args).
   - **System call interface**: deferred / "use BSI to a stub" with
     explicit note that real BSI to a SIB-blob is out of scope.

3. **Implement `Target`** in `src/lib.rs`:
   ```rust
   pub struct Ibm1130Target;

   impl sw_target_core::Target for Ibm1130Target {
       type Arch = sw_ibm1130_isa::Ibm1130;
       type CallConv = abi::Ibm1130CallConv;

       fn type_width(ty: PrimType) -> usize { ... }    // 1=byte, 2=word, etc
       fn type_alignment(ty: PrimType) -> usize { ... }
       const STACK_GROWS: StackDirection = StackDirection::Down;
       const POINTER_BITS: u32 = 16;                    // word-addressed; ptr = 1 word = 16 bits
   }
   ```

4. **Implement `CallingConvention`** in `src/abi.rs`:
   ```rust
   pub struct Ibm1130CallConv;

   impl sw_target_core::CallingConvention<sw_ibm1130_isa::Ibm1130> for Ibm1130CallConv {
       fn arg_regs() -> &'static [Reg] { &[Reg::Acc] }
       fn return_reg() -> Reg { Reg::Acc }
       fn caller_saved() -> &'static [Reg] { &[Reg::Acc, Reg::Ext, Reg::Xr1] }
       fn callee_saved() -> &'static [Reg] { &[Reg::Xr2] }
       fn frame_pointer() -> Option<Reg> { Some(Reg::Xr3) }
       fn stack_pointer() -> Reg { ... }    // we may need to invent a "logical sp"
       const STACK_ALIGNMENT: usize = 1;    // word-aligned
   }
   ```
   Note: the 1130 has no hardware stack pointer. Our ABI defines XR2 or
   a memory location as the SP. Choose and document.

5. **Implement `RegisterClasses`** in `src/classes.rs`:
   ```rust
   pub struct Ibm1130RegClasses;

   impl sw_target_core::RegisterClasses<sw_ibm1130_isa::Ibm1130> for Ibm1130RegClasses {
       fn gpr() -> &'static [Reg] { &[Reg::Acc, Reg::Xr1, Reg::Xr2] }
       fn reserved() -> &'static [Reg] { &[Reg::Xr3, Reg::Iar] }
       fn fixed_pairs() -> &'static [(Reg, Reg)] { &[(Reg::Acc, Reg::Ext)] }
   }
   ```

6. **Implement `type_width` / `type_alignment`** in `src/types.rs`:
   - Pointer = 1 word (16 bits) since memory is word-addressed.
   - I8 / U8 / Bool = 1 word (no sub-word access on 1130; pad).
   - I16 / U16 = 1 word.
   - I32 / U32 = 2 words (use ACC+EXT pair).
   - I64 / U64 = 4 words (manual lowering by codegen).
   - Alignment = 1 word for everything.

7. **Add a smoke test** that exercises each impl: arg_regs returns the
   declared list, type_width returns expected values, fixed_pairs is
   ACC+EXT.

8. **Move to the final location**:
   ```
   mv <tempdir>/sw-ibm1130-target ~/github/sw-comp-history/sw-ibm1130-target
   cd ~/github/sw-comp-history/sw-ibm1130-target
   git init && git add -A && git commit -m "init: sw-ibm1130-target invented ABI + RegisterClasses"
   gh repo create sw-comp-history/sw-ibm1130-target --public --source=. --push
   ```

9. **Update `gen-isa/docs/status.md`** crate-state row.

10. **Commit gen-isa changes** and `agentrail complete` with:
    - `--summary` describing the ABI choices and tests.
    - `--reward 1`.
    - `--actions` describing approach.
    - `--next-slug ibm1130-codegen`.
    - `--next-prompt` defining step 9 (scaffold `sw-ibm1130-codegen`;
      implement `sw_codegen_core::Backend`; hand-write per-IR-op
      lowering for Add, Sub, Load, Store, Icmp, CondBranch, Call,
      Return; snapshot tests against curated TIR samples; push).

11. STOP.

# Constraints

- ABI is invented and documented in `docs/abi.md`. Future readers must
  be able to understand the choices.
- Generated `sw-ibm1130-target` must `cargo build / test / clippy -D
  warnings / fmt --check` clean.
- ASCII-only in `docs/abi.md`.
- This step does not touch `sw-ibm1130-isa` (already shipped).

# Acceptance

- `~/github/sw-comp-history/sw-ibm1130-target` exists and is pushed.
- `Target`, `CallingConvention`, `RegisterClasses` all implemented and
  unit-tested.
- ABI documented in `docs/abi.md`.
- `gen-isa/docs/status.md` updated.
