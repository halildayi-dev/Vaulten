# Vaulten — System Architecture

## 1. Overview

Vaulten is a local-first, fully offline desktop password manager.

- Target: Windows 10/11 x64.
- Desktop: Tauri 2.
- Backend: Rust stable, edition 2024.
- Frontend: React 19, TypeScript 6, Vite 8, Tailwind CSS 4.
- Database: SQLite through `rusqlite` with bundled SQLite.

No server, cloud, account, telemetry, or external network dependency.

Detailed security decisions and accepted risks: `docs/security.md`.
Interface contracts (IPC and storage format): `docs/contracts/`.

## 2. Architectural Principles

1. Rust owns all security-sensitive operations and business logic.
2. React is responsible for presentation and UI state only.
3. Tauri IPC is the only communication boundary between frontend and backend.
4. Secrets must never be persisted or logged in plaintext.
5. Use established cryptographic libraries; never implement custom cryptography.
6. Keep core Rust modules independent of Tauri.
7. Keep dependencies and exposed permissions minimal.

## 3. High-Level Architecture

```text
React / TypeScript
       |
       v
Frontend API Layer
       |
       v
Tauri IPC
       |
       v
Rust Commands
       |
       v
Vault Service
   /    |     \
Crypto Storage Other Services
               - Generator
               - Clipboard
```

React handles screens, forms, dialogs and presentation state.

Rust handles validation, encryption, persistence, sessions, throttling, auto-lock, password generation and clipboard operations.

## 4. Rust Backend

Core modules:

- `commands/`: Thin Tauri command adapters.
- `vault/`: Vault lifecycle, session, validation, throttling and auto-lock.
- `crypto/`: Key derivation, encryption, decryption and key management.
- `storage/`: SQLite connection, migrations and repository.
- `generator.rs`: Secure password generation.
- `clipboard.rs`: Clipboard access and timed clearing.
- `dto.rs`: IPC request and response types.
- `error.rs`: Internal errors and IPC error mapping.

### Dependency Rules

- Only `commands/` and `lib.rs` may depend on Tauri.
- `vault/` orchestrates core operations.
- `crypto/` must not access the database or global state.
- `storage/` must only handle opaque encrypted records and metadata.
- SQL must remain inside the storage repository.
- SQL queries must use parameterized statements.

## 5. Frontend

The frontend uses plain React with built-in state management.

- `src/api/` is the only layer allowed to call Tauri IPC.
- Feature hooks manage frontend data and UI behavior.
- React components must not invoke Tauri commands directly.
- Session state is managed through context and a reducer.
- No global state management library or router is required for the MVP.
- Vault data must not be stored in localStorage, sessionStorage, IndexedDB or cookies.

When the vault locks, the unlocked application subtree must be discarded. Revealed passwords, generated passwords and form drafts must not remain in UI state.

## 6. IPC Boundary

The frontend and Rust communicate through explicit Tauri commands.

The MVP exposes:

- 5 vault commands.
- 4 folder commands.
- 6 entry commands.
- 1 password generator command.
- 1 event: `vault://locked`.

Only explicitly approved commands may be exposed.

Every data operation requires an unlocked vault. Passwords must never be included in entry-list responses.

Secrets may cross IPC only when required by an explicit operation, such as unlocking, creating an entry, revealing a password or generating one.

IPC errors must not expose internal paths, SQL statements, stack traces or secret values.

## 7. Cryptography and Key Management

### Algorithms

- KDF: Argon2id, version 0x13.
- KDF parameters: 64 MiB memory, 3 iterations, parallelism 1.
- Salt: 16 random bytes.
- AEAD: XChaCha20-Poly1305.
- Key size: 256 bits.
- Nonce: 24 random bytes per encryption.
- Authentication tag: 16 bytes.

### Key Hierarchy

```text
Master Password
      |
      v
 NFKC Normalization
      |
      v
    Argon2id
      |
      v
 KEK (32 bytes)
      |
      v
 Unwrap Vault Key
      |
      v
 Vault Key (32 bytes)
      |
      v
 Encrypt / Decrypt Records
```

A random vault key encrypts individual records. The master password-derived key encrypts only the vault key.

Keys and secret buffers must remain in Rust memory and be zeroized when no longer needed.

### Authenticated Encryption

XChaCha20-Poly1305 must authenticate both ciphertext and associated data (AAD).

- Vault key wrapping authenticates the format version and KDF parameters.
- Folder records authenticate their record type, ID, and timestamps. Entry records also authenticate their folder ID. The exact encoding is in `docs/contracts/vault-file-format.md`.
- Every encryption operation uses a fresh random nonce.

No custom cryptographic construction is permitted.

## 8. Storage and Data Model

A single SQLite database named `vault.db` is stored in the operating system's per-user application data directory.

Main tables:

- `vault_meta`
- `folders`
- `entries`

SQLite schema versioning uses `PRAGMA user_version`.

Foreign keys are enabled, `secure_delete` is on, and `synchronous` is FULL. Folder deletion cascades to its entries.

All SQL operations belong to the storage repository. Multi-row operations use transactions.

### Data Protection

**Encrypted:**
- Folder names.
- Platform names.
- URLs.
- Account identifiers (usernames, email addresses, phone numbers).
- Passwords.
- Wrapped vault key ciphertext.

**Plaintext metadata:**
- Format version.
- KDF algorithm and parameters.
- KDF salt.
- Encryption nonces.
- Record IDs.
- Entry-to-folder relationships (authenticated, not encrypted).
- Creation and update timestamps (authenticated, not encrypted).

Ciphertext lengths and row counts are also observable.

Nonces are public encryption parameters, not secret keys. The wrapped vault key itself remains encrypted.

## 9. Session and Vault Lifecycle

Vault states:

```text
uninitialized → locked → unlocked
                     ↑       |
                     └───────┘
                       lock
```

- The application starts locked whenever a vault already exists.
- The vault key exists only in Rust memory while unlocked.
- Locking drops the session and zeroizes the vault key.
- Five minutes of inactivity triggers automatic locking.
- Rust owns the authoritative inactivity timer.
- Failed unlock attempts are throttled after the fifth failure.
- Throttle delays increase exponentially and are capped at 300 seconds.
- The failure counter is held in memory and resets on application restart. This is an accepted risk (see `docs/security.md`).

The frontend mirrors the backend session state; it is never the authority.

## 10. Password Generation and Clipboard

### Password Generator

- Fixed length: 20 characters.
- Must contain lowercase, uppercase, digits and symbols.
- Uses the operating system CSPRNG.
- Generated passwords remain temporary until explicitly saved.

### Clipboard

- Clipboard operations are performed in Rust.
- Passwords are cleared after 60 seconds if the clipboard still contains the copied value.
- Clipboard content changed by another application must never be overwritten.
- Locking the vault does not cancel a pending clipboard clear.

## 11. Security Boundaries

- No outbound network connections.
- No remote scripts, fonts or assets.
- Strict Content Security Policy.
- No unnecessary Tauri plugins or broad permissions.
- No filesystem or shell access exposed to the webview.
- No secret values in logs or error messages.
- No plaintext vault data in frontend persistence.
- No automatic vault repair or overwrite after corruption.

## 12. Testing Strategy

**Rust**
- Cryptographic round-trip and tamper detection.
- Key derivation and wrapping.
- Vault create, unlock and lock.
- Session enforcement and throttling.
- Automatic locking.
- SQLite persistence and transaction behavior.
- Plaintext-at-rest checks.
- IPC validation and secret redaction.
- Generator and clipboard behavior.

**Frontend**
- Vault screen transitions.
- Password masking and reveal/hide.
- Clearing sensitive UI state on lock.
- Form validation.
- Error handling and activity reporting.

**Integration**
- Full vault lifecycle.
- Folder and entry workflows.
- Offline operation.
- No plaintext in the database.
- No unexpected network connections.

Only relevant tests should run for each development task. Full verification is required before MVP release.

## 13. Scope

The MVP supports one local vault, one user, single-level folders and platform entries.

Cloud synchronization, import/export, autofill, sharing, password changes and functional search are outside the MVP scope. OS lock- or sleep-triggered locking and master password recovery are also outside the MVP scope.