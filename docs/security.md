# Vaulten — Security Decisions and Accepted Risks

This document records the security decisions for Vaulten, the threats they address, and the risks
accepted for the MVP. It is the reference for Constitution Principles I–III. Detailed formats are in
[contracts/vault-file-format.md](contracts/vault-file-format.md) and
[contracts/ipc-commands.md](contracts/ipc-commands.md).

## 1. Threats

**In scope**

- An attacker who obtains a copy of `vault.db` and reads it. Protected by field-level encryption and
  Argon2id-based key derivation.
- An attacker who modifies `vault.db`. Detected for ciphertext, nonce, header, folder assignment, and
  timestamps (see section 3). Not detected for whole-row replay or row deletion (section 6).
- Repeated guessing of the master password through the application's unlock path. Slowed by a
  progressive delay (section 4).
- Passwords left on screen or in UI state after use. Masked by default; cleared on hide, folder change,
  and lock.
- Passwords left on the clipboard. Cleared after 60 seconds if unchanged (section 5).

**Out of scope for the MVP**

- A compromised operating system, malware running as the user, debuggers, or memory inspection of a
  running unlocked vault.
- Keyloggers and screen capture.
- Physical access while the vault is unlocked.
- Network attackers. The application makes no network connections (Principle IV).

## 2. Cryptography and key management

| Decision | Value | Rationale |
|----------|-------|-----------|
| Password KDF | Argon2id, version 0x13, 64 MiB, 3 iterations, parallelism 1, 16-byte salt | Memory-hard, current recommendation (RFC 9106); exceeds OWASP minimum |
| Key hierarchy | Random 256-bit vault key encrypts records; password-derived KEK only wraps the vault key | Password change re-wraps 48 bytes; no separate password hash is stored |
| AEAD | XChaCha20-Poly1305, fresh random 24-byte nonce per encryption | 192-bit nonce is safe with random nonces |
| Password verification | Successful authentication of the wrapped vault key | No verifier or hash stored, so no extra offline guessing oracle |
| Password normalization | Unicode NFKC before KDF; length counted in Unicode scalar values | Same visible password derives the same key across input methods |
| Master password policy | 12 to 1,024 characters; no character-class rules | Spec FR-006; cap bounds resource use |
| KDF parameters on read | Bounds-checked: memory 19,456–1,048,576 KiB; iterations 1–10; parallelism 1–8 | A tampered header cannot force excessive memory use |
| Secret memory | `zeroize` and `secrecy` in Rust; `Debug` output redacted; session dropped on lock | Limits lifetime and accidental disclosure |
| Custom cryptography | None | Principle III |

Parameter changes are security changes and require tests (Principle III).

### Record authenticated data

- Folder: `"vaulten/v1/folder" || id || created_at || updated_at`.
- Entry: `"vaulten/v1/entry" || id || folder_id || created_at || updated_at`.
- Wrap: `"vaulten/v1/wrap" || format_version || kdf parameters || salt`.

Decision: `folder_id`, `created_at`, and `updated_at` are bound into the entry and folder AAD, so changing a
record's folder assignment or timestamps fails authentication. The `folder_id` binding was approved as a
format change for version 1 before any release.

## 3. Storage

- Single SQLite file, `vault.db`, in the per-user application data directory. Path is resolved in Rust.
- Field-level encryption of all user content: folder names, platform names, URLs, identifiers,
  passwords. Only IDs, the entry-to-folder link, timestamps, and header metadata are plaintext.
- Plaintext metadata is accepted. Its integrity is protected by AAD (section 2). Its confidentiality is
  not. Record sizes, row counts, and timestamps are observable.
- Full-database encryption (SQLCipher) was rejected to avoid a native OpenSSL-based build dependency.
- Pragmas: `foreign_keys=ON`, `secure_delete=ON` (deleted ciphertext is overwritten), `synchronous=FULL`.
- Folder name uniqueness is enforced in the application layer because names are encrypted. The database
  cannot enforce it.
- No database-level repair or overwrite. A damaged vault is reported as `VAULT_CORRUPT`.

## 4. Unlock throttling and session

- Failures 1–4: no delay. From failure 5: wait `min(2^(n-5), 300)` seconds before the next accepted
  attempt. The counter resets on successful unlock.
- Throttled attempts do not run the KDF.
- The counter is held in memory. It resets on application restart.
  **Accepted risk (K6):** restarting the application resets the counter. Throttling protects the UI path
  only. Offline guessing against a copied file is bounded by the Argon2id cost and the 12-character
  minimum.
- Rust owns the session (vault key), the failure counter, and the inactivity timer. The frontend only
  reports activity and mirrors state (Principle V).
- Auto-lock after 5 minutes without user activity (keyboard or pointer input inside the application).
  Rust tracks activity with both a monotonic clock and a wall-clock time. Either clock reaching 5 minutes
  locks the vault. A wall clock that moved backward is ignored.
- **Decision (K4):** locking on OS lock or sleep is out of MVP scope. Constitution Principle "Auto-lock"
  was amended to match.
- Locking drops the session and zeroizes the vault key. The UI discards all unlocked state.
- The application starts locked on every launch.

## 5. Clipboard and generator

- Copy is performed in Rust. The password never enters the webview on copy.
- Clear after 60 seconds only if the clipboard still matches the copied value. Only a SHA-256 digest of
  the copied value is kept. Content placed by another application is never overwritten.
- Clear timer is independent of lock state.
- On Windows, clipboard history and cloud-sync exclusion hints are set. The Tauri clipboard plugin is not
  enabled, so the clipboard is not exposed to the webview.
- Generator: 20 characters, all four classes, OS CSPRNG, rejection sampling, Fisher–Yates shuffle. No
  user-configurable options.
- **Accepted risk:** clipboard clearing is best-effort. Clipboard managers and other applications can
  keep copies outside the app's control.

## 6. Accepted risks (MVP)

| Risk | Impact | Mitigation |
|------|--------|------------|
| Restart resets the unlock failure counter (K6) | Online guessing through the UI is not limited across restarts | Argon2id cost; 12-character minimum |
| JavaScript strings cannot be zeroized | A master password, revealed password, or generated password remains in webview memory until garbage collected | Uncontrolled master password field cleared on submit; short-lived UI state; full reset on lock |
| Whole-row replay: an older valid row (ciphertext, nonce, timestamps) can be restored | Entry content can revert to an earlier valid version | Not mitigated in MVP. Would need a version counter or an external anchor |
| Row deletion is not detected | An attacker with file write access can remove entries | Not mitigated in MVP |
| Ciphertext sizes, row counts, and timestamps are observable | Account inventory size can be estimated | Field-level encryption; no padding in MVP |
| Clipboard managers can retain copies | A copied password may persist beyond 60 seconds | Best-effort clear; exclusion hints on Windows |
| No memory locking (mlock) or swap protection | Secrets may reach the page file | Not in MVP |
| Single-instance enforcement absent | Two instances can open the same file; one window may show stale data | SQLite serializes writes |
| Master password recovery not available (spec Assumptions) | A forgotten master password means the vault cannot be opened | Documented for the user; no recovery path |

## 7. Scope decisions

- No master password change in MVP. The format allows a future re-wrap.
- No OS lock or sleep-triggered locking in MVP (K4).
- No import, export, sync, autofill, sharing, or cloud backup (Principle VIII).
- No nested folders; entries cannot move between folders.
- No functional search. Only the search field's location exists.
- No telemetry, update checks, or network access (Principle IV).
