# Password Manager Constitution

## Core Principles

### I. Security First (NON-NEGOTIABLE)

Security outranks convenience, performance, and feature requests. When a trade-off
is unavoidable, the option that protects the user's secrets MUST be chosen, and the
trade-off MUST be documented in the relevant spec or in `docs/security.md`. Security-relevant
defects are release blockers.

### II. No Plaintext Secrets at Rest

The master password, plaintext passwords, and encryption keys MUST NOT be persisted
in plaintext anywhere: not in SQLite, files, logs, crash reports, clipboard
history managed by the app, or local storage of the web view. Sensitive values
SHOULD be held in memory for the shortest practical time and cleared (zeroized)
when no longer needed. Logs MUST NOT contain secrets.

### III. Vetted Cryptography Only

Custom cryptographic algorithms, modes, or protocols MUST NOT be written. Only
well-maintained, reputable libraries and standard primitives MUST be used (for
example, a standard AEAD cipher for encryption and a memory-hard KDF such as
Argon2id for master-password derivation). Parameters MUST follow current
published guidance, and any change to them MUST be treated as a security change
that requires tests.

### IV. Local-Only and Offline Operation

The application MUST run entirely on the user's machine. It MUST NOT require a
server, cloud service, user account, or network connection to function. It MUST
NOT send telemetry, analytics, crash reports, or update checks containing user
data. Any feature that introduces network access requires a constitution
amendment and explicit user approval.

### V. Separated Presentation and Native Core

The React/TypeScript layer is responsible for presentation only. The Rust layer
is responsible for native and core operations, including cryptography, key
management, vault persistence, and clipboard and file access. Business rules
MUST NOT live in UI components, and UI state MUST NOT be the source of truth for
security decisions. Each module SHOULD have a single clear purpose and remain
independently testable.

### VI. Minimal Tauri IPC Surface

Tauri commands and events MUST expose only the native capabilities the UI
strictly needs. Each command MUST validate its inputs in Rust and MUST NOT return
secrets unless the UI requires them for an explicit user action. Broad or generic
command surfaces (such as arbitrary file or shell access) MUST NOT be exposed.
Tauri capabilities and permissions MUST be scoped to the minimum required.

### VII. Test Security-Critical Paths

Cryptographic operations, key derivation, encryption and decryption round-trips,
vault locking and unlocking, master-password verification, IPC input validation,
and secret zeroization or redaction MUST have automated tests. Tests for these
paths MUST be written before or alongside the implementation, and a security
test failure MUST block merge.

### VIII. MVP Scope Discipline

Only features explicitly defined in the current MVP spec MAY be implemented.
Features outside that scope (for example, sync, browser autofill, sharing, or
cloud backup) MUST NOT be added unless they are explicitly defined and approved
through the spec process. Prefer the simplest design that satisfies the spec
(YAGNI).

## Technology and Security Constraints

- Desktop shell: Tauri. Native and core logic: Rust. Presentation: React with
  TypeScript.
- Persistent local storage: SQLite. Only encrypted secret material MAY be stored;
  schema fields containing secrets MUST be encrypted before writing.
- Cryptographic dependencies MUST come from established crates or packages.
  Each new cryptographic dependency MUST be justified in the plan.
- The web view MUST NOT load remote scripts, styles, or fonts. Content Security
  Policy MUST restrict sources to the local application.
- Clipboard use for passwords MUST be time-limited where the platform allows it.
- Auto-lock on inactivity is required in MVP scope. Auto-lock on system lock or sleep
  is out of MVP scope and MAY be added later through a spec amendment.

## Development Workflow and Quality Gates

- Every feature MUST start from a written specification and plan, then be broken
  into tasks recorded in `docs/backlog.md`, following `docs/workflow.md`. Spec Kit
  MUST NOT be used.
- Every change touching security-critical paths (Principle VII) MUST include
  tests and MUST be reviewed for compliance with Principles I–III before merge.
- Dependencies, Tauri capabilities, and IPC command changes MUST be reviewed as
  attack-surface changes.
- Code, tests, and documentation MUST be kept consistent; undocumented behavior
  is treated as a defect.

## Governance

This constitution supersedes all other project practices. Where a spec, plan,
or task conflicts with it, the constitution prevails and the conflicting artifact
MUST be corrected.

Amendments require: a written proposal describing the change and its rationale,
a review by the project maintainer, an update to this file with a new version
and dates, and a review of dependent templates and artifacts for consistency.

Versioning follows semantic versioning:
- MAJOR: removal or incompatible redefinition of a principle or governance rule.
- MINOR: new principle or section, or materially expanded guidance.
- PATCH: clarifications, wording, or typo fixes that do not change meaning.

Compliance review: every plan MUST include a constitution check against
Principles I–VIII. Every pull request MUST confirm it does not violate
Principles II, III, IV, and VI. Unjustified complexity MUST be rejected in review.

**Version**: 2.0.0 | **Ratified**: 2026-11-01 | **Last Amended**: 2026-11-22
