# Research: Password Vault MVP

All technical unknowns for the plan are resolved below. Each decision records the security
rationale (Principle I) and rejected alternatives. Library versions were checked against the npm
and crates.io registries on 2026-11-01; exact API details must be re-verified when the project is
scaffolded because several RustCrypto crates are on recent releases.

## Decisions

### D1. Key derivation: Argon2id

- **Decision**: Derive a 32-byte key-encryption key from the NFKC-normalized master password with
  Argon2id, memory 64 MiB, 3 iterations, parallelism 1, 16-byte random salt. Parameters are stored
  in the vault header so they can be raised later.
- **Rationale**: Argon2id is the current recommended memory-hard password KDF (RFC 9106). These
  parameters exceed the OWASP minimum and should unlock in well under a second on a laptop, within
  the 3-second goal. Parallelism 1 keeps the cost predictable on low-core machines.
- **Alternatives**: scrypt and PBKDF2 (weaker against GPU/ASIC guessing; PBKDF2 is not memory-hard);
  higher memory (slower unlock on low-end machines with no MVP benefit).
- **Accepted risk**: parameters on disk are attacker-controllable, so they are covered by the wrap
  AAD and bounds-checked before use.

### D2. Authenticated encryption: XChaCha20-Poly1305

- **Decision**: XChaCha20-Poly1305 for both key wrapping and records, with a fresh random 24-byte
  nonce per encryption and associated data binding the context.
- **Rationale**: The 192-bit nonce makes random nonces safe without a counter, which would be
  fragile across crashes and restores. It is a standard AEAD with a maintained RustCrypto
  implementation.
- **Alternatives**: AES-256-GCM (96-bit nonce makes random nonces riskier at scale; fine on
  hardware with AES-NI but offers no MVP advantage); AES-GCM-SIV (extra complexity).

### D3. Key hierarchy: wrapped random vault key

- **Decision**: A random 256-bit vault key encrypts all records; the password-derived key only
  wraps the vault key. The wrapped key's successful authentication is the master password check.
- **Rationale**: No password hash or verifier is stored, avoiding an offline-guessing oracle other
  than the wrapped key itself. A future password change re-wraps 48 bytes instead of re-encrypting
  the database.
- **Alternatives**: Encrypt records directly with the password-derived key (password change
  becomes a full re-encryption); store a separate password hash (extra stored guessing target).

### D4. Storage encryption scope: field-level, all user content

- **Decision**: Encrypt entire folder and entry payloads (names, URLs, identifiers, passwords) as
  one authenticated blob per record. Only random IDs, the folder relation, and timestamps are
  plaintext.
- **Rationale**: Folder and platform names reveal which services a person uses, so they are
  protected too. Decrypting on demand is cheap at the expected scale (about 1,000 entries), so the
  loss of SQL-side sorting and searching is acceptable; sorting happens after decryption.
- **Alternatives**: SQLCipher (whole-file encryption; needs a bundled OpenSSL-based build, which
  complicates Windows builds and adds a native dependency); encrypting only passwords (leaks
  account inventory).
- **Accepted risk**: record sizes, counts, and timestamps are visible to someone with the file.

### D5. SQLite access: rusqlite (bundled) with a repository trait

- **Decision**: `rusqlite` with the bundled SQLite, one connection behind a mutex, parameterized
  statements only, schema versioned with `PRAGMA user_version` and embedded SQL migrations.
  Pragmas: `foreign_keys=ON`, `secure_delete=ON`, `synchronous=FULL`.
- **Rationale**: Smallest dependency set that satisfies "SQLite as the local store" without an
  async runtime or ORM. `secure_delete` overwrites deleted ciphertext. Bundling avoids depending on
  a system SQLite.
- **Alternatives**: `sqlx` (async, compile-time query checks; heavier than needed);
  `tauri-plugin-sql` (exposes SQL to the webview, which violates Principle VI); an ORM (unneeded
  abstraction).

### D6. Authoritative session, throttle, and auto-lock live in Rust

- **Decision**: Rust owns the session (vault key), failed-attempt counter, and inactivity timer.
  The UI only reports activity and mirrors state. Time is read through an injectable clock.
- **Rationale**: Principle V forbids UI state being the source of truth for security decisions. A
  frozen or compromised UI must not be able to keep the vault unlocked. Injectable time makes the
  rules testable without waiting.
- **Alternatives**: UI timer calling `vault_lock` (unsafe if the UI hangs); Rust-only activity
  inferred from commands (would not count pure scrolling or typing, contradicting FR-005a).

### D7. Auto-lock clock: monotonic plus wall clock

- **Decision**: Track `Instant` and `SystemTime`; lock if either shows at least 300 seconds since
  the last activity. Ignore a wall clock that moved backward.
- **Rationale**: The monotonic clock may not advance during system sleep on some platforms, which
  would leave the vault unlocked after a long sleep. The wall clock covers that case.
- **Alternatives**: Monotonic only (sleep gap); OS lock/sleep event hooks (the spec chose inactivity
  timeout, not OS-lock locking, and the hooks are platform-specific).

### D8. Throttle schedule and persistence

- **Decision**: No delay for failures 1-4; from the 5th failure on, wait `min(2^(n-5), 300)` seconds before the next attempt (1 s, 2 s, 4 s, ... capped at 300 s). The
  counter lives in memory and resets on restart or successful unlock.
- **Rationale**: Matches FR-003a (progressive delay, no permanent lockout or erasure). In-memory
  state avoids an unauthenticated writable counter in the database.
- **Accepted risk**: restarting the app resets it. Throttling protects the UI path only; offline
  guessing against a copied file is bounded by Argon2id cost and the 12-character minimum.
- **Alternatives**: Persisted counter (tamperable and adds schema); lockout or wipe (spec forbids).

### D9. Clipboard: arboard in Rust, no Tauri clipboard plugin

- **Decision**: Use the `arboard` crate directly from Rust. Do not enable the Tauri clipboard
  plugin. On Windows, set the clipboard-history and cloud-sync exclusion hints. Keep only a SHA-256
  digest of the copied value for the clear-if-unchanged check.
- **Rationale**: The plugin exposes clipboard read and write to JavaScript through capabilities,
  widening the IPC surface (Principle VI). A direct Rust call lets the password go from decrypted
  record to clipboard without ever entering the webview. The digest avoids holding a plaintext copy
  for 60 seconds.
- **Alternatives**: `tauri-plugin-clipboard-manager` (wider surface); keep plaintext for the
  compare (longer plaintext lifetime).
- **Accepted risk**: clipboard clearing is best effort; other clipboard managers can retain
  content outside the app's control.

### D10. Password generator

- **Decision**: 20 characters from four classes (lowercase, uppercase, digits, symbols) using the
  OS CSPRNG, at least one of each class, uniform selection with rejection sampling, Fisher-Yates
  shuffle. No user-configurable options in the MVP.
- **Rationale**: Satisfies FR-029 (16+ characters, all classes) with a standard construction and no
  custom cryptography. No options keeps the IPC surface and UI minimal (Principle VIII).
- **Alternatives**: Passphrase or configurable length and classes (valuable but outside the MVP
  spec).

### D11. Secret handling in memory

- **Decision**: `zeroize` and `secrecy` for the master password, derived keys, vault key, and
  decrypted payload buffers; `Debug` output redacted; session dropped on lock.
- **Rationale**: Reduces the lifetime and accidental disclosure of secrets in Rust memory.
- **Accepted risk**: JavaScript strings cannot be reliably zeroized, so secrets that must reach the
  UI (the master password on submit, a revealed or generated password) remain in webview memory
  until garbage collected. Mitigations: uncontrolled input cleared on submit, short-lived state,
  full UI state reset on lock. No memory locking (mlock) or swap protection in the MVP.

### D12. Unicode normalization of the master password

- **Decision**: Apply NFKC normalization before deriving the key; count length in Unicode scalar
  values; cap input at 1,024 characters.
- **Rationale**: Prevents the same visually identical password from deriving different keys across
  input methods (NIST SP 800-63B guidance). The cap prevents resource abuse. Normalization is part
  of format version 1 and cannot change without a version bump.
- **Alternatives**: No normalization (lockouts from input-method differences).

### D13. IPC permission model

- **Decision**: Register all commands through an `AppManifest` in `build.rs` so each gets its own
  generated `allow-<command>` permission, and list only those in the single capability file. Enable
  no Tauri plugins. Keep `withGlobalTauri` false.
- **Rationale**: Tauri 2 exposes unlisted app commands to all windows by default; the manifest makes
  the allowed surface explicit and reviewable (Principle VI).
- **Alternatives**: Default open command exposure (broader than needed).

### D14. Webview hardening

- **Decision**: Strict Content Security Policy; no remote scripts, styles, or fonts; Inter bundled
  from an npm package; URLs shown as text; no use of webview storage for app data; devtools off in
  release.
- **Rationale**: Principles II and IV and the constitution's technology constraints.
- **Open verification**: the precise `connect-src` entries for Tauri IPC on Windows
  (`ipc:` and `http://ipc.localhost`) must be confirmed during scaffolding.

### D15. Frontend state management: built-in React only

- **Decision**: `useReducer` plus context for session state and small feature hooks for folders and
  entries. No router, no global store, no data-fetching library.
- **Rationale**: Few screens and a single user. Fewer dependencies mean a smaller supply-chain
  surface for a security product (Principle VIII).
- **Alternatives**: Zustand/Redux (not needed); TanStack Query (caching semantics could retain
  decrypted data longer than intended).

### D16. IPC type sharing: hand-written TypeScript types plus a contract document

- **Decision**: Mirror DTOs manually in `src/api/types.ts`, documented in
  [contracts/ipc-commands.md](contracts/ipc-commands.md).
- **Rationale**: 16 commands do not justify a code generator dependency.
- **Alternatives**: `specta`/`tauri-specta` or `ts-rs` generation (reconsider if the surface grows
  or the types drift).

### D17. Crate layout: single Rust crate

- **Decision**: One crate with module layering; `crypto`, `storage`, `vault`, `generator`, and the
  clipboard logic do not import `tauri`.
- **Rationale**: Simplicity (Principle VIII). The boundary is enforced by convention and review and
  can be extracted into a workspace crate without interface changes.
- **Alternatives**: Workspace with a `core` crate (stronger compile-time isolation and faster core
  tests, more structure).

### D18. Tooling versions

- **Decision**: TypeScript 6.0.x because typescript-eslint's peer range is `<6.1.0` (TypeScript 7 is
  published but unsupported by the lint toolchain). Vite 8 requires Node 20.19+ or 22.12+; the
  installed Node 22.20 qualifies. Tailwind CSS 4 uses its Vite plugin and a CSS `@theme` block.
- **Rationale**: Keeps lint, typecheck, and test tooling mutually compatible at scaffolding time.

## Deferred and out of scope for the MVP

Per Principle VIII, these are intentionally not designed beyond leaving room in the format:

- Search logic and indexing (only the field's location is built)
- Master password change, vault export/import/backup, nested folders, moving entries
- Single-instance enforcement (two app instances could open the same file; no data loss expected
  because SQLite serializes writes, but state may be stale in one window)
- Memory locking, screen-capture protection, OS-lock-triggered locking
- Cross-platform validation for macOS and Linux, code signing, auto-update
- Password strength meter, generator options, URL opening

## Dependencies

Versions are the latest stable at planning time (2026-11-01). Re-verify at scaffolding; pin exact
versions through the lockfiles. Use the `rand` major version whose `rand_core` matches the one used
by `argon2` and `chacha20poly1305` to avoid duplicate incompatible trait versions.

### Rust (`src-tauri/Cargo.toml`)

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2.12 | Desktop shell, IPC, events |
| `tauri-build` (build) | 2.7 | Build script, app manifest for command permissions |
| `serde`, `serde_json` | 1.0 | DTOs and record payload serialization |
| `argon2` | 0.6 | Argon2id key derivation |
| `chacha20poly1305` | 0.11 | XChaCha20-Poly1305 AEAD |
| `rand` (or `getrandom`) | 0.10 | OS CSPRNG for salts, nonces, keys, generator |
| `zeroize` | 1.9 | Wipe secrets from memory |
| `secrecy` | 0.10 | Redacted secret wrappers |
| `unicode-normalization` | 0.1 | NFKC normalization of the master password |
| `rusqlite` (feature `bundled`) | 0.41 | SQLite access |
| `uuid` (feature `v4`) | 1.29 | Record IDs |
| `arboard` | 3.7 | Clipboard write, read-back, clear |
| `sha2` | 0.11 | SHA-256 digest for clear-if-unchanged check |
| `thiserror` | 2.0 | Error enums |

Dev-dependencies: `tempfile` (temporary database files for the no-plaintext test); a Tauri
`test` feature or mock runtime only if command-layer tests need it.

No networking crates, no logging framework, no Tauri plugins.

### JavaScript (`package.json`)

| Package | Version | Purpose |
|---------|---------|---------|
| `react`, `react-dom` | 19.3 | UI |
| `@tauri-apps/api` | 2.12 | Used only inside `src/api/` |
| `@fontsource-variable/inter` | 5.3 | Bundled local font |

Dev dependencies:

| Package | Version | Purpose |
|---------|---------|---------|
| `@tauri-apps/cli` | 2.12 | `tauri dev` / `tauri build` |
| `vite` | 8.3 | Dev server and bundler |
| `@vitejs/plugin-react` | 6.1 | React transform |
| `tailwindcss`, `@tailwindcss/vite` | 4.3 | Styling |
| `typescript` | 6.0 | Type checking |
| `@types/react`, `@types/react-dom`, `@types/node` | matching | Types |
| `vitest` | 5.0 | Unit tests |
| `jsdom` | 30 | DOM test environment |
| `@testing-library/react`, `@testing-library/user-event`, `@testing-library/jest-dom` | 16.3 / 14.6 / latest | Component tests |
| `eslint`, `typescript-eslint`, `eslint-plugin-react-hooks` | 10.14 / 8.72 / 7.1 | Linting |
| `prettier` | 3.9 | Formatting |

### Toolchain prerequisites

Rust stable (not installed on the planning machine; install through rustup), Microsoft C++ Build
Tools (MSVC), WebView2 runtime (included in Windows 11), Node.js 22.12+ and npm, Git.
