# Data Model: Password Vault MVP

Entities come from [requirements.md](requirements.md) (Key Entities, FR-008 to FR-021). Persisted entities have an
encrypted form (stored) and a plaintext form (in memory only while the vault is unlocked). On-disk
byte layout is in [contracts/vault-file-format.md](contracts/vault-file-format.md).

## Entities

### Vault

The single protected container. Exactly one per installation.

| Field | Type | Notes |
|-------|------|-------|
| format_version | integer | 1 in the MVP |
| kdf parameters | algorithm, memory, iterations, parallelism, salt | Argon2id; stored in header |
| wrapped_key | nonce + ciphertext | Vault key encrypted under the password-derived key |
| created_at | unix seconds | Plaintext |

**States** (derived, not stored): `uninitialized` (no header) → `locked` → `unlocked`.

Transitions:

| From | Event | To |
|------|-------|----|
| uninitialized | `vault_create` succeeds | unlocked |
| locked | `vault_unlock` with correct password | unlocked |
| locked | `vault_unlock` with wrong password | locked (failure counter +1) |
| unlocked | `vault_lock` | locked |
| unlocked | 5 minutes without activity | locked (event `vault://locked`) |
| any | app exits | locked on next launch |

### Session (memory only)

| Field | Type | Notes |
|-------|------|-------|
| vault_key | 32 bytes | Zeroized on drop; exists only while unlocked |
| last_activity_monotonic | Instant | Auto-lock clock 1 |
| last_activity_wall | SystemTime | Auto-lock clock 2 |

### Throttle state (memory only)

| Field | Type | Notes |
|-------|------|-------|
| consecutive_failures | integer | Reset on successful unlock or app restart |
| blocked_until | instant | None until the 5th failure; delay = `min(2^(n-5), 300)` seconds |

### Folder

| Field | Type | Rules |
|-------|------|-------|
| id | UUID v4 | Plaintext, immutable |
| name | string | Encrypted. Trimmed; 1-100 characters; unique case-insensitively among folders (FR-012) |
| created_at, updated_at | unix seconds | Plaintext |

Relationship: one folder has many entries. Deleting a folder deletes its entries (FR-011).
Renaming to the same name with only case changes is allowed; renaming to another folder's name is
rejected. Uniqueness is enforced in the application layer because names are encrypted; the database cannot enforce it.

### Platform Entry

| Field | Type | Rules |
|-------|------|-------|
| id | UUID v4 | Plaintext, immutable |
| folder_id | UUID | Plaintext, required, references a folder; bound into the entry record AAD, so changing it fails authentication; entries cannot move in the MVP |
| platform_name | string | Encrypted. Required; trimmed; 1-500 characters (FR-014, FR-021) |
| platform_url | string | Encrypted. Optional; 0-500 characters; stored as entered, no format validation or network use |
| identifier | string | Encrypted. Optional; 0-500 characters; free text (username, email, phone, other) |
| password | string | Encrypted. Required; 1-1,000 characters; never returned by list operations |
| created_at, updated_at | unix seconds | Plaintext |

Duplicate platform names are allowed (one platform can have several accounts).
Update semantics: omitting `password` on update keeps the stored password; providing it replaces
it (empty is rejected).

### Master Password

Not an entity in storage. Input only: 12-1,024 Unicode characters (FR-006), NFKC-normalized before
key derivation, never persisted or logged.

## Encrypted record payloads

Serialized as JSON, then sealed (see file-format contract). The `v` field allows payload evolution.

```text
FolderPayload { v: 1, name }
EntryPayload  { v: 1, platform_name, platform_url, identifier, password }
```

## SQLite schema (version 1)

```sql
PRAGMA user_version = 1;

CREATE TABLE vault_meta (
  id                 INTEGER PRIMARY KEY CHECK (id = 1),
  format_version     INTEGER NOT NULL,
  kdf_algorithm      TEXT    NOT NULL,           -- 'argon2id'
  kdf_memory_kib     INTEGER NOT NULL,
  kdf_iterations     INTEGER NOT NULL,
  kdf_parallelism    INTEGER NOT NULL,
  kdf_salt           BLOB    NOT NULL,           -- 16 bytes
  wrapped_key_nonce  BLOB    NOT NULL,           -- 24 bytes
  wrapped_key        BLOB    NOT NULL,           -- 48 bytes (32 + 16 tag)
  created_at         INTEGER NOT NULL
);

CREATE TABLE folders (
  id          TEXT    PRIMARY KEY,               -- UUID
  nonce       BLOB    NOT NULL,                  -- 24 bytes
  ciphertext  BLOB    NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE TABLE entries (
  id          TEXT    PRIMARY KEY,               -- UUID
  folder_id   TEXT    NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
  nonce       BLOB    NOT NULL,
  ciphertext  BLOB    NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE INDEX idx_entries_folder ON entries(folder_id);
```

Notes:

- The `vault_meta` single-row constraint prevents a second header. A database with data tables but
  no header row is reported as `VAULT_CORRUPT`; the app never overwrites it.
- Ordering by name happens after decryption (names are encrypted); entry counts per folder come from
  `SELECT folder_id, COUNT(*) FROM entries GROUP BY folder_id` without decrypting.
- Multi-row operations (create vault, delete folder) run in a single transaction.
- Future schema changes use `PRAGMA user_version` and forward-only migrations.

## Validation summary

| Rule | Source | Enforced in |
|------|--------|-------------|
| Folder name 1-100, trimmed, unique case-insensitive | FR-012 | `vault/validation.rs` (UI mirrors) |
| Entry name required, 1-500 | FR-014, FR-021 | `vault/validation.rs` |
| Entry password required, 1-1,000 | FR-015, FR-021 | `vault/validation.rs` |
| URL, identifier optional, 0-500 | FR-016, FR-017, FR-021 | `vault/validation.rs` |
| Master password 12-1,024 characters | FR-006 | `vault/validation.rs` |
| All data operations require unlocked session | FR-001, FR-004 | `vault/service.rs` |
