# Feature Specification: Password Vault MVP

**Feature Branch**: `001-password-vault-mvp`

**Created**: 2026-11-01

**Status**: Draft

**Input**: User description: "The MVP should include: a vault unlock screen with master password; a vault lock action; a main interface with a folder area, a platform content area, and a global search field at the top (UI location defined, search behavior may be deferred); platform entries with name (required), URL, optional identifier, and password (required); add, edit, and delete entries; masked passwords with reveal and hide; copy password to clipboard; generate or regenerate passwords when adding or editing; fully offline operation."

## Clarifications

### Session 2026-11-01

- Q: Should the vault lock automatically after a period of inactivity, and if so, after how long? → A: Lock after 5 minutes of inactivity
- Q: What minimum length should the master password have? → A: 12 characters minimum, no character-class rules
- Q: After repeated wrong master password attempts, should the app slow down further attempts? → A: After 5 failed attempts, each further attempt waits progressively longer (no permanent lockout)
- Q: Which language should the application's user interface use in the MVP? → A: English only

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Unlock and Lock the Vault (Priority: P1)

A user opens the application and must enter their master password before any stored platform
entry is visible. When finished, the user locks the vault so that its contents are no longer
visible until the master password is entered again.

**Why this priority**: Every other capability depends on access control. Without a secure unlock
and lock, no stored credential can be trusted or used.

**Independent Test**: Launch the application with an existing vault, verify that only the unlock
screen is shown, enter the correct master password, confirm the main interface appears, then
trigger the lock action and confirm the main interface is no longer visible.

**Acceptance Scenarios**:

1. **Given** a vault exists and the application is launched, **When** the application opens,
   **Then** only the unlock screen is shown and no folder, platform, or password data is visible.
2. **Given** the unlock screen is shown, **When** the user enters the correct master password,
   **Then** the main interface is shown.
3. **Given** the unlock screen is shown, **When** the user enters an incorrect master password,
   **Then** the vault remains locked, an error message is shown, and the entered password is not
   displayed in plain text.
4. **Given** the vault is unlocked, **When** the user selects the lock action, **Then** the vault
   becomes locked and the unlock screen is shown again.
5. **Given** the vault is unlocked, **When** the application is closed and reopened, **Then** the
   vault starts in the locked state.

---

### User Story 2 - Manage Folders (Priority: P2)

A user organizes platform entries into folders. The user can create a folder, rename it, select it
to view its entries, and delete it.

**Why this priority**: Folders provide the organizational structure the content area depends on,
and they are required before users can meaningfully store many entries.

**Independent Test**: Unlock the vault, create a folder, rename it, select it, and delete it,
confirming each change is reflected in the folder area.

**Acceptance Scenarios**:

1. **Given** the vault is unlocked, **When** the user creates a folder with a non-empty name,
   **Then** the folder appears in the folder area.
2. **Given** a folder exists, **When** the user renames it to a new non-empty name, **Then** the
   folder shows the new name.
3. **Given** multiple folders exist, **When** the user selects one, **Then** that folder is marked
   as selected and the content area shows only platform entries in that folder.
4. **Given** a folder contains platform entries, **When** the user deletes the folder and confirms,
   **Then** the folder and its platform entries are removed.
5. **Given** the user attempts to create or rename a folder with an empty name, **When** they
   confirm, **Then** the change is rejected with a message and no folder is created or renamed.

---

### User Story 3 - Manage Platform Entries (Priority: P3)

A user adds, views, edits, and deletes platform entries within a selected folder. Each entry holds
a platform name, an optional URL, an optional identifier, and a password.

**Why this priority**: This is the core value of the product: storing and retrieving account
credentials. It depends on folders (User Story 2) and an unlocked vault (User Story 1).

**Independent Test**: Unlock the vault, select a folder, add an entry with all fields, verify it
appears in the content area, edit one of its fields, then delete it and confirm it is gone.

**Acceptance Scenarios**:

1. **Given** a folder is selected, **When** the user adds an entry with a platform name and a
   password, **Then** the entry appears in the content area of that folder.
2. **Given** a folder is selected, **When** the user tries to save an entry without a platform
   name or without a password, **Then** the save is rejected and the missing field is indicated.
3. **Given** an entry exists, **When** the user edits its platform name, URL, identifier, or
   password and saves, **Then** the updated values are shown.
4. **Given** an entry exists, **When** the user deletes it and confirms, **Then** the entry is
   removed from the folder.
5. **Given** a platform entry has an optional URL or identifier left empty, **When** it is saved,
   **Then** the entry is saved successfully and the empty fields are shown as empty.
6. **Given** entries exist in a folder, **When** the user views the content area, **Then** each
   entry shows its platform name and identifier, and its password is masked.

---

### User Story 4 - Reveal, Hide, and Copy Passwords (Priority: P4)

A user can reveal a masked password when needed, hide it again, and copy it to the clipboard
without displaying it.

**Why this priority**: Masking protects against shoulder-surfing by default, while reveal and copy
keep the password usable. These depend on stored entries (User Story 3).

**Independent Test**: Open an entry, reveal its password, hide it, and copy it; paste the clipboard
contents into a text field and confirm the password matches.

**Acceptance Scenarios**:

1. **Given** an entry is displayed, **When** the user has not interacted with its password,
   **Then** the password is masked.
2. **Given** a masked password, **When** the user selects reveal, **Then** the password is shown in
   plain text until the user hides it or the vault is locked.
3. **Given** a revealed password, **When** the user selects hide, **Then** the password is masked
   again.
4. **Given** an entry, **When** the user selects copy password, **Then** the password is placed on
   the system clipboard and a confirmation is shown.
5. **Given** a password was copied, **When** a time limit of 60 seconds passes without a new copy
   action, **Then** the application clears the clipboard if it still contains that password.

---

### User Story 5 - Generate and Regenerate Passwords (Priority: P5)

When adding or editing an entry, a user can ask the application to generate a strong password and
regenerate it as many times as needed before saving.

**Why this priority**: Generation helps users avoid weak or reused passwords, but entries can still
be created manually without it, so it is an enhancement over the core flow.

**Independent Test**: Open the add-entry form, generate a password, regenerate it, and save the
entry; confirm the saved password matches the last generated value.

**Acceptance Scenarios**:

1. **Given** the add or edit form is open, **When** the user selects generate, **Then** a new
   password is filled into the password field.
2. **Given** a generated password is present, **When** the user selects regenerate, **Then** the
   password field is replaced with a different generated password.
3. **Given** a generated password, **When** the user saves the entry, **Then** the saved password
   is the generated value.
4. **Given** a generated password, **When** the user has not saved, **Then** the generated password
   is not stored.

---

### User Story 6 - Global Search Location (Priority: P6)

A user sees a global search field at the top of the main interface. The field's position and
purpose are defined in this MVP. Search results behavior MAY be delivered later.

**Why this priority**: Defining the location now keeps layout stable; functional search is
explicitly deferred.

**Independent Test**: Unlock the vault and confirm a search field is visible at the top of the main
interface, spanning all folders in purpose.

**Acceptance Scenarios**:

1. **Given** the main interface is shown, **When** the user looks at the top of the screen,
   **Then** a global search field is visible.
2. **Given** the search field is visible in this MVP, **When** the user types a query, **Then** the
   interface MAY show no results or a "search coming soon" notice without error.

---

### Edge Cases

- What happens when the user enters the wrong master password repeatedly? The vault remains locked
  and each failed attempt is reported; no data is revealed.
- What happens when the user deletes the currently selected folder? The selection clears and the
  content area shows no entries until another folder is selected.
- What happens when a folder is deleted while a platform entry in it is revealed? The entry is
  removed and its password is no longer displayed.
- What happens when the vault is locked while a password is revealed? The password is hidden and
  the content area is cleared.
- What happens when the clipboard is changed by another application before the 60-second clear?
  The application MUST NOT overwrite or clear clipboard content it did not place there.
- What happens when a platform URL is entered without a scheme (e.g., `example.com`)? The entry is
  saved as entered; no network request is made.
- What happens when a folder name or platform name is very long? Names are truncated visually but
  stored in full, up to a maximum of 100 characters for folder names and 500 characters for platform names (FR-012, FR-021).
- What happens when the user tries to create a folder with a name identical to an existing folder?
  The creation is rejected with a message asking for a unique name.

## Requirements *(mandatory)*

### Functional Requirements

**Vault access**

- **FR-001**: System MUST present an unlock screen as the first screen whenever the vault is locked.
- **FR-002**: System MUST require the master password to unlock the vault.
- **FR-003**: System MUST reject an incorrect master password and keep the vault locked.
- **FR-003a**: After 5 consecutive failed unlock attempts, System MUST delay each further unlock
  attempt by a progressively longer wait, and MUST NOT permanently lock or erase the vault. The
  delay resets after a successful unlock.
- **FR-004**: System MUST provide a lock action that immediately locks the vault and returns the
  user to the unlock screen.
- **FR-005**: System MUST start in the locked state on every launch.
- **FR-005a**: System MUST automatically lock the vault after 5 minutes without user activity
  (keyboard or pointer input within the application), returning the user to the unlock screen.
- **FR-006**: System MUST allow the user to create a vault by choosing a master password on first
  launch when no vault exists (see Assumptions). The master password MUST be at least 12
  characters long; no specific character classes are required.
- **FR-007**: System MUST mask the master password input on the unlock screen.

**Folders**

- **FR-008**: Users MUST be able to create a folder with a non-empty name.
- **FR-009**: Users MUST be able to rename a folder to a non-empty name.
- **FR-010**: Users MUST be able to select a folder, which then determines the entries shown in the
  content area.
- **FR-011**: Users MUST be able to delete a folder after confirming the action; deleting a folder
  also deletes the platform entries it contains.
- **FR-012**: System MUST reject folder names that are empty, exceed 100 characters, or duplicate
  an existing folder name.

**Platform entries**

- **FR-013**: Users MUST be able to add a platform entry to the selected folder.
- **FR-014**: Each platform entry MUST have a platform name, which is required.
- **FR-015**: Each platform entry MUST have a password, which is required.
- **FR-016**: Each platform entry MAY have a platform URL.
- **FR-017**: Each platform entry MAY have an identifier, which is free text that can hold a
  username, email address, phone number, or other account identifier.
- **FR-018**: Users MUST be able to edit any field of an existing platform entry.
- **FR-019**: Users MUST be able to delete a platform entry after confirming the action.
- **FR-020**: System MUST reject a save when the platform name or password is missing and indicate
  which field is missing.
- **FR-021**: System MUST limit platform name, URL, and identifier to 500 characters each, and
  passwords to 1,000 characters.

**Password visibility and clipboard**

- **FR-022**: System MUST display every platform password masked by default.
- **FR-023**: Users MUST be able to reveal a masked password and hide a revealed password.
- **FR-024**: System MUST return any revealed password to masked state when the vault is locked or
  the user changes the selected folder.
- **FR-025**: Users MUST be able to copy a platform password to the system clipboard without first
  revealing it.
- **FR-026**: System MUST clear the clipboard 60 seconds after a password copy, only if the
  clipboard still contains the copied value.

**Password generation**

- **FR-027**: While adding or editing an entry, users MUST be able to generate a password.
- **FR-028**: While adding or editing an entry, users MUST be able to regenerate the password,
  producing a different value from the previous one.
- **FR-029**: Generated passwords MUST be at least 16 characters long and include uppercase
  letters, lowercase letters, digits, and symbols.
- **FR-030**: A generated password MUST NOT be stored unless the user saves the entry.

**Global search**

- **FR-031**: The main interface MUST display a global search field at the top of the screen, above
  the folder area and content area.
- **FR-032**: Search results behavior is OUT OF SCOPE for this MVP. The field MAY be inert or show
  a placeholder notice.

**Offline and privacy**

- **FR-033**: System MUST operate fully without a network connection, including unlock, folder
  management, entry management, password generation, and clipboard use.
- **FR-034**: System MUST NOT require an account, server, or cloud service.
- **FR-035**: System MUST NOT transmit user data, including vault contents, master passwords, and
  platform details, to any external destination.

### Key Entities *(include if feature involves data)*

- **Vault**: The single protected container for all user data. Has a locked or unlocked state and
  is protected by a master password.
- **Folder**: A named grouping of platform entries. Has a unique name and belongs to the vault.
- **Platform Entry**: A stored account record. Attributes: platform name (required), platform URL
  (optional), identifier (optional), password (required). Belongs to exactly one folder.
- **Master Password**: The secret the user supplies to unlock the vault. Not itself displayed or
  stored as readable text.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can unlock the vault with the correct master password in under 10 seconds from
  the unlock screen appearing.
- **SC-002**: A user can add a complete platform entry (name, URL, identifier, password) in under
  60 seconds.
- **SC-003**: 95% of users can locate and copy a stored password to the clipboard within 15 seconds
  on first attempt.
- **SC-004**: In 100% of tested states while the vault is locked, no folder names, platform names,
  identifiers, URLs, or passwords are visible on screen.
- **SC-005**: In 100% of tested states while passwords are not revealed, every password is shown
  only as masked characters.
- **SC-006**: The complete MVP workflow (unlock, create folder, add entry, reveal, copy, edit,
  delete, lock) can be completed with the network disabled, with no degraded behavior.
- **SC-007**: After a password is copied, the clipboard no longer contains it within 60 seconds in
  100% of tests where the clipboard was not changed by the user.

## Assumptions

- The MVP targets a single user on a single desktop device with a single vault.
- No vault exists on first launch in the MVP; first-run creation with a chosen master password is
  included so that the unlock flow can be tested (FR-006).
- Master password recovery is out of scope; a forgotten master password means the vault cannot be
  opened. This is documented for the user.
- Folders are a single level; nested folders are out of scope.
- Each platform entry belongs to exactly one folder; moving entries between folders is out of scope
  unless added later.
- Clipboard clearing is best-effort and depends on the operating system's clipboard behavior.
- Import, export, sync, browser autofill, sharing, and cloud backup are out of scope, consistent with
  the constitution's MVP scope discipline.
- The user interface is English only in the MVP; localization is out of scope.
- Search results and search filtering are deferred; only the search field's location is defined.
- Standard password-generation character sets are acceptable for the generator.
