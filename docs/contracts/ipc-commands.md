# Contract: Tauri IPC Commands and Events

This is the complete interface between the React/TypeScript layer and the Rust core. Anything not
listed here MUST NOT be exposed. Field names below are the JSON names crossing the boundary
(camelCase). Commands are `async`; "locked" means no unlocked session exists.

## Types

```text
VaultStatus     = "uninitialized" | "locked" | "unlocked"
FolderSummary   = { id: string, name: string, entryCount: number }
EntrySummary    = { id: string, folderId: string, platformName: string,
                    platformUrl: string, identifier: string,
                    createdAt: number, updatedAt: number }          // never includes password
NewEntry        = { folderId: string, platformName: string, platformUrl?: string,
                    identifier?: string, password: string }
EntryUpdate     = { platformName: string, platformUrl: string, identifier: string,
                    password?: string }                               // omit password = unchanged
IpcError        = { code: ErrorCode, field?: string, reason?: string,
                    retryAfterSecs?: number }
ValidationReason = "empty" | "missing" | "too_short" | "too_long" | "duplicate"
```

Timestamps are unix seconds. Strings use UTF-8. Absent optional strings are treated as empty.

`field` and `reason` are set together with `VALIDATION`. `reason` is one of `ValidationReason`.
`field` names the input that failed; `reason` says why.

## Commands

### Vault

| Command | Arguments | Returns | Errors | Notes |
|---------|-----------|---------|--------|-------|
| `vault_status` | none | `VaultStatus` | `VAULT_CORRUPT`, `STORAGE_ERROR` | Called at startup and after errors |
| `vault_create` | `masterPassword: string` | `void` (vault becomes unlocked) | `VALIDATION` (`field` = `masterPassword`; `reason` = `too_short` \| `too_long`), `ALREADY_INITIALIZED`, `STORAGE_ERROR` | 12-1,024 characters; UI performs the confirmation-match check |
| `vault_unlock` | `masterPassword: string` | `void` | `INVALID_PASSWORD`, `UNLOCK_THROTTLED` (+`retryAfterSecs`), `NOT_INITIALIZED`, `VAULT_CORRUPT` | The failure that starts a delay (5th and later failures) returns `INVALID_PASSWORD` with `retryAfterSecs`. An attempt made during an active delay returns `UNLOCK_THROTTLED` with `retryAfterSecs` and does not run the KDF. |
| `vault_lock` | none | `void` | none | Idempotent |
| `vault_report_activity` | none | `void` | `VAULT_LOCKED` | Resets the inactivity timer; UI calls at most once every 5 seconds |

### Folders (all require unlocked)

| Command | Arguments | Returns | Errors |
|---------|-----------|---------|--------|
| `folder_list` | none | `FolderSummary[]` sorted by name | `VAULT_LOCKED` |
| `folder_create` | `name: string` | `FolderSummary` | `VAULT_LOCKED`, `VALIDATION` (`field` = `name`; `reason` = `empty` \| `too_long` \| `duplicate`) |
| `folder_rename` | `id: string, name: string` | `FolderSummary` | `VAULT_LOCKED`, `NOT_FOUND`, `VALIDATION` (`field` = `name`; `reason` as for `folder_create`) |
| `folder_delete` | `id: string` | `void` (cascades to entries) | `VAULT_LOCKED`, `NOT_FOUND` |

### Entries (all require unlocked)

| Command | Arguments | Returns | Errors |
|---------|-----------|---------|--------|
| `entry_list` | `folderId: string` | `EntrySummary[]` sorted by platform name, no passwords | `VAULT_LOCKED`, `NOT_FOUND` |
| `entry_create` | `entry: NewEntry` | `EntrySummary` | `VAULT_LOCKED`, `NOT_FOUND` (folder), `VALIDATION` (`field` = `platformName` \| `password` \| `platformUrl` \| `identifier`; `reason` = `missing` \| `empty` \| `too_long`) |
| `entry_update` | `id: string, update: EntryUpdate` | `EntrySummary` | `VAULT_LOCKED`, `NOT_FOUND`, `VALIDATION` (same `field` and `reason` values as `entry_create`) |
| `entry_delete` | `id: string` | `void` | `VAULT_LOCKED`, `NOT_FOUND` |
| `entry_reveal_password` | `id: string` | `string` (the password) | `VAULT_LOCKED`, `NOT_FOUND` |
| `entry_copy_password` | `id: string` | `void` (password goes to clipboard inside Rust) | `VAULT_LOCKED`, `NOT_FOUND`, `CLIPBOARD_ERROR` |

### Generator

| Command | Arguments | Returns | Errors |
|---------|-----------|---------|--------|
| `password_generate` | none | `string` (20 characters; lower, upper, digit, symbol) | `VAULT_LOCKED` |

`password_generate` requires an unlocked vault so a locked app exposes no functionality. It is
stateless; nothing is stored until the user saves an entry.

## Events (Rust → UI)

| Event | Payload | When |
|-------|---------|------|
| `vault://locked` | `{ reason: "auto" }` | The inactivity monitor locked the vault. Manual lock does not emit; the UI already knows. |

On receipt the UI discards all unlocked state and shows the unlock screen.

## Error codes

| Code | Meaning | UI behavior |
|------|---------|-------------|
| `VAULT_LOCKED` | Operation needs an unlocked vault | Go to unlock screen |
| `NOT_INITIALIZED` / `ALREADY_INITIALIZED` | Vault state mismatch | Re-check `vault_status` |
| `INVALID_PASSWORD` | Wrong master password; includes `retryAfterSecs` when this failure starts a delay | Show error; keep field cleared; if `retryAfterSecs` is set, disable submit and show countdown |
| `UNLOCK_THROTTLED` | Attempt made during an active delay; includes `retryAfterSecs` | Disable submit and show countdown |
| `VALIDATION` | Input rule violated; `field` and `reason` name it | Show field message for the reason |
| `NOT_FOUND` | Unknown folder or entry id | Refresh lists |
| `CLIPBOARD_ERROR` | Clipboard unavailable | Show error; password not copied |
| `VAULT_CORRUPT` | Header missing, tampered, or unreadable | Fatal error screen; no repair attempt |
| `STORAGE_ERROR` | Database I/O failure | Generic error message |
| `INTERNAL` | Unexpected failure | Generic error message |

Error payloads never contain secrets, SQL text, file paths, or stack traces.

## Permission model

- Every command above is declared in the `build.rs` app manifest, producing one `allow-<command>`
  permission each.
- `capabilities/default.json` applies to the single `main` window and lists exactly those
  permissions plus the minimal core permission to listen to events.
- No Tauri plugin permissions (filesystem, shell, opener, clipboard, updater, http) are granted.

## Secret exposure rules

| Value | Crosses the boundary | Direction |
|-------|----------------------|-----------|
| Master password | `vault_create`, `vault_unlock` | UI → Rust, on submit only |
| Entry password (new or changed) | `entry_create`, `entry_update` | UI → Rust |
| Entry password (existing) | `entry_reveal_password` only, on explicit user action | Rust → UI |
| Generated password | `password_generate` | Rust → UI |
| Vault key, derived keys | Never | Stay in Rust |
| Passwords in lists | Never | Not included in `EntrySummary` |
