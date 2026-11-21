# Quickstart: Password Vault MVP

How to set up, run, and validate the MVP once it is implemented. This is a run and validation guide;
implementation steps belong in `tasks.md`. Contracts: [ipc-commands.md](contracts/ipc-commands.md),
[vault-file-format.md](contracts/vault-file-format.md). Data: [data-model.md](data-model.md).

## Prerequisites

| Tool | Requirement |
|------|-------------|
| Rust | Stable via rustup, with `clippy` and `rustfmt` (not currently installed on the planning machine) |
| MSVC build tools | Microsoft C++ Build Tools (Desktop development with C++) |
| WebView2 | Runtime (preinstalled on Windows 11) |
| Node.js / npm | Node 22.12 or newer, npm 10+ |
| Git | Any recent version |

## Setup

```bash
npm ci
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
```

## Run

```bash
npm run tauri dev      # development app with hot reload
npm run tauri build    # release build and Windows installer
```

## Automated checks

```bash
npm run typecheck
npm run lint
npm run test                                              # frontend (Vitest)
npm run test:rust                                         # Rust unit + integration tests
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo audit --file src-tauri/Cargo.lock
npm audit --omit=dev
```

All must pass. The Rust suite includes the security-critical tests listed in the plan (crypto
tamper tests, throttle and auto-lock schedules, locked-state enforcement for every command, the
no-plaintext-at-rest check, redaction, clipboard clearing).

## Manual validation scenarios

Run with a fresh application data directory (delete `vault.db` to reset).

### 1. Create, lock, and unlock (User Story 1; FR-001 to FR-007, FR-005a)

1. Launch the app. Expected: create-vault screen (no vault yet).
2. Enter an 11-character master password. Expected: rejected with a length message.
3. Enter a 12+ character password and confirm it. Expected: the main interface appears.
4. Click Lock. Expected: unlock screen; no folder or entry text visible.
5. Enter a wrong password. Expected: error message; input masked and cleared.
6. Enter the correct password. Expected: main interface returns.
7. Fail the unlock 5 times in a row. Expected: submit disabled with a countdown; a correct password
   works once the wait ends.
8. Unlock, then leave the app untouched for 5 minutes. Expected: the unlock screen appears on its
   own. Moving the mouse once a minute for 6 minutes must not lock it.
9. Close and reopen the app. Expected: unlock screen (locked on launch).

### 2. Folders (User Story 2; FR-008 to FR-012)

1. Create folders `Work` and `Personal`; try an empty name, a 101-character name, and `work`.
   Expected: the three invalid attempts are rejected.
2. Rename `Personal` to `Home`; select each folder. Expected: selection highlights and the content
   area follows.
3. Delete a folder containing entries. Expected: the dialog states the entry count; after
   confirming, the folder and its entries are gone.

### 3. Entries (User Story 3; FR-013 to FR-021)

1. In a selected folder, add an entry with name and password only. Expected: it appears; URL and
   identifier are empty.
2. Try saving without a name, and without a password. Expected: rejected, field indicated.
3. Edit each field; leave the password blank on edit. Expected: updates appear; the stored password
   is unchanged. Enter a new password. Expected: replaced.
4. Delete an entry after confirming. Expected: removed.

### 4. Reveal, hide, copy (User Story 4; FR-022 to FR-026)

1. Expected: all passwords are masked on first display.
2. Reveal, then hide. Expected: shown, then masked again. Reveal, then switch folders or lock.
   Expected: masked again or gone.
3. Copy a password and paste it elsewhere. Expected: it matches; a confirmation appears.
4. Wait 60 seconds without copying anything else, then paste. Expected: the clipboard is empty.
5. Copy a password, then copy different text from another application, wait 60 seconds. Expected:
   the other text is still on the clipboard.

### 5. Generate (User Story 5; FR-027 to FR-030)

1. Open the add form and generate. Expected: a 20-character password with all four classes.
2. Regenerate several times. Expected: a different value each time.
3. Cancel the form. Expected: nothing stored. Generate, save, then reveal. Expected: the saved
   value matches.

### 6. Search location (User Story 6; FR-031, FR-032)

Expected: a search field is visible at the top of the main interface; typing causes no error and
shows no results or a coming-soon notice.

### 7. Offline and no plaintext at rest (FR-033 to FR-035; constitution II and IV)

1. Disable all network adapters, then run scenarios 1-5. Expected: identical behavior.
2. Create entries using recognizable text, lock, then search the raw bytes of `vault.db` (for
   example with a hex or text search tool) for the master password, platform name, URL, identifier,
   password, and folder name. Expected: none found.
3. While the app runs, confirm with a network monitor that it opens no outbound connections.
4. Confirm the installed app makes no requests to remote origins (all assets local) and that the
   webview storage for the app contains no vault data.

## Expected outcomes

All scenarios pass, all automated checks pass, and the success criteria SC-001 through SC-007 in
[requirements.md](requirements.md) are met.
