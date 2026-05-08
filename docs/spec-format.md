# ISA TOML Spec Format

> Status: draft 1, created 2026-05-08 in saga step `spec-format`. Stabilises
> as the generator parser is built (saga step `scaffolder-codegen`); breaking
> changes are expected during that step. After that, evolve via additive
> fields only.

This document specifies the TOML format that the hybrid scaffolder
(`gen-isa`) consumes to drive the mechanical parts of per-ISA `-isa` crate
generation. One spec file per ISA. The scaffolder reads the spec and emits
`opcode.rs`, `register.rs`, `encode.rs`, `decode.rs`, `branch.rs`,
`format.rs` (when needed), and `lib.rs` (the `Architecture` impl). Codegen
patterns, ABI, and emulator semantics remain hand-written and are out of
scope for this format.

Reference samples live in `docs/spec-examples/cor24.toml` and
`docs/spec-examples/ibm1130.toml`.

## Format choices and rationale

Each design choice below is locked. Subsequent steps may push back; that
push-back updates this section, not retroactive code.

| Choice | Decision | Rationale |
|---|---|---|
| One spec per ISA, or shared multi-ISA file | **One per ISA**. | Matches the sibling-repo layout (decisions.md Sec 1); each spec lives next to its target crate; no cross-ISA coupling at the spec layer. |
| TOML key naming | **snake_case throughout**. | TOML convention; matches Rust struct-field convention via `serde`. |
| Register pairs | **Top-level `[[fixed_pair]]` array** with `hi`/`lo`/`purpose`. | Maps directly to `sw_target_core::RegisterClasses::fixed_pairs()`. Avoids smearing pair info across two register entries. |
| Reserved / pseudo registers | **`class = "reserved" \| "pc" \| "sp" \| "fp" \| "flags"`** on the register entry. | Allocator and codegen want to filter by class anyway; centralises the role declaration. |
| Multiple opcodes with same mnemonic | **Distinct `name` (enum variant); same `mnemonic` allowed**. | COR24 has `AddReg` and `AddImm` both spelled `add`; the Rust enum needs unique variants but the assembler does not. |
| Instruction subforms (1130 Short/Long) | **Each format declared once**; opcode lists `formats = ["short", "long"]`. | Generalises to S/370's format zoo without nesting. The decoder uses a per-format `discriminator` to pick. |
| Length dispatch | **Either per-opcode (COR24-style) or first-word-bits (1130-style)**; the spec uses whichever fits the ISA. | Avoids forcing one mechanism on both ISA families. |
| Encoding style | **`encoding_style = "bit_fields" \| "rom_table"`**. ROM-table mode skips `encode.rs` / `decode.rs` generation. | COR24's encoding is hand-supplied via a ROM lookup; no other planned ISA shares this. The flag explicitly opts out of generation rather than forcing a fake bit-field model. |

## Top-level structure

```toml
[arch]              # identity and crate-naming
[memory]            # word size, endianness, address unit
[branch]            # short-branch range, pipeline-delay semantics
[encoding]          # encoding_style and dispatch info

[[register]]        # one per discrete register; repeat
[[register_bank]]   # alternative bulk-register declaration; repeat (rare)
[[fixed_pair]]      # allocator-visible register pairs; repeat

[[format]]          # one per instruction format; repeat
[[opcode]]          # one per Rust enum variant; repeat
```

## Sections

### `[arch]`

Identity. All fields required.

| Field | Type | Meaning |
|---|---|---|
| `display_name` | string | Human-readable, e.g. `"IBM 1130"`. Used in docs and `Architecture::NAME`. |
| `type_name` | string | Rust type that implements `Architecture`, e.g. `"Ibm1130"`. PascalCase. |
| `crate_slug` | string | Short slug for the crate-name suffix in `sw-{slug}-{isa,target,...}`, e.g. `"ibm1130"`. lowercase, kebab-acceptable. |

### `[memory]`

| Field | Type | Meaning |
|---|---|---|
| `address_unit` | enum | One of `"Byte"`, `"Word16"`, `"Word24"`, `"Word32"`. Drives `AddressUnit`. |
| `endian` | enum | One of `"Big"`, `"Little"`, `"ByteStream"`. Drives `Endian`. |
| `word_bits` | integer | Native word width in bits. |
| `min_instr_bytes` | integer | Smallest legal instruction in bytes. |
| `max_instr_bytes` | integer | Largest legal instruction in bytes. |

### `[branch]`

| Field | Type | Meaning |
|---|---|---|
| `short_offset_min` | integer | Most negative short-branch offset (inclusive). |
| `short_offset_max` | integer | Most positive short-branch offset (inclusive). |
| `max_short_branch_instructions` | integer | Convenience constant: how many max-length instructions fit in the short range. |
| `pipeline_delay_bytes` | integer | Bytes added to PC to compute the branch base. `0` for "next-instruction" semantics; non-zero for pipelined branches (COR24 = 4). |

### `[encoding]`

| Field | Type | Meaning |
|---|---|---|
| `style` | enum | `"bit_fields"` or `"rom_table"`. ROM-table opts out of `encode.rs`/`decode.rs` generation. |
| `length_dispatch` | enum | `"per_opcode"` (length determined by which opcode it is) or `"first_word_bits"` (length determined by inspecting bits of the first word). |

### `[[register]]`

One entry per named register.

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Lowercase mnemonic, e.g. `"acc"`, `"r0"`, `"x5"`. |
| `class` | enum | `"gpr"`, `"acc"`, `"ext"`, `"xr"`, `"reserved"`, `"pc"`, `"sp"`, `"fp"`, `"flags"`. |
| `index` | integer | Numeric identifier used in encodings. Unique per register. |
| `display_name` | string (optional) | Override for assembly output (e.g. `"FP"` instead of `"r3"`). |

### `[[register_bank]]` (rare, for register-rich ISAs)

Alternative to listing 32 individual registers.

| Field | Type | Meaning |
|---|---|---|
| `name_prefix` | string | e.g. `"x"` -> `x0`, `x1`, ..., `x{count-1}`. |
| `class` | enum | Usually `"gpr"`. |
| `count` | integer | Number of registers in the bank. |
| `index_base` | integer | Lowest numeric index (usually 0). |
| `zero_register` | integer (optional) | Index of a hardwired-zero register, if any (RISC-V x0). |

### `[[fixed_pair]]`

| Field | Type | Meaning |
|---|---|---|
| `hi` | string | Register name of the high half. |
| `lo` | string | Register name of the low half. |
| `purpose` | string | Documentation note (e.g. `"double-precision arithmetic"`). |

### `[[format]]`

One entry per instruction format. Length-only ISAs declare formats named
`"single_byte"`, `"two_bytes"`, `"four_bytes"` etc; richer ISAs use names
like `"short"`, `"long"`, `"rr"`, `"rx"`.

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Format identifier referenced from `[[opcode]]` entries. |
| `size_bytes` | integer | Total size of an instruction in this format. |
| `discriminator` | table (optional) | When `length_dispatch = "first_word_bits"`, this declares which bits identify the format. Fields: `offset` (from MSB of first word), `width`, `value`. |

#### `[[format.field]]`

Bit-field layout of the format's operands. Required when `encoding_style = "bit_fields"`.

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Field identifier (e.g. `"op"`, `"ra"`, `"disp"`). |
| `kind` | enum | `"opcode"`, `"reg"`, `"imm"`, `"disp"`, `"indirect"`, `"tag"`, `"reserved"`. |
| `offset` | integer | Bit offset from the MSB of the format (bit 0 = MSB). |
| `width` | integer | Field width in bits. |
| `signed` | bool (optional, default `false`) | Whether the field is sign-extended on decode. |
| `value` | integer (optional) | Required value for `"reserved"` fields (e.g. F-bit = 0 for short form). |

### `[[opcode]]`

One per Rust enum variant.

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Rust enum variant name in PascalCase (`"AddReg"`, `"LoadDouble"`). Unique per spec. |
| `mnemonic` | string | Assembler text mnemonic; lowercase by convention. May repeat across opcodes. |
| `value` | integer | Numeric opcode value (encoded into the `"opcode"` bit-field). |
| `format` | string | Single-format opcode: name of the format. |
| `formats` | array of string (optional) | Multi-format opcode: names of the formats this opcode appears in. Mutually exclusive with `format`. |
| `role` | string (optional) | Codegen-pattern hook tag, e.g. `"arith.add"`, `"mem.load"`, `"branch.cond"`, `"call"`, `"ret"`. Free-form for now; standardised when `scaffolder-codegen` lands. |

## Validation rules

The `scaffolder-codegen` step (saga step 5) will enforce these at parse time:

1. Every `[[opcode].format]` (or each entry in `formats`) names an existing `[[format]]`.
2. Every `[[fixed_pair]]` references existing register names.
3. Register `index` values are unique.
4. Opcode `name` values are unique. Opcode `value` values are unique within a format.
5. When `encoding_style = "bit_fields"`, every format covers exactly `size_bytes * 8` bits across its `[[format.field]]` entries with no gaps and no overlaps.
6. When `length_dispatch = "first_word_bits"`, every format declares a `discriminator`.

## Open questions for later steps

- Does the role tag need to be a closed enum (codegen-driven) or stay
  free-form? Decide in `scaffolder-codegen`.
- Do we need a `[[intrinsic]]` section for ISA-specific operations the
  IR exposes via `Op::Intrinsic`? Probably yes for IBM 1130 (e.g.
  `M` multiply with implicit ACC+EXT). Defer.
- How do we describe condition codes / flags consumed by branch
  instructions? Currently implicit in the codegen; spec may need a
  `[[condition]]` section once `sw-ibm1130-codegen` lands.

## Stability

This format is unstable until saga step `scaffolder-codegen` (step 5)
ships. Breaking changes during steps 3-5 are expected and welcome -- the
format is meant to be forged against actual generator code. Once that
step closes, evolve via additive fields only.
