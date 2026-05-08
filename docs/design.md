# Design: Layered Multi-ISA Toolchain

> Status: exploratory draft, not git-tracked. Created 2026-05-08.

This is the technical companion to `architecture.md` and `prd.md`. The architecture is layered (frontends → IR → codegen-core → per-ISA trio). This document specifies the trait surface and crate boundaries at each layer.

## 1. Design tenets

1. **Frontends are written once.** Every language frontend emits `sw-tir` IR. No frontend ever touches an ISA-specific type.
2. **Native types first, traits second.** Each `-isa` and `-target` crate's primary API is concrete and ergonomic. Trait `impl`s are an additional adapter layer for the codegen and tooling that needs uniformity.
3. **No cross-layer leakage.** ISA quirks stay in `-isa` and `-codegen`. ABI choices stay in `-target`. IR is innocent of all of it.
4. **One concrete consumer per trait method.** Every method must be motivated by a current consumer, not "in case someone needs it".
5. **Round-trip is the test.** For every ISA, `encode(decode(bytes)) == bytes` and `disasm(asm) == asm`. Non-negotiable.
6. **Most abstraction-disruptive ISA drives the design first** (see `architecture.md §6.1` for the explicit axis). IBM 1130 forces `AddressUnit` and register-poor allocator design; CDP1802 forces PC-via-register-selector. S/370 and S/390 are bigger but stress different axes (format zoo, access registers) that are better tackled on a mature foundation.

## 2. `sw-isa-core`

Pure ISA description. No ABI, no codegen, no IR awareness.

```rust
// sw-isa-core/src/lib.rs
#![no_std]

pub mod address;
pub mod endian;
pub mod format;
pub mod register;

pub trait Architecture {
    type Opcode: Copy + core::fmt::Debug + Eq + Mnemonic;
    type Register: Copy + core::fmt::Debug + Eq + register::RegisterId;
    type Instruction: Copy + core::fmt::Debug;
    type Address: address::AddressType;
    type Format: format::FormatInfo;

    const NAME: &'static str;
    const ENDIAN: endian::Endian;
    const ADDRESS_UNIT: address::AddressUnit;
    const WORD_BITS: u32;
    const MAX_INSTR_BYTES: usize;
    const MIN_INSTR_BYTES: usize;

    fn decode(bytes: &[u8], pc: Self::Address)
        -> Result<(Self::Instruction, usize), DecodeError>;
    fn encode(insn: &Self::Instruction, out: &mut [u8])
        -> Result<usize, EncodeError>;
    fn disassemble(insn: &Self::Instruction, w: &mut dyn core::fmt::Write)
        -> core::fmt::Result;
}

pub trait Mnemonic { fn mnemonic(&self) -> &'static str; }
```

```rust
// sw-isa-core/src/address.rs
pub enum AddressUnit { Byte, Word16, Word24, Word32 }

pub trait AddressType: Copy + Ord {
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
    fn step(self, n: i64) -> Self; // step by AddressUnit, not bytes
}

// sw-isa-core/src/endian.rs
pub enum Endian { Big, Little, ByteStream }

// sw-isa-core/src/format.rs
pub trait FormatInfo: Copy + core::fmt::Debug {
    fn size_bytes(&self) -> usize;
}

// Default for length-only-discriminating ISAs.
pub enum Length { B1, B2, B3, B4, B6, B8 }
impl FormatInfo for Length { /* … */ }

// sw-isa-core/src/register.rs
pub trait RegisterId: Copy {
    fn index(self) -> u32;
    fn name(self) -> &'static str;
}
```

For ISAs with format complexity beyond length (S/370 RR/RX/RS/SI/SS), the `-isa` crate defines its own `enum Format` and implements `FormatInfo`.

## 3. `sw-target-core`

Describes the ISA in its compiler-target role. **Depends on `sw-isa-core`** but adds ABI-and-allocator concerns the ISA description has no opinion on.

```rust
// sw-target-core/src/lib.rs
#![no_std]
use sw_isa_core::Architecture;

pub trait Target {
    type Arch: Architecture;
    type CallConv: CallingConvention<Self::Arch>;

    /// Type widths in *bytes* (or in address-units if word-addressed).
    fn type_width(ty: PrimType) -> usize;
    fn type_alignment(ty: PrimType) -> usize;

    const STACK_GROWS: StackDirection;
    const POINTER_BITS: u32;
}

#[derive(Copy, Clone, Debug)]
pub enum PrimType { I8, I16, I32, I64, U8, U16, U32, U64, Ptr, Bool }

pub enum StackDirection { Down, Up }

pub trait CallingConvention<A: Architecture> {
    /// Argument registers in order. Args beyond this go on the stack.
    fn arg_regs() -> &'static [A::Register];
    /// Register holding scalar return value.
    fn return_reg() -> A::Register;
    /// Caller-saved registers (volatile across calls).
    fn caller_saved() -> &'static [A::Register];
    /// Callee-saved registers (preserved across calls).
    fn callee_saved() -> &'static [A::Register];
    /// Frame pointer, if used.
    fn frame_pointer() -> Option<A::Register>;
    /// Stack pointer.
    fn stack_pointer() -> A::Register;
    /// Stack alignment at call boundary.
    const STACK_ALIGNMENT: usize;
}

/// Register classes for the allocator.
pub trait RegisterClasses<A: Architecture> {
    /// Registers usable for general-purpose integer values.
    fn gpr() -> &'static [A::Register];
    /// Registers reserved (cannot be allocated).
    fn reserved() -> &'static [A::Register];
    /// Register pairs that must be allocated together (1130 ACC+EXT).
    fn fixed_pairs() -> &'static [(A::Register, A::Register)] { &[] }
}
```

### Why `Target::Arch` instead of `Target: Architecture`?

A: Architecture is a *what is the ISA* description. Target is a *how do we use the ISA* description. The same ISA could in principle have multiple ABIs (different calling conventions, different alignment); `Target` should be free to vary independently. Composition (`type Arch`) > inheritance (`Target: Architecture`).

## 4. `sw-tir` — Target-Independent Representation

A small, opinionated SSA-like IR that frontends emit and codegens consume. **No `sw-isa-core` dep.**

### Top-level shape

```rust
// sw-tir/src/lib.rs
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
    pub globals: Vec<Global>,
}

pub struct Function {
    pub name: String,
    pub linkage: Linkage,
    pub signature: FnSig,
    pub blocks: Vec<Block>,
    pub locals: Vec<Local>, // stack slots
}

pub struct Block {
    pub label: BlockId,
    pub params: Vec<Value>,        // SSA block parameters (φ-equivalents)
    pub instrs: Vec<Instr>,
    pub terminator: Terminator,
}

pub enum Op {
    // arithmetic
    Add(Value, Value), Sub(Value, Value), Mul(Value, Value),
    SDiv(Value, Value), UDiv(Value, Value),
    SRem(Value, Value), URem(Value, Value),
    // bitwise
    And(Value, Value), Or(Value, Value), Xor(Value, Value),
    Shl(Value, Value), Shr(Value, Value), AShr(Value, Value),
    // memory
    Load { addr: Value, ty: Type },
    Store { addr: Value, val: Value },
    Alloca { ty: Type, count: Value },
    // calls
    Call { callee: Value, args: Vec<Value> },
    // conversions
    SExt(Value, Type), ZExt(Value, Type), Trunc(Value, Type),
    // comparisons (produce i1)
    Icmp { pred: ICmp, lhs: Value, rhs: Value },
    // address-of, gep-equivalent for compound types
    AddrOf(Symbol),
    StructIndex { base: Value, field: u32 },
    ArrayIndex { base: Value, idx: Value, elem_size: u32 },
    // generic intrinsic — codegen lowers per-target
    Intrinsic { name: &'static str, args: Vec<Value> },
}

pub enum Terminator {
    Return(Option<Value>),
    Branch(BlockId, Vec<Value>),               // unconditional with block args
    CondBranch { cond: Value, t: BlockId, t_args: Vec<Value>,
                                f: BlockId, f_args: Vec<Value> },
    Unreachable,
}
```

### Type system

```rust
pub enum Type {
    I1, I8, I16, I32, I64,
    Ptr(Box<Type>),
    Array(Box<Type>, u32),
    Struct(StructId),
    Void,
}
```

The frontend chooses concrete IR types using **target-independent** sizes. Codegen maps `I32` → 32-bit on whatever the ISA's bit-width unit is. For the IBM 1130 (16-bit words), `I32` lowers to a two-word pair. The frontend never sees this.

### What's deliberately not in IR

- **No vectors, no SIMD, no atomics, no floating-point** until a frontend needs them.
- **No "byte" vs "word" distinction.** IR is in abstract bits. Codegen handles addressing.
- **No exception/unwinding metadata.** Languages that need it lower it to explicit branches in IR.
- **No debug info.** Add later if useful.

### Why SSA-ish?

SSA gives the optimisations and register-allocator scaffolding the cleanest input. "SSA-ish" because we use **block parameters** instead of φ nodes — easier to lower, equivalent in expressive power.

## 5. `sw-tir-opt`

Standard target-independent passes:

- Constant folding
- Dead-code elimination
- Mem2reg (alloca → SSA value)
- Basic inliner (size-budgeted)
- Peephole rewrites on IR
- Branch threading
- Common subexpression elimination

Each pass is a `fn(&mut Module) -> bool` (returns whether it made changes). Passes are composed via a simple driver. **No target awareness whatsoever.**

## 6. `sw-codegen-core`

Scaffolding for lowering IR to instructions. Generic over `T: Target`.

```rust
// sw-codegen-core/src/lib.rs
use sw_target_core::Target;
use sw_tir::{Function, Module};

pub trait Backend {
    type Target: Target;

    /// Lower an entire module to a per-target object representation.
    fn lower_module(&self, m: &Module) -> Result<Object<Self::Target>, BackendError>;

    /// Emit assembly text.
    fn emit_asm(&self, m: &Module, w: &mut dyn core::fmt::Write)
        -> Result<(), BackendError>;
}

/// What the backend produces: a sequence of native instructions plus
/// fixups (label-relative branches that need resolving in a final pass).
pub struct Object<T: Target> {
    pub functions: Vec<EmittedFunction<T>>,
    pub globals: Vec<EmittedGlobal>,
}

pub struct EmittedFunction<T: Target> {
    pub name: String,
    pub instrs: Vec<<T::Arch as sw_isa_core::Architecture>::Instruction>,
    pub fixups: Vec<Fixup>,
    pub frame_size: u32,
}
```

### Sub-modules

- `sw-codegen-core::regalloc` — generic linear-scan and graph-colouring allocators, parameterised by `RegisterClasses<A>`. ISAs with quirky register files (1130 ACC+EXT pair, x86 fixed-register ops) supply their own allocator if needed.
- `sw-codegen-core::branch` — branch relaxation. Reads `BranchRange` from the `-isa` crate; rewrites short branches to long branches when targets are out of range.
- `sw-codegen-core::frame` — function prologue/epilogue templates parameterised by `CallingConvention`.
- `sw-codegen-core::asm` — assembly text pretty-printer driven by `Architecture::disassemble`.
- `sw-codegen-core::fixup` — resolve label-relative addresses post-allocation.

### What lives in `sw-{arch}-codegen`

- **Instruction selection patterns.** IR `Add` → COR24 `add ra,rb` is a one-liner. IR `Mul` on 1130 is "load to ACC, MPY with EXT involvement, store result pair" — much messier. All of this is per-ISA.
- **Target-specific peepholes.** "On COR24, `mov ra,ra` is a nop, drop it." "On RISC-V, fold consecutive additions of small immediates."
- **Lowering of intrinsics** — `intrinsic("syscall", ...)` becomes whatever the ISA does for syscalls.
- **Choice of allocator.** Most targets use the generic linear-scan; 1130 likely needs a custom one.

### Branch relaxation, concretely

`sw-cor24-isa::branch` already gives us `can_short_branch`, offset min/max, max-instruction-bytes. `sw-codegen-core::branch` consumes these plus the encoded byte offsets to produce the final branch encoding:

1. Lay out instructions assuming all branches are short.
2. Walk pairs of (branch, target); for any out-of-range branch, rewrite to a long-branch sequence.
3. Re-layout. Repeat to fixed point (usually one or two iterations).

Each ISA's `-isa` crate provides the constants; the relaxation algorithm is shared.

## 7. Per-ISA crate template

```
sw-{arch}-isa/
  src/
    lib.rs       ← re-exports + impl sw_isa_core::Architecture
    opcode.rs    ← Opcode enum + From<u{N}> + mnemonic + format-of-opcode
    register.rs  ← name table + parser + impl RegisterId
    encode.rs    ← encoding helpers
    decode.rs    ← decoding helpers
    branch.rs    ← range constants
    format.rs    ← (optional) Format enum + impl FormatInfo
  tests/
    roundtrip.rs ← encode∘decode = id, decode∘encode = id

sw-{arch}-target/
  src/
    lib.rs       ← Target struct + impl sw_target_core::Target
    abi.rs       ← CallingConvention impl
    classes.rs   ← RegisterClasses impl
    types.rs     ← PrimType width / alignment table
  docs/
    abi.md       ← human-readable ABI documentation (we invent these)

sw-{arch}-codegen/
  src/
    lib.rs       ← Backend impl
    select.rs    ← instruction-selection patterns
    lower/       ← per-IR-op lowering modules
    peephole.rs  ← target peepholes
    intrinsics.rs ← target-specific intrinsic lowering
  tests/
    snapshot.rs  ← golden-file tests: IR sample → asm
```

## 8. Worked example: bringing up IBM 1130

### `sw-ibm1130-isa`

Two `Instruction` variants (Short, Long), word-addressed. `decode` reads big-endian u16 words.

```rust
pub struct Short { pub op: Opcode, pub tag: u8, pub disp: i8 }
pub struct Long  { pub op: Opcode, pub tag: u8, pub indirect: bool, pub address: u16 }
pub enum Instruction { Short(Short), Long(Long) }

pub struct Ibm1130;
impl sw_isa_core::Architecture for Ibm1130 {
    type Address = WordAddress;
    const ADDRESS_UNIT: AddressUnit = AddressUnit::Word16;
    const ENDIAN: Endian = Endian::Big;
    const WORD_BITS: u32 = 16;
    const MIN_INSTR_BYTES: usize = 2;
    const MAX_INSTR_BYTES: usize = 4;
    // …
}
```

### `sw-ibm1130-target`

ABI is invented. Suggested:

- `int` = 16-bit (1 word). `long` = 32-bit (2 words). Pointer = 16-bit (1 word, since memory is word-addressed).
- Stack grows down. Frame pointer = XR3.
- Args: passed via fixed memory slots near the routine entry (1130 has no register-rich convention). Return value in ACC (or ACC+EXT for `long`).
- Allocator: ACC+EXT treated as a single "allocatable long pair"; XR1, XR2 allocatable short. XR3 reserved as FP. Register file is so small that the allocator will spill heavily — that's fine for a teaching toolchain.

### `sw-ibm1130-codegen`

Per-IR-op lowering. The interesting bits:

- `Add(a, b)` → `LD a; A b; STO result` (load ACC from a, add b, store).
- `Mul(a, b)` → `LD a; M b; STO result_lo (EXT); STO result_hi (ACC)` if 32-bit; the two-word result lives in ACC+EXT.
- `Load { ptr }` → `LD *(ptr_word_address)`.
- `CondBranch { cond, t, f }` → compare result already in ACC from the cond; emit `BSC` (branch on selected condition).

Instruction selection patterns are written by hand; we don't have GCC/LLVM-style table-driven selection. For 12 languages × 5 ISAs the hand-written table is manageable.

## 9. How frontends shed their target-awareness

Today (assumption): each compiler emits COR24 assembly directly.

### Refactor pattern

1. Identify the compiler's "lower to assembly" boundary.
2. Replace the assembly emitter with an IR builder.
3. Replace assembly-specific decisions (register choice, calling-convention details) with IR-level decisions (let codegen handle them).
4. Verify: the compiler's existing tests still pass, but now via `IR → cor24-codegen → assembly`.

The cost is per-language but linear. After all languages are refactored, **adding ISA N+1 costs zero language changes**.

### What if a frontend can't be refactored?

Some frontends may have been written so close to the metal that an IR boundary doesn't fit cleanly. Acceptable fallback: keep that frontend COR24-only and exclude it from multi-ISA support. Document explicitly which languages support which ISAs.

## 10. Trait surface evolution policy

`sw-isa-core`, `sw-target-core`, `sw-codegen-core`, `sw-tir` will all see breaking changes during initial multi-ISA bring-up. Policy:

- No crates.io publication until **three** ISAs and **two** languages have shipped end-to-end. This forces real-world friction into the design before we lock it.
- Breaking changes require a coordinated bump across all consumers. Workspace-internal versioning makes this cheap.
- After v1.0, breaking changes follow semver; cadence is annual at most.

## 11. Open decisions

- **D1. Repo structure.** Single workspace under `sw-embed/sw-toolchain/`, or separate sibling repos as today? Tooling and CI are easier with a workspace; user's existing layout is sibling repos. Lean toward **workspace for the new layered crates** (`sw-isa-core`, `sw-target-core`, `sw-codegen-core`, `sw-tir`, `sw-tir-opt`) plus the per-ISA `{isa,target,codegen}` trio; per-ISA repos may stay siblings if user prefers. **User to confirm.**
- **D2. IR shape.** Three-address SSA with block parameters (proposed) vs stack-machine (simpler frontends, harder codegen) vs hybrid. Lean toward SSA — pays off in optimisation passes and register allocation.
- **D3. SSA construction.** Frontends produce SSA directly, or post-pass converts to SSA? SSA-direct simplifies the IR consumer; harder for frontends. Pragmatic answer: provide an `IRBuilder` that supports both alloca-based naive emission and direct-SSA emission; `mem2reg` cleans up the alloca form.
- **D4. Distinct address types per ISA?** Proposed: yes — newtype wrappers prevent mixing 1130 word-addresses with COR24 byte-addresses. Cost: small ergonomics tax.
- **D5. Custom allocator on IBM 1130.** Use generic linear-scan with very aggressive spilling, or write a custom allocator that knows about ACC+EXT? Punt: start with generic + spill, measure, write custom if codegen quality is poor.
- **D6. ABI documentation format.** Plain `docs/abi.md` per `-target` crate, or formal `.toml` ABI descriptors? Prefer plain markdown until a tool actually consumes the format.
- **D7. Conformance tests.** A single workspace `conformance/` crate that loads each `(Arch, Target, Backend)` triple and runs the same end-to-end test against each? Probably yes; file as a follow-up once two ISAs are up.
- **D8. Frontend audit.** What is the actual current shape of each compiler? **Discovery task in plan.md §3.** Resolve before sequencing the language-refactor work.
