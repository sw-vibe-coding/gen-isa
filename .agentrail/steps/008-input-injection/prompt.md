Step 8 of saga 'forth-on-1130' -- make the kernel actually parse something.

# Goal

Inject a small FORTH source string into the kernel's input area via direct memory writes, run the kernel for N steps, and observe parsing happen. This is the first real interpretive step: the kernel should call FETCH, build a WORD, call DO to look up in the dictionary, and find a known entry.

# Why this approach

ACCEPT normally reads via LIBF DISK1 (stubbed). Three options were considered in saga step 6's RUNTIME-STATUS.md Sec 'Open questions / next-saga inputs' / item 3:

  (a) Inject a buffer at startup -- cheap, doesn't match historical behaviour. PICK THIS for now.
  (b) Build a small loadcards device -- closer to historical.
  (c) Real LIBF DISK1 -- substantial scope, deferred.

Option (a) keeps scope tight.

# What to do

1. Add a runtime test that:
   - Boots the kernel as before.
   - After START runs (~25 steps), reads workspace[-3] (the char pointer 'C') and SECT (the sector buffer base) to find where ACCEPT will read from.
   - Writes a few EBCDIC bytes packed into words at SECT's address: spell out a known dict word like 'HEX' or 'OR' or 'NEXT' (e.g. for 'HEX' the EBCDIC bytes are 0xC8 0xC5 0xE7 = 200, 197, 231, then a blank 0x40). Pack two bytes per word in the SECT buffer.
   - Sets workspace[-3] to point at the start of SECT, plus a sentinel for the end.
   - Runs another N steps; expects FETCH+CONVE+NEXT+DO to find the dict entry.
   - Asserts a specific kernel-state observable: e.g. XR3 points at the matched dictionary entry's address.

2. Iterate. Will probably surface more bugs in the kernel translation (e.g. the '2*WORD' arithmetic from LC.5 may bite here; the BNP I COM indirect-conditional from LC.6/LC.16 may be needed; one or more workspace-pointer-arithmetic bugs).

3. Update RUNTIME-STATUS.md with what worked / what broke.

4. Commit and push.

5. agentrail complete with --next-slug postmortem (step 9) defining the wrap-up.

# Constraints

- Use option (a): inject memory directly. Don't try to fix LIBF in this step.
- ASCII-only.
- If you discover a toolchain bug (emulator, asm, isa), commit it separately on the affected crate.
- If you discover a kernel-translation bug, fix it in kernel.asm + log in TRANSLATION-LOG.
- Don't translate the FORTH listing to executable form in this step -- input injection only.

# Acceptance

- A new runtime test exercises the parse-and-dispatch path.
- RUNTIME-STATUS.md updated with the observed behaviour.
- Toolchain or translation bugs (if any) have separate commits with rationale.

# Out of scope

- Real disk I/O (saga step 9+ candidate).
- Loading listing.fth into the running kernel.
- Adding asm features unless the runtime trace demands them.