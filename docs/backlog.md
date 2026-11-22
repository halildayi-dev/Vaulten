# Vaulten — Development Backlog

## Task Status

- `TODO` — Not started
- `IN_PROGRESS` — Currently being implemented
- `BLOCKED` — Waiting for a decision or dependency
- `REVIEW` — Implementation finished, awaiting review
- `DONE` — Acceptance criteria passed and user approved

## Recommended Order

Sequential execution is the default (see `docs/workflow.md`). Follow the order below unless a task's
dependencies allow otherwise.

DOC-002 → REPO-001 → REPO-002 → SETUP-001 → SETUP-002 → SETUP-003 → SETUP-004 → CORE-001 → CORE-002
→ CRYPTO-001 → CRYPTO-002 → CRYPTO-003 → STORAGE-001 → STORAGE-002 → Future phases

---

## Phase 1 — Project Documentation

### DOC-001: Review Project Documentation
- **Status:** DONE
- **Dependencies:** None
- **Objective:** Review the project instructions, architecture, and workflow documents for consistency.
- **Scope:**
  - Review `CLAUDE.md`, `docs/architecture.md`, `docs/workflow.md`.
  - Identify missing or contradictory decisions.
  - Record decisions in `docs/security.md` and `docs/contracts/`.
- **Files:** Review only; updates to `docs/`, `.specify/memory/constitution.md`, `specs/001-password-vault-mvp/`.
- **Acceptance Criteria:**
  - No unresolved architectural contradictions.
  - Security decisions are documented.
  - Development workflow is clear.
- **Verification:** Manual document review.
- **Security Notes:** Accepted risks are recorded in `docs/security.md`.

### DOC-002: Finalize Development Backlog
- **Status:** DONE
- **Dependencies:** DOC-001
- **Objective:** Establish the initial development roadmap and task structure.
- **Scope:**
  - Review this backlog.
  - Adjust task order and dependencies.
  - Keep this file as the single source of truth for task status.
- **Files:** `docs/backlog.md`.
- **Acceptance Criteria:**
  - Initial development tasks are agreed upon.
  - Task dependencies are clear.
- **Verification:** Manual review and approval.
- **Security Notes:** None.

---

## Phase 2 — Repository Cleanup and Git

### REPO-001: Remove Legacy Spec Kit Structure
- **Status:** DONE
- **Dependencies:** DOC-002
- **Objective:** Remove Spec Kit and obsolete specification artifacts after their relevant decisions have been migrated.
- **Scope:**
  - Migrate the constitution to `docs/constitution.md` with its principles intact.
  - Migrate the SQL schema from `specs/001-password-vault-mvp/data-model.md` to `docs/data-model.md`.
  - Migrate the dependency list and decisions D1–D18 from `research.md` to `docs/dependencies.md` and `docs/security.md`.
  - Migrate the manual validation scenarios from `quickstart.md` to `docs/validation.md`.
  - Verify that `docs/contracts/` matches `specs/001-password-vault-mvp/contracts/`, then remove the `specs/` copies.
  - Decide on the empty file `docs/ideas/offline-mod.md`.
  - Remove `.specify/` and the `speckit-*` skills in `.claude/skills/`.
  - Remove only approved legacy files.
- **Files:** `.specify/`, `specs/`, `.claude/skills/speckit-*`, `docs/` (new files listed above).
- **Acceptance Criteria:**
  - No required architectural or security decision is lost.
  - Spec Kit-specific files and configuration are removed.
  - The new documentation remains sufficient to guide development.
- **Verification:** Inspect the final repository structure and Git diff.
- **Security Notes:** Confirm that no security decision in `research.md` or `plan.md` exists only in a deleted file.

### REPO-002: Establish Git Baseline
- **Status:** DONE
- **Dependencies:** REPO-001
- **Objective:** Establish a clean Git baseline before implementation.
- **Scope:**
  - Use `main` as the default branch. The repository currently has no commits on `master`.
  - Extend the existing `.gitignore` with `target/`, `dist/`, and `.claude/settings.local.json`.
  - Remove the temporary "SYNC IMPACT REPORT" comment from `.specify/memory/constitution.md` (or from `docs/constitution.md` after REPO-001).
  - Review files to be included in the initial commit.
  - Create the initial commit on `main`.
- **Files:** `.gitignore`, `docs/constitution.md`, all approved documentation.
- **Acceptance Criteria:**
  - No secrets, credentials, generated files, or unnecessary legacy artifacts are committed.
  - Repository has a clean working tree.
  - Initial commit on `main` contains the approved project documentation.
- **Verification:** `git status` and commit inspection.
- **Security Notes:** `.env`, `*.db`, `*.kdbx`, and `.claude/settings.local.json` must not be committed.

---

## Phase 3 — Application Foundation

### SETUP-001: Initialize Tauri and React Project
- **Status:** DONE
- **Dependencies:** REPO-002
- **Objective:** Create the minimal desktop application foundation.
- **Scope:**
  - Initialize Tauri 2.
  - Set up React, TypeScript, and Vite. Vite uses port 1420, `strictPort`, bound to localhost.
  - Configure the Rust application.
  - Set `withGlobalTauri` to false and bundle only the Windows installer.
  - Set the strict Content Security Policy from `docs/architecture.md` and confirm the Tauri IPC `connect-src` values on Windows.
  - Register the command manifest in `build.rs` and a capability file with no plugin permissions.
  - Confirm the application launches on Windows.
- **Files:** `package.json`, `vite.config.ts`, `index.html`, `src/`, `src-tauri/` (`Cargo.toml`, `build.rs`, `tauri.conf.json`, `capabilities/default.json`).
- **Acceptance Criteria:**
  - Frontend and Rust backend compile.
  - Desktop application launches successfully.
  - CSP blocks external origins.
  - No Tauri plugins are enabled.
  - No unrelated features are introduced.
- **Verification:** Relevant build commands and manual launch test.
- **Security Notes:** No remote scripts, fonts, or assets. Devtools are disabled in release builds.

### SETUP-002: Configure Styling Foundation
- **Status:** TODO
- **Dependencies:** SETUP-001
- **Objective:** Configure Tailwind CSS and the project's design tokens.
- **Scope:**
  - Configure Tailwind CSS 4 with its Vite plugin.
  - Map `docs/design-tokens.md` into an `@theme` block.
  - Bundle the Inter font locally.
  - Establish basic global styles.
- **Files:** `src/styles/index.css`, `vite.config.ts`, `package.json`.
- **Acceptance Criteria:**
  - Tailwind works in the React application.
  - Design tokens are centralized.
  - No unnecessary UI components are added.
- **Verification:** Build and a minimal styling check.
- **Security Notes:** Font is bundled from an npm package, not fetched remotely.

### SETUP-003: Configure Rust Dependencies and Toolchain
- **Status:** TODO
- **Dependencies:** SETUP-001
- **Objective:** Establish the Rust dependency and toolchain foundation.
- **Scope:**
  - Pin the Rust toolchain in `rust-toolchain.toml` (stable, with clippy and rustfmt).
  - Add only the dependencies listed in `docs/dependencies.md`, re-verifying versions at this time.
  - Configure `[lints.clippy]` to deny `unwrap_used`, `expect_used`, `dbg_macro`, `print_stdout`, and `print_stderr` outside tests.
  - Enable `#![forbid(unsafe_code)]` in the app crate.
- **Files:** `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `rust-toolchain.toml`.
- **Acceptance Criteria:**
  - Dependencies compile successfully.
  - Toolchain configuration is reproducible (`cargo --locked`).
  - No unnecessary dependencies are introduced.
- **Verification:** `cargo check`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo audit`.
- **Security Notes:** Each cryptographic dependency is part of the attack surface. Versions are pinned in the lockfile.

### SETUP-004: Configure Frontend Tooling
- **Status:** TODO
- **Dependencies:** SETUP-001
- **Objective:** Establish type checking, linting, formatting, and frontend tests.
- **Scope:**
  - Configure TypeScript 6.0, ESLint with typescript-eslint and react-hooks, Prettier 3.
  - Configure Vitest with jsdom and Testing Library.
  - Add `npm` scripts: `typecheck`, `lint`, `test`.
  - Enforce that only `src/api/` imports `@tauri-apps/api` (lint rule).
- **Files:** `package.json`, `package-lock.json`, `tsconfig.json`, `eslint.config.js`, `.prettierrc`, `vitest.config.ts`.
- **Acceptance Criteria:**
  - `npm run typecheck`, `npm run lint`, and `npm run test` run successfully.
  - The `src/api/` boundary is enforced by a lint rule.
- **Verification:** The three npm scripts, and `npm audit --omit=dev`.
- **Security Notes:** Dev dependencies are not shipped. Run `npm ci` from the lockfile.

---

## Phase 4 — Rust Core Foundation

### CORE-001: Establish Rust Core Module Structure
- **Status:** TODO
- **Dependencies:** SETUP-003
- **Objective:** Create the initial Rust core module structure.
- **Scope:**
  - Create the modules from `docs/architecture.md` §4: `commands/`, `vault/`, `crypto/`, `storage/`, `generator.rs`, `clipboard.rs`, `dto.rs`, `error.rs`.
  - Define shared error handling (`AppError` with `thiserror`, mapped at the boundary).
  - Keep core modules independent of Tauri.
- **Files:** `src-tauri/src/lib.rs`, `main.rs`, `error.rs`, `dto.rs`, `commands/`, `vault/`, `crypto/`, `storage/`, `generator.rs`, `clipboard.rs`.
- **Acceptance Criteria:**
  - Module boundaries follow `docs/architecture.md`.
  - Only `commands/` and `lib.rs` depend on Tauri (verified with `cargo tree` or grep).
  - Project compiles.
- **Verification:** `cargo check` and module-level review.
- **Security Notes:** `AppError` must not carry SQL text, paths, or key material.

### CORE-002: Define IPC DTOs and Validation Boundaries
- **Status:** TODO
- **Dependencies:** CORE-001
- **Objective:** Establish typed data transfer objects and validation boundaries.
- **Scope:**
  - Define request and response structures matching `docs/contracts/ipc-commands.md`.
  - Define `IpcError { code, field?, reason?, retryAfterSecs? }`.
  - Implement validation rules from `docs/data-model.md` in `vault/validation.rs`: folder names 1–100, platform names 1–500, URL and identifier 0–500, passwords 1–1,000, master password 12–1,024.
  - Redact `Debug` and `Display` for secret-bearing types and for `IpcError`.
- **Files:** `src-tauri/src/dto.rs`, `src-tauri/src/error.rs`, `src-tauri/src/vault/validation.rs`.
- **Acceptance Criteria:**
  - DTOs match the approved IPC contract.
  - Invalid input is rejected at the Rust boundary with the correct `field` and `reason`.
  - Sensitive values are not exposed in `Debug` output or error payloads.
- **Verification:** Focused unit tests (validation table, redaction).
- **Security Notes:** Validation is authoritative in Rust; frontend checks are cosmetic only.

---

## Phase 5 — Cryptography and Storage

### CRYPTO-001: Argon2id Key Derivation
- **Status:** TODO
- **Dependencies:** CORE-001
- **Objective:** Derive the key-encryption key from the master password.
- **Scope:**
  - Implement `crypto/kdf.rs`: NFKC normalization, Argon2id (64 MiB, 3 iterations, parallelism 1, 32-byte output).
  - Bounds-check KDF parameters read from disk (memory 19,456–1,048,576 KiB; iterations 1–10; parallelism 1–8).
- **Files:** `src-tauri/src/crypto/kdf.rs`, `src-tauri/src/crypto/mod.rs`.
- **Acceptance Criteria:**
  - Same inputs derive the same key; different salt or password derives a different key.
  - NFKC-equivalent inputs derive the same key.
  - Production parameters match the constants; out-of-bounds parameters are rejected.
- **Verification:** `cargo test` for the KDF module (reduced-cost parameters where needed).
- **Security Notes:** Principle VII applies. Use only the RustCrypto `argon2` crate. Zeroize inputs and outputs.

### CRYPTO-002: XChaCha20-Poly1305 Authenticated Encryption
- **Status:** TODO
- **Dependencies:** CORE-001
- **Objective:** Provide seal and open operations with associated data.
- **Scope:**
  - Implement `crypto/aead.rs`: seal and open with a 24-byte random nonce from the OS CSPRNG.
- **Files:** `src-tauri/src/crypto/aead.rs`.
- **Acceptance Criteria:**
  - Round-trip works.
  - Modifying ciphertext, nonce, AAD, or key makes open fail without partial plaintext.
  - Two seals of the same plaintext produce different output.
- **Verification:** `cargo test` for the AEAD module.
- **Security Notes:** Use only the RustCrypto `chacha20poly1305` crate. No custom construction.

### CRYPTO-003: Vault Key Wrapping and Record AAD
- **Status:** TODO
- **Dependencies:** CRYPTO-001, CRYPTO-002
- **Objective:** Generate, wrap, and unwrap the vault key, and build record AAD.
- **Scope:**
  - Implement `crypto/keys.rs`: random 32-byte vault key; wrap and unwrap with `wrap_aad` (format version, KDF parameters, salt).
  - Implement folder and entry record AAD exactly as in `docs/contracts/vault-file-format.md` (including `folder_id`, `created_at`, `updated_at`).
- **Files:** `src-tauri/src/crypto/keys.rs`.
- **Acceptance Criteria:**
  - Wrap and unwrap round-trip; a wrong password fails authentication.
  - Changing any header parameter, salt, or version fails unwrap.
  - Changing `folder_id`, `created_at`, or `updated_at` in record AAD fails open.
- **Verification:** `cargo test` for keys and AAD.
- **Security Notes:** Authentication failure is the only password check. No verifier is stored.

### STORAGE-001: SQLite Connection and Schema Migrations
- **Status:** TODO
- **Dependencies:** CORE-001, SETUP-003
- **Objective:** Create the local database with the version 1 schema.
- **Scope:**
  - Implement `storage/mod.rs` (connection) with pragmas `foreign_keys=ON`, `secure_delete=ON`, `synchronous=FULL`.
  - Implement `storage/migrations.rs` with schema version 1 from `docs/data-model.md` and `PRAGMA user_version`.
- **Files:** `src-tauri/src/storage/mod.rs`, `src-tauri/src/storage/migrations.rs`.
- **Acceptance Criteria:**
  - A new database is created at version 1 with the expected tables and indexes.
  - Foreign keys are enforced; deleting a folder cascades to its entries.
  - Opening a database with a higher `user_version` is refused without modification.
- **Verification:** `cargo test` for migrations and pragmas (in-memory and temporary files).
- **Security Notes:** Parameterized statements only. No plaintext user content in the schema.

### STORAGE-002: Vault Repository
- **Status:** TODO
- **Dependencies:** STORAGE-001
- **Objective:** Store and read opaque encrypted records through a repository.
- **Scope:**
  - Implement `storage/repo.rs` (`VaultRepository` trait and SQLite implementation): meta, folders, entries as `(nonce, ciphertext)` blobs.
  - Use transactions for multi-row operations (vault create, folder delete).
  - Provide `folder_entry_counts()` without decryption.
- **Files:** `src-tauri/src/storage/repo.rs`.
- **Acceptance Criteria:**
  - CRUD for folders and entries works against an in-memory database.
  - Repository returns and accepts only opaque blobs.
  - Multi-row operations are atomic.
- **Verification:** `cargo test` for the repository.
- **Security Notes:** Storage never sees plaintext or keys.

---

## Future Phases

The following areas will be broken into smaller tasks when their dependencies are ready:

1. Vault service: create, unlock, lock, folder and entry operations (`vault/service.rs`), using CRYPTO and STORAGE tasks.
2. Session, unlock throttling, and auto-lock (`vault/session.rs`, `vault/throttle.rs`, `vault/autolock.rs`).
3. Password generator (`generator.rs`) and clipboard port with delayed clear (`clipboard.rs`).
4. Tauri IPC command implementation and app-level state (`commands/`, `lib.rs`).
5. React frontend: API layer, session provider, unlock and create screens, folder and entry screens.
6. Integration: full lifecycle tests, no-plaintext-at-rest test, offline checks, security review, and release preparation.

---

## Task Template

Use this template when adding a new task.

### TASK-ID: Task Title

- **Status:** TODO
- **Dependencies:** None
- **Objective:** What should this task accomplish?
- **Scope:**
  - What is included?
  - What is explicitly excluded?
- **Files:** Expected files to create or modify.
- **Acceptance Criteria:**
  - [ ] Criterion 1
  - [ ] Criterion 2
- **Verification:** How will the result be checked?
- **Security Notes:** Relevant security considerations, or None.

---

## Backlog Rules

- This file is the single source of truth for task status.
- Only approved tasks may be implemented.
- Keep tasks small and independently verifiable.
- Do not combine unrelated changes into one task.
- Add dependencies explicitly.
- Every task lists `Files` and `Security Notes` (use "None" when not applicable).
- Update task status after implementation and review.
- Do not mark a task `DONE` before user approval.
- Add new tasks when implementation reveals necessary work.
