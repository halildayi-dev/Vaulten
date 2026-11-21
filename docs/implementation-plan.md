# Implementation Plan: Password Vault MVP

**Branch**: `001-password-vault-mvp` | **Date**: 2026-11-01 | **Spec**: [requirements.md](requirements.md)

**Input**: Feature specification from `/requirements.md`

## Summary

Build Vaulten, a local-first, fully offline desktop password manager. The React/TypeScript layer
renders the unlock screen, folder panel, entry list, entry form, and the global search field
location. All security-relevant behavior lives in Rust: key derivation, encryption, vault
persistence, session and auto-lock state, unlock throttling, password generation, and clipboard
handling. The two layers communicate only through a small, explicit set of Tauri commands and one
event.

Technical approach (details in [dependencies.md](dependencies.md)):

- **Key hierarchy**: the master password (Unicode-normalized) is stretched with Argon2id into a
  key-encryption key. That key wraps a random 256-bit vault key with XChaCha20-Poly1305. The vault
  key encrypts each folder and entry as an individually authenticated record. The master password
  and keys are never written to disk.
- **Storage**: one SQLite file holding a metadata header (KDF parameters, salt, wrapped key) and
  tables of encrypted records. IDs, folder links, timestamps, and header metadata are plaintext; folder links and timestamps are authenticated through record AAD.
- **Session**: the unlocked vault key lives only in Rust memory (zeroized on drop). Rust owns the
  inactivity timer, so the 5-minute auto-lock holds even if the UI misbehaves.
- **Least exposure**: entry lists never contain passwords. Copying a password happens entirely in
  Rust; the UI receives a password only on an explicit reveal or when generating one.

## Technical Context

**Language/Version**: Rust (stable, pinned in `rust-toolchain.toml`, edition 2024); TypeScript 6.0.x
(typescript-eslint supports `<6.1.0`; TypeScript 7 is not yet supported by the lint toolchain);
Node.js 22.12+ LTS

**Primary Dependencies**: Tauri 2.12; React 19; Vite 8; Tailwind CSS 4; rusqlite (bundled SQLite);
RustCrypto `argon2`, `chacha20poly1305`; `zeroize`, `secrecy`; `arboard` (clipboard). Full list in
[dependencies.md](dependencies.md#dependencies).

**Storage**: Single local SQLite file `vault.db` in the OS per-user application data directory.
Field-level authenticated encryption; no database-level encryption extension.

**Testing**: `cargo test` (unit and integration, with fake clock and fake clipboard ports); Vitest +
React Testing Library for the frontend; `clippy -D warnings`; `cargo audit` and `npm audit`;
manual quickstart scenarios for the full app.

**Target Platform**: Windows 10/11 (x64) is the primary development and validation target. macOS and
Linux are not validated in the MVP; code must not use Windows-only APIs outside `clipboard.rs`.

**Project Type**: desktop-app (Tauri shell, Rust core, React presentation)

**Performance Goals**: Unlock completes in under 3 seconds on a typical laptop (spec SC-001 allows
10 seconds). Folder switching and entry list rendering feel instant (under 100 ms) for up to 1,000
entries.

**Constraints**: Fully offline with zero network access; no telemetry; no remote assets; webview
Content Security Policy blocks all external origins; no secrets in webview storage.

**Scale/Scope**: One user, one vault, up to roughly 10 folders and 1,000 entries (decrypted on
demand, so no indexing structures are needed in the MVP). About 6 screens/dialogs and 16 IPC
commands.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Pre-research | Post-design |
|-----------|------|--------------|-------------|
| I. Security First | Trade-offs resolve toward protecting secrets and are documented | PASS | PASS: every decision in dependencies.md records its security rationale and accepted risks |
| II. No Plaintext Secrets at Rest | Master password, passwords, keys never persisted in plaintext (DB, files, logs, webview storage) | PASS | PASS: only the AEAD-wrapped vault key is stored; all user content is encrypted; no logging; a raw-file test asserts no sentinel plaintext (see Testing Strategy) |
| III. Vetted Cryptography Only | No custom algorithms; standard primitives from maintained libraries | PASS | PASS: Argon2id, XChaCha20-Poly1305, OS CSPRNG via RustCrypto crates and `rand`; no custom constructions |
| IV. Local-Only and Offline | No server, account, telemetry, or network | PASS | PASS: no network crates, no updater, no remote assets; CSP blocks external origins; fonts bundled |
| V. Separated Presentation and Native Core | React = presentation only; Rust = crypto, keys, persistence, clipboard; no business rules in UI | PASS | PASS: frontend `api/` layer is the only IPC caller; auto-lock, throttling, and validation are authoritative in Rust (UI validation is cosmetic duplication) |
| VI. Minimal Tauri IPC Surface | Only needed commands; validated inputs; no secrets returned unless required; scoped capabilities | PASS | PASS: 16 explicit commands, each with its own permission; no fs, shell, clipboard, or opener plugins exposed to the webview; secrets returned only by `entry_reveal_password` and `password_generate` |
| VII. Test Security-Critical Paths | Crypto, KDF, round-trip, lock/unlock, verification, IPC validation, zeroization/redaction tested | PASS | PASS: dedicated test plan below covers each listed area |
| VIII. MVP Scope Discipline | Only spec-defined features | PASS | PASS: no search logic, import/export, sync, autofill, password change, or nested folders; deferred items listed in dependencies.md |

No violations. Complexity Tracking is empty.

## Architecture

### Responsibilities

| Concern | React/TypeScript (presentation) | Rust (native/core) |
|---------|--------------------------------|--------------------|
| Screens and layout | Unlock, create-vault, main shell, folder panel, entry list, entry form, dialogs, search field | None |
| Form validation | Immediate feedback only (empty, length) | Authoritative validation of every command input |
| Secrets | Holds the master password only while typing/submitting; holds a revealed or generated password only while displayed | Derives keys, holds vault key, encrypts/decrypts, copies to clipboard, generates passwords |
| Lock state | Mirrors state received from Rust; returns to unlock screen on `vault://locked` or any `VAULT_LOCKED` error | Single source of truth; inactivity timer; throttling |
| Persistence | None (no localStorage, sessionStorage, IndexedDB, or cookies for app data) | SQLite via the storage module |
| Clipboard | Calls `entry_copy_password`, shows confirmation | Writes, schedules clear, clears only if unchanged |

### Layering rule (Rust)

```text
commands (Tauri adapters)  →  vault (service)  →  crypto
                                     │            storage
                                     └──────────→ generator, clipboard (ports)
```

- `crypto`, `storage`, `vault`, `generator`, and the clipboard logic MUST NOT depend on `tauri`.
  Only `commands/` and `lib.rs` import Tauri types. This keeps core logic unit-testable without a
  webview and keeps the IPC layer thin.
- `storage` never sees plaintext or keys; it stores and returns opaque `(nonce, ciphertext)` blobs.
- `crypto` never touches the database or global state; it is pure functions over bytes.
- `vault` orchestrates: it owns the session, throttle, and auto-lock logic and is the only module
  that combines `crypto` and `storage`.

### IPC boundary

Sixteen commands and one event, defined in [contracts/ipc-commands.md](contracts/ipc-commands.md):

| Group | Commands |
|-------|----------|
| Vault | `vault_status`, `vault_create`, `vault_unlock`, `vault_lock`, `vault_report_activity` |
| Folders | `folder_list`, `folder_create`, `folder_rename`, `folder_delete` |
| Entries | `entry_list`, `entry_create`, `entry_update`, `entry_delete`, `entry_reveal_password`, `entry_copy_password` |
| Generator | `password_generate` |
| Event | `vault://locked` (Rust → UI) |

Rules:

1. Every data command fails with `VAULT_LOCKED` unless a session exists.
2. Each command is registered in an `AppManifest` in `build.rs` so Tauri generates a per-command
   `allow-*` permission. The single capability file lists exactly these permissions plus the minimal
   core event listen permission. No Tauri plugin (fs, shell, opener, clipboard, updater) is enabled.
3. Commands are `async` and run CPU or disk work on a blocking worker so the UI never freezes during
   the key derivation.
4. Errors cross the boundary as `{ code, field?, retryAfterSecs? }` with no internal detail
   (see Error Handling).
5. There is no search command; the search field is inert UI in the MVP.

### Frontend structure and state

Plain React with built-in state: `useReducer` plus context for session state, feature hooks for
data. No router (screens are driven by session state) and no global store library.

```text
AppRoot
 └─ VaultSessionProvider        status: loading | uninitialized | locked | unlocked | fatal
     ├─ CreateVaultScreen        (uninitialized)
     ├─ UnlockScreen             (locked; shows throttle countdown)
     └─ MainShell                (unlocked)
         ├─ Header: GlobalSearchField + LockButton
         ├─ FolderPanel          useFolders()  → list, selected id, create/rename/delete dialogs
         └─ EntryPanel           useEntries(selectedFolderId) → list, form dialog, row actions
```

- **Only `src/api/` imports `@tauri-apps/api`.** Components call hooks; hooks call typed `api/`
  functions. Components never call `invoke` directly, which keeps UI and application logic
  separate and lets tests mock a single module.
- **Reset on lock**: the `locked` transition replaces the whole `MainShell` subtree and discards all
  hook state (revealed passwords, generated passwords, form drafts, folder names, entries).
- **Activity reporting**: `useActivityReporter` listens for `pointerdown`, `pointermove`, and
  `keydown` on `window` while unlocked and calls `vault_report_activity` at most once every
  5 seconds.
- **Masked by default**: `PasswordField` renders `type="password"`; the revealed value lives only in
  component state and is cleared on hide, folder change, lock, or unmount.
- **Master password input** uses an uncontrolled field read from a ref on submit and cleared
  immediately afterward, so it is not retained in React state.
- **Styling**: Tailwind CSS 4 with an `@theme` block mapping [docs/design-tokens.md](design-tokens.md)
  (semantic colors, spacing, radii, component sizes). Light/dark follow the OS preference; the
  choice is not persisted. Inter is bundled locally through an npm package (no remote font fetch).
- **URLs are plain text** in the UI (never rendered as links), so no navigation or remote load is
  possible from stored data.

### Rust module structure

```text
src-tauri/src/
├── main.rs          # entry point, calls lib::run
├── lib.rs           # Tauri builder: state, command registration, auto-lock monitor task
├── error.rs         # AppError (thiserror) + IpcError mapping
├── dto.rs           # serde request/response types shared by commands
├── commands/        # thin adapters: parse → call service → map result/error
│   ├── vault.rs  folders.rs  entries.rs  passwords.rs
├── vault/
│   ├── service.rs   # create/unlock/lock, folder and entry operations, uses crypto + storage
│   ├── session.rs   # Session { vault_key, last_activity } with zeroize-on-drop
│   ├── throttle.rs  # failed-attempt counter and delay schedule (injectable clock)
│   ├── autolock.rs  # inactivity check logic (injectable clock)
│   └── validation.rs# length and emptiness rules, duplicate folder names
├── crypto/
│   ├── kdf.rs       # Argon2id parameters, derive key-encryption key
│   ├── aead.rs      # XChaCha20-Poly1305 seal/open with AAD
│   └── keys.rs      # vault key generation, wrap/unwrap, random bytes
├── storage/
│   ├── migrations.rs# schema version via PRAGMA user_version
│   ├── repo.rs      # VaultRepository: meta, folders, entries (opaque blobs)
│   └── mod.rs       # connection setup (pragmas)
├── generator.rs     # password generation (CSPRNG)
└── clipboard.rs     # ClipboardPort trait, arboard adapter, delayed-clear logic
```

### Data access layer

`storage::VaultRepository` is the only code that issues SQL. It exposes intention-revealing
methods over opaque records (for example `insert_entry(record)`, `list_entries(folder_id)`,
`delete_folder(id)`, `folder_entry_counts()`), uses parameterized statements exclusively, and wraps
multi-row changes in transactions. It is a trait implemented by the SQLite repository, so vault
service tests can run against an in-memory database. Folder deletion relies on
`ON DELETE CASCADE` in a single statement. The schema is in [data-model.md](data-model.md).

### Vault encryption and key derivation

Full byte-level format in [contracts/vault-file-format.md](contracts/vault-file-format.md).

```text
master password ──NFKC──► Argon2id(salt, m=64 MiB, t=3, p=1) ──► KEK (32 bytes)
vault key (random 32 bytes, OS CSPRNG) ──XChaCha20-Poly1305(KEK, nonce, AAD=header)──► wrapped key
record plaintext (JSON) ──XChaCha20-Poly1305(vault key, random 24-byte nonce, AAD=label‖id‖[folder_id]‖created_at‖updated_at)──► ciphertext
```

- Wrapping a random vault key (instead of encrypting records with the password-derived key) means
  a later password change only re-wraps 48 bytes. The MVP does not implement password change.
- **Password verification** is the authenticated decryption of the wrapped key. A wrong password
  fails the tag check; no separate verifier or password hash is stored.
- The wrap AAD binds the format version and KDF parameters, so tampering with them fails
  authentication. KDF parameters read from disk are bounds-checked before use to stop a tampered
  file from forcing excessive memory use.
- Record AAD binds each ciphertext to its table label, row ID, timestamps, and (for entries) folder ID,
  preventing a ciphertext from being accepted in a different role or row, or with altered metadata.
- All secrets in memory use `Zeroizing`/`SecretString`; their `Debug` output is redacted.

### Unlock, lock, and auto-lock flow

```text
launch ─► vault_status ─► uninitialized ─► CreateVaultScreen ─► vault_create ─► unlocked
                      └─► locked ───────► UnlockScreen ───────► vault_unlock ─► unlocked
unlocked ─► vault_lock | auto-lock | app exit ─► locked (session dropped, key zeroized)
```

`vault_unlock`:

1. If throttled, return `UNLOCK_THROTTLED` with `retryAfterSecs` without running the KDF.
2. Normalize and derive the KEK on a blocking worker; try to unwrap the vault key.
3. On failure: increment the failed counter, return `INVALID_PASSWORD` (plus the delay if now
   throttled). On success: reset the counter and create the session.

Throttle (FR-003a): failures 1-4 impose no delay; each failure `n ≥ 5` imposes a wait of
`min(2^(n-5) seconds, 300 seconds)` before the next attempt is accepted. The counter lives in Rust
memory, resets on success, and resets on app restart (documented limit: it protects the UI path;
offline guessing is bounded by the Argon2id cost, not by this counter).

Auto-lock (FR-005a): Rust stores `last_activity` as both a monotonic `Instant` and a wall-clock
`SystemTime`, updated by `vault_report_activity` and by every successful vault command. A monitor
task ticks every second; if either clock shows 5 minutes or more elapsed, it drops the session and
emits `vault://locked { reason: "auto" }`. Using both clocks covers sleep/hibernate (the monotonic
clock may not advance while the machine sleeps; the wall clock does). A wall clock that moved
backward is ignored. Closing the app ends the process, so the next launch starts locked (FR-005).

### Password generation and clipboard

- **Generation** (`generator.rs`): the OS CSPRNG; fixed 20-character length (spec minimum is 16);
  at least one lowercase, uppercase, digit, and symbol, the remainder drawn uniformly with
  rejection-sampled indices from the combined set, then shuffled with a Fisher–Yates shuffle. The
  command is stateless: nothing is stored until the user saves the entry (FR-030). Regeneration is
  another call; a repeat of a 20-character value is statistically impossible.
- **Copy** (`entry_copy_password`): Rust decrypts the entry, writes the password to the clipboard,
  and returns nothing. On Windows it requests exclusion from clipboard history and cloud sync.
  Only a SHA-256 digest of the copied value is kept in memory, along with a copy token.
- **Clear** (FR-026): a delayed task fires after 60 seconds. If the token is still the latest copy
  and the current clipboard text hashes to the stored digest, it clears the clipboard; otherwise it
  does nothing, so content from other applications is never overwritten. A new copy supersedes the
  previous timer. The timer is independent of lock state, so locking does not cancel the clear.

### Error handling strategy

- Internal `AppError` (thiserror) with variants: `Locked`, `NotInitialized`, `AlreadyInitialized`,
  `InvalidPassword`, `Throttled { retry_after }`, `Validation { field, rule }`, `NotFound`,
  `Storage`, `Crypto`, `Clipboard`, `Internal`.
- At the boundary, `AppError` becomes `IpcError { code, field?, retryAfterSecs? }`. Codes are stable
  strings (`VAULT_LOCKED`, `INVALID_PASSWORD`, ...). Storage, crypto, and internal details are
  collapsed into generic codes and never include SQL text, paths, key material, or user content.
- No logging of secrets; the MVP writes no persistent logs. Non-test code denies `unwrap`/`expect`
  and `dbg!`/`println!` through Clippy lints.
- The frontend owns all user-facing English text: a single `errorMessages` map keyed by code.
  `VAULT_LOCKED` from any call transitions the UI to the unlock screen.
- A startup failure (`VAULT_CORRUPT`, `STORAGE_ERROR`) shows a dedicated error screen with no
  attempt to repair or overwrite the vault file.

### Security-critical testing strategy

Principle VII requires tests for each of these; all are in `cargo test` unless noted.

| Area | Tests |
|------|-------|
| AEAD | Round-trip; each of ciphertext, nonce, AAD, key modified → open fails; two seals of the same plaintext differ (nonce uniqueness) |
| KDF | Same inputs → same key; different salt or password → different key; production parameters meet minimum thresholds (assert constants); tampered/out-of-bounds parameters are rejected; NFKC-equivalent inputs derive the same key (reduced-cost parameters used in tests) |
| Create/unlock/lock | Create → lock → unlock restores data; wrong password rejected; every data command returns `VAULT_LOCKED` while locked (table-driven over all commands); session is empty after lock |
| Throttle | Table-driven schedule with a fake clock: no delay through failure 4, 1 s after failure 5 doubling thereafter, cap at 300 s, reset on success; throttled attempts do not run the KDF |
| Auto-lock | Fake clock: locks at 5 minutes of inactivity; activity resets the timer; either clock exceeding the limit locks; backward wall-clock jump does not lock early or panic |
| No plaintext at rest | Create a vault and entries using distinctive sentinel strings (master password, platform name, URL, identifier, password, folder name); read the raw database file and assert none appear, in UTF-8 or UTF-16 |
| Tamper resistance | Altering any stored record or header byte (including folder assignment and timestamps) causes the affected read or unlock to fail cleanly (no panic, no partial plaintext) |
| IPC input validation | Empty/whitespace names, over-length fields, duplicate folder name (case-insensitive), missing required fields, short master password, unknown IDs |
| Redaction | `Debug`/`Display` of secret-bearing types and of `IpcError` never contain the secret value |
| Generator | Length and class coverage across 10,000 samples; samples are unique; no characters outside the allowed set |
| Clipboard | With a fake clipboard port and fake clock: clears after 60 s if unchanged; does not clear if another app changed it; a second copy supersedes the first timer; clear still happens after lock |
| Frontend (Vitest) | Masked by default; reveal/hide; revealed value cleared on lock and folder change; master password field cleared after submit; error-code mapping; `VAULT_LOCKED` returns to the unlock screen; activity reporter is throttled |
| Static checks | `cargo clippy -D warnings`; `#![forbid(unsafe_code)]` in the app crate; `cargo audit`; `npm audit --omit=dev` |

Manual validation of the full application is in [validation.md](validation.md).

### Development and build configuration

- `rust-toolchain.toml` pins stable with `clippy` and `rustfmt`; `Cargo.lock` and
  `package-lock.json` are committed; installs use `npm ci` and `cargo --locked`.
- Scripts: `dev`, `build`, `typecheck`, `lint`, `test` (Vitest), `test:rust`
  (`cargo test --manifest-path src-tauri/Cargo.toml`), `tauri` (Tauri CLI passthrough).
- `tauri.conf.json`: `withGlobalTauri` false; strict CSP (`default-src 'none'`, `script-src 'self'`,
  `style-src 'self'`, `font-src 'self'`, `img-src 'self' data:`, `connect-src ipc: http://ipc.localhost`,
  `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`); no remote URLs; no updater;
  `bundle.targets` limited to the Windows installer. The exact `connect-src` values for Tauri IPC
  must be confirmed when the project is scaffolded.
- Release profile: `lto`, `codegen-units = 1`, `strip`; devtools are not enabled in release builds.
- Lint configuration: `[lints.clippy]` denies `unwrap_used`, `expect_used`, `dbg_macro`,
  `print_stdout`, and `print_stderr` outside tests.
- Vite dev server fixed port 1420, `strictPort`, bound to localhost.
- SQLite pragmas on connection: `foreign_keys=ON`, `secure_delete=ON`, `synchronous=FULL`.

## Project Structure

### Documentation (this feature)

```text

├── implementation-plan.md                         # This file
├── dependencies.md                     # Phase 0: decisions, rationale, alternatives, dependencies
├── data-model.md                   # Phase 1: entities, validation, state, SQL schema
├── validation.md                   # Phase 1: setup and validation scenarios
├── contracts/
│   ├── ipc-commands.md             # Phase 1: command and event contract
│   └── vault-file-format.md        # Phase 1: on-disk format and crypto parameters
├── checklists/requirements.md
└── tasks.md                        # Phase 2 (/speckit-tasks, not created here)
```

### Source Code (repository root)

```text
.
├── package.json  package-lock.json  index.html
├── vite.config.ts  tsconfig.json  eslint.config.js  .prettierrc
├── rust-toolchain.toml
├── src/                                  # React + TypeScript (presentation only)
│   ├── main.tsx
│   ├── app/
│   │   ├── App.tsx
│   │   ├── VaultSessionProvider.tsx      # session reducer, context, lock event subscription
│   │   └── MainShell.tsx
│   ├── api/                              # the ONLY module that calls Tauri
│   │   ├── client.ts                     # invoke wrapper, error normalization
│   │   ├── vault.ts  folders.ts  entries.ts  passwords.ts
│   │   ├── events.ts                     # vault://locked subscription
│   │   ├── errors.ts                     # ApiError + error code type
│   │   └── types.ts                      # DTO types mirroring contracts/ipc-commands.md
│   ├── features/
│   │   ├── vault/                        # UnlockScreen, CreateVaultScreen, LockButton, useActivityReporter
│   │   ├── folders/                      # FolderPanel, FolderDialog, DeleteFolderDialog, useFolders
│   │   ├── entries/                      # EntryPanel, EntryRow, EntryFormDialog, PasswordField, useEntries
│   │   └── search/                       # GlobalSearchField (inert in MVP)
│   ├── components/ui/                    # Button, IconButton, Input, Dialog, EmptyState
│   ├── lib/                              # errorMessages.ts, throttle.ts
│   ├── styles/index.css                  # Tailwind import + @theme design tokens
│   └── test/                             # setup, API mocks
├── src-tauri/                            # Rust (native/core)
│   ├── Cargo.toml  Cargo.lock  build.rs  tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/
│   ├── src/                              # module tree shown above
│   └── tests/                            # integration: vault_flow.rs, no_plaintext_at_rest.rs
├── docs/                                 # existing design tokens and notes
└── specs/
```

**Structure Decision**: One Tauri application with a single Rust crate and a single frontend
package (the standard Tauri 2 layout). A separate core crate or workspace was considered and
rejected for the MVP because the module layering rule plus the "no `tauri` import" convention gives
the same testability with less structure; it can be extracted later without changing the module
interfaces.

## Complexity Tracking

No constitution violations to justify.
