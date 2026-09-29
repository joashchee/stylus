# Platform parity

Every macOS-specific mechanism Stylus uses, and what Windows and Linux
would need. Windows and Linux builds come after macOS covers the
milestone (build step 8 in Diskette's `docs/stylus-notes.md`).
Unresearched beyond naming the mechanism.

| Feature | macOS | Windows | Linux |
|---|---|---|---|
| Install location for local release builds | `~/Applications/Stylus.app` via `ditto` (`scripts/build-release.sh`) | n/a (installer) | n/a (package) |

Planned, add a row when each lands: file associations for `.ANS`, `.XB`,
`.BIN` and the rest, and Quick Look previews of `.ANS`.
