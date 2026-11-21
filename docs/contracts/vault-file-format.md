# Contract: Vault Storage and Cryptographic Format (version 1)

Defines what is written to `vault.db` and how it is protected. Changes to anything here require a
new `format_version` and a migration plan. Only standard primitives from maintained libraries are
used; nothing here is a custom cryptographic construction (Constitution Principle III).

Related: [../security.md](../security.md) (decisions and accepted risks). The SQLite schema is in
`specs/001-password-vault-mvp/data-model.md`.

## Location

`vault.db` inside the operating system's per-user application data directory for the
application identifier. The path is resolved in Rust; the UI never sees or supplies it.

## Primitives and parameters

| Item | Value |
|------|-------|
| Password KDF | Argon2id (version 0x13), output 32 bytes |
| KDF memory / iterations / parallelism | 65,536 KiB / 3 / 1 |
| KDF salt | 16 random bytes (OS CSPRNG), unique per vault |
| AEAD | XChaCha20-Poly1305 (32-byte key, 24-byte nonce, 16-byte tag) |
| Nonces | 24 random bytes per encryption (OS CSPRNG) |
| Vault key | 32 random bytes (OS CSPRNG), generated at creation |
| Password preprocessing | Unicode NFKC normalization, then UTF-8 bytes |
| Allowed KDF bounds on read | memory 19,456-1,048,576 KiB; iterations 1-10; parallelism 1-8; otherwise `VAULT_CORRUPT` |

## Key hierarchy

```text
password  --NFKC-->  bytes  --Argon2id(salt, params)-->  KEK (32 bytes, memory only)
vault key (random, memory only while unlocked)
wrapped_key = XChaCha20-Poly1305.seal(KEK, wrapped_key_nonce, vault key, AAD = wrap_aad)
record      = XChaCha20-Poly1305.seal(vault key, nonce, payload JSON, AAD = record_aad)
```

Only `wrapped_key` and its nonce are stored. The master password, the KEK, and the vault key are
never written to disk, logs, or webview storage.

## Associated data

```text
wrap_aad            = "vaulten/v1/wrap" || format_version (u32 BE) || kdf_algorithm
                      || memory_kib (u32 BE) || iterations (u32 BE) || parallelism (u32 BE) || salt

record_aad (folder) = "vaulten/v1/folder" || id (16 bytes)
                      || created_at (i64 BE) || updated_at (i64 BE)

record_aad (entry)  = "vaulten/v1/entry"  || id (16 bytes) || folder_id (16 bytes)
                      || created_at (i64 BE) || updated_at (i64 BE)
```

`id` and `folder_id` are the UUIDs' 16 bytes each. `created_at` and `updated_at` are the unix-second
values stored in the same row, encoded as signed 64-bit big-endian integers. Binding the header
parameters into `wrap_aad` makes any change to the stored KDF parameters, version, or salt fail
authentication. Binding the label, id, folder id, and timestamps into `record_aad` prevents a
ciphertext from being accepted as a different record type or row, moved to another folder, or
returned with altered timestamps.

An update re-seals the record with a fresh nonce and the new `updated_at` value.

Replacing a whole row with an older valid row (ciphertext, nonce, and timestamps together) is not
detected. Row deletion is not detected either. See [../security.md](../security.md).

## Stored layout

| Stored value | Plaintext on disk? |
|--------------|--------------------|
| KDF parameters, salt, format version | Yes (not secret) |
| Wrapped vault key and nonce | Ciphertext |
| Folder name | Ciphertext |
| Entry name, URL, identifier, password | Ciphertext |
| Record ids | Yes (metadata) |
| Entry-to-folder relation, created/updated timestamps | Yes (metadata), authenticated through `record_aad` |
| Ciphertext lengths and row counts | Observable |

## Operations

1. **Create**: validate password; generate salt and vault key; derive KEK; seal vault key; insert
   the header row in one transaction; hold the session in memory.
2. **Unlock**: read header; bounds-check parameters; derive KEK; open `wrapped_key`. Authentication
   failure means a wrong password (or a damaged header) and returns `INVALID_PASSWORD`; the
   throttle counter increments.
3. **Write record**: serialize the payload, generate a fresh nonce, seal with the vault key and the
   row's `record_aad`, write `(nonce, ciphertext)`. Updating a record always uses a new nonce.
4. **Read record**: open with the vault key and the record AAD built from the row's stored values;
   on authentication failure return an error for that record without exposing partial plaintext.
5. **Lock**: drop the session, zeroizing the vault key. No key material is cached elsewhere.

## Payloads

```text
FolderPayload { v: 1, name }
EntryPayload  { v: 1, platform_name, platform_url, identifier, password }
```

UTF-8 JSON. Decrypted buffers are zeroized after deserialization.

## Evolution

- Raising KDF cost: new vaults use new parameters; existing vaults keep their stored parameters
  until a future password-change or re-wrap feature rewrites the header.
- A reader that sees a higher `format_version` than it supports MUST refuse to open the vault and
  MUST NOT modify the file.

## Test obligations

The test suite verifies: AEAD round-trip and tamper detection (ciphertext, nonce, AAD, key),
tampering with folder assignment or timestamps (authentication fails cleanly), header-parameter
tampering rejection, out-of-bounds parameter rejection, nonce uniqueness, and that the raw database
file contains none of the sentinel plaintext values used in the "no plaintext at rest" test.
