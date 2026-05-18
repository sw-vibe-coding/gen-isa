Step 10 of saga 'forth-on-1130' -- get the kernel to actually match an injected word against the dictionary.

# Goal

Inject the EBCDIC bytes for a known dictionary entry name (e.g. 'HEX', 'NEXT') and assert that after running the kernel for N steps:
- The kernel's WORD slot (workspace[2]) contains the packed FORTH internal code for the injected name.
- The kernel's data stack reflects whatever the matched primitive does (HEX pushes a hex value; NEXT recurses; etc.).

This is the first 'kernel parses and executes a FORTH word' test.

# Constraints

- ASCII-only.
- Don't add asm features unless the runtime trace demands it.
- LIBF still stubbed.

# Acceptance

- The new test asserts on workspace[2] holding the packed FORTH code for the injected name (e.g. for 'HEX' = /110E in name-high).
- All previously-passing tests still pass.
- RUNTIME-STATUS.md updated.