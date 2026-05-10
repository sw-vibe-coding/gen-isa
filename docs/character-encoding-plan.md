# Character Encoding on the IBM 1130 (and across our toolchain)

> Status: design doc. Created 2026-05-10 during saga step 11
> (`ibm1130-emulator`) after the user flagged that strings.asm's
> "ASCII H, I, J" comments are historically inaccurate. No code
> implementing this plan yet -- this captures the intended approach
> so a future "character I/O" or "string library" saga can land it
> without re-litigating the design.

## 1. The actual historical situation

There is no single "1130 character encoding". The 1130 used different
codes for different devices, and EBCDIC was *not* the on-the-wire
code for most of them in the era we care about. Rough taxonomy:

| Device                      | On-the-wire code                   |
| --------------------------- | ---------------------------------- |
| 1442 / 2501 card reader     | Hollerith (12-bit punch pattern)   |
| 1442 / 1132 / 1403 printer  | A "PTT" or printer-translation code; per-device |
| 1131 / 1132 console printer | A 1131-specific code               |
| Paper tape (1134/1055)      | PTTC (Perforated Tape Trans. Code) |
| Disk (1131/2310/2315)       | Whatever the program writes        |

EBCDIC is the **System/360** native encoding. The 1130's contemporary
S/360 host ecosystem read/wrote EBCDIC, and IBM's compilers and
utilities for the 1130 (FORTRAN, Assembler) translated to/from it on
the boundary. So a 1130 *running an IBM compiler that talked to a
host* would deal with EBCDIC at translation time, but the bytes
sitting in 1130 memory were typically in a 1130-specific code packed
two-per-word.

**Bottom line:** "EBCDIC" is the right answer at the file/system
boundary; per-device codes are right at the I/O point; the in-memory
representation is per-program.

## 2. What we actually want for our toolchain

The user's framing (paraphrasing the original message):

> ASCII for source listings and human-readable artifacts; EBCDIC
> when stored "in" the 1130 (because that's the historically
> realistic encoding for tools that talked to the S/360 ecosystem);
> Hollerith / per-device codes are a punch-card / printer concern
> that maps to/from "Latin text" on the I/O boundary.

So three encoding layers:

1. **Source / listing layer (ASCII)** — what we write in `.asm`
   files, what tests print, what humans read. Always ASCII (or UTF-8
   when we expand to Unicode-aware diagnostics).
2. **Storage layer (EBCDIC, when chosen)** — what an emulator stores
   in memory when running a program that "expects EBCDIC". This is
   per-program: a program can use any encoding internally; we just
   provide an EBCDIC table for programs that want it.
3. **Device layer (Hollerith / PTTC / printer code)** — encoded only
   at I/O boundaries (card reader, printer, paper tape). Maps to/from
   ASCII or EBCDIC depending on the device and program convention.

## 3. Concrete plan

### Phase A: opt-in EBCDIC table, no behavior change

Add `sw-ibm1130-runtime` (a new sibling crate to `-emulator`) or a
`encoding` module inside `-emulator` itself, containing:

- `pub fn ascii_to_ebcdic(byte: u8) -> Option<u8>` — single-byte
  translation table (256 entries).
- `pub fn ebcdic_to_ascii(byte: u8) -> Option<u8>` — inverse.
- `pub fn ascii_str_to_ebcdic_words(s: &str) -> Vec<u16>` — pack two
  EBCDIC bytes per word, big-endian (matches 1130 memory layout).

Decision: house in `sw-ibm1130-runtime` rather than `-emulator`,
because asm and codegen also benefit (an asm `DC.STR "HELLO"`
directive that emits packed-EBCDIC is the natural endpoint).

### Phase B: asm directive `DC.STR` for EBCDIC string literals

Extend `sw-ibm1130-asm` to support:

```text
GREETING:  DC.STR  "HELLO"   ; packs as EBCDIC, two bytes per word
```

Mechanics: lexer recognises `"..."` string literal; encoder
translates each byte through `ascii_to_ebcdic`; emits packed words.
Listing output shows the source ASCII alongside the EBCDIC hex.

Also support `DC.ASTR "..."` for "raw ASCII per byte, two per word"
when the user explicitly wants ASCII storage (useful for testing
the encoding pipeline itself).

### Phase C: device-aware I/O on the emulator

Currently `XIO` is a no-op stub. When we build out I/O:

- Console printer (1132): on `XIO write`, look up the byte through
  `printer_code_to_ascii` (or `ebcdic_to_ascii` if the program
  wrote EBCDIC) and append to a virtual print buffer that tests
  can inspect.
- Card reader: on `XIO read`, translate ASCII (the source-layer
  representation of the punched card image) to Hollerith, then to
  whatever the program's read routine expects.
- Punched-card output: inverse.

The emulator carries a configurable "device encoding map" so a
program can declare `// device 0x42 uses EBCDIC; device 0x07 uses
ASCII-passthrough`.

### Phase D: round-trip discipline

Tests that traverse the full pipeline:

- `assemble("GREETING: DC.STR \"HELLO\"") -> bytes`
- `bytes -> emulator -> XIO write -> printer buffer -> ASCII`
- assert printer-buffer string equals `"HELLO"`.

This catches accidental encoding drift and is the contract for
"strings work end-to-end."

## 4. What strings.asm should say *now*

Until Phase A lands, strings.asm uses arbitrary 16-bit words
(`0x0048`, `0x0049`, `0x004A`) and the `; H, I, J` comments imply
ASCII. That's misleading. The accurate state is:

- The values are **arbitrary 16-bit words**. The demo doesn't
  depend on what they encode.
- If interpreted as ASCII, the bytes happen to spell `H I J`.
- A real 1130 program writing characters would more likely use
  EBCDIC -- but we have no EBCDIC table yet, and the demo's point
  is the LOOP+SENTINEL pattern, not the encoding.

The strings.asm comment will be updated to make this explicit.

## 5. Saga placement

This work doesn't belong in the current
`foundation-and-1130-bringup` saga. Suggested placement:

- **Phase A** (encoding tables): a small "1130-runtime" follow-up
  saga of 1-2 steps, after the current saga's postmortem.
- **Phase B** (asm `DC.STR`): same saga; landing alongside the
  encoding tables.
- **Phase C** (device I/O): part of a "1130 peripheral simulation"
  saga that also brings up timer interrupts, disk I/O, etc.
- **Phase D** (round-trip tests): one test commit per phase as the
  pieces become testable.

## 6. References for when this work lands

- `sw-comp-history/demo-ibm-1130-system` (MIT, code-reuse OK):
  peripheral simulator with disk geometry / card / printer device
  models. Best source for device-layer encoding tables.
- `sw-comp-history/ibm-1130-rs` (MIT, code-reuse OK): in-memory
  layout conventions, packed-character handling.
- IBM 1130 Subroutine Library, C26-5929-4: PTT and printer
  conversion subroutines (`PRNT1`, `WRTY0`, etc.).
- `sw-comp-history/S1130` (read-only reference, no code reuse):
  more complete EBCDIC and Hollerith tables.

## 7. Open questions to resolve at design time

- Do we ship a single canonical EBCDIC code page (CP-037 US/Canada
  is the historical default for 1130-era tooling), or do we make
  the code page configurable per-program?
- Does the asm `DC.STR` literal default to EBCDIC or ASCII? My lean
  is EBCDIC by default (the "historically accurate" default); ASCII
  via `DC.ASTR` for explicit override.
- Do we need a "trace mode" in the emulator that prints memory as
  ASCII (translated) for debugging? Probably yes -- cheap to add.

These are explicitly **not decided here**. The phase-A saga will
relitigate them when it has running code in front of it.
