# Vaulten — Development Workflow

## 1. Core Approach

Vaulten is developed incrementally using small, independently verifiable tasks.

- The user defines, prioritizes, and approves tasks.
- Claude Code implements approved tasks.
- The user reviews and accepts completed work.
- No large, one-shot implementation.
- No Spec Kit workflows.
- No autonomous feature expansion.

## 2. Task Definition

Every task must include:

- **ID:** Unique task identifier.
- **Objective:** What needs to be achieved.
- **Scope:** What is included and excluded.
- **Dependencies:** Tasks that must be completed first.
- **Files:** Expected files to create or modify, if known.
- **Acceptance Criteria:** Observable conditions for completion.
- **Verification:** Exact tests or commands required.
- **Security Notes:** Relevant security constraints, if applicable.

Tasks must be small enough to implement and verify in one focused session whenever practical.

## 3. Task Execution

For each task:

1. Start Claude Code in the project directory.
2. Provide the approved task and its acceptance criteria.
3. Ask Claude Code to inspect relevant existing code before making changes.
4. Implement only the assigned scope.
5. Run the required verification commands.
6. Review the diff for unintended changes.
7. Produce a concise completion report.
8. Update the task status only after verification.
9. Wait for user review and approval before proceeding.

If requirements are unclear, contradictory, or require architectural changes, stop and ask.

## 4. Task Status

Use these statuses:

- `TODO`
- `IN_PROGRESS`
- `BLOCKED`
- `REVIEW`
- `DONE`

A task becomes `DONE` only after its acceptance criteria pass and the user approves it.

## 5. Parallel Execution

Parallel tasks are allowed only when:

- They have no unresolved dependencies.
- They do not modify overlapping files.
- They do not depend on unfinished architectural decisions.
- Their changes can be integrated and verified independently.

Sequential execution is the default.

Do not use multiple Claude Code sessions or agents unless the user explicitly approves parallel execution.

## 6. Testing and Verification

- Run only tests relevant to the current task.
- Add tests when they verify meaningful behavior or prevent regressions.
- Do not add tests solely to increase test count.
- Security-critical behavior must have automated tests.
- Never report a check as passed unless it was executed.
- Run broader verification at integration milestones.

## 7. Security-Critical Changes

Changes involving cryptography, key management, storage protection, session handling, IPC permissions or secret exposure require additional scrutiny.

- Follow the approved architecture and security decisions.
- Never invent cryptographic designs.
- Add focused automated tests.
- Request an independent security review before accepting high-risk changes.
- Do not weaken security controls to make tests pass.

## 8. Completion Report

Keep reports concise:

- **Result:** What was implemented.
- **Files:** Files changed.
- **Verification:** Commands executed and their results.
- **Acceptance:** Passed or failed criteria.
- **Risks:** Only relevant unresolved issues.

Do not produce lengthy explanations, repeat the task description, or include unnecessary code excerpts.

## 9. Context and Cost Management

- Prefer one focused task per Claude Code session.
- Use `/clear` when starting unrelated work or when context becomes unnecessarily large.
- Do not clear context while an unfinished task still needs it.
- Use Haiku for routine implementation and verification.
- Reserve stronger models for complex architectural decisions and focused security reviews.
- Avoid unnecessary agents, repeated repository-wide analysis and verbose reports.

## 10. Completion Rule

A task is complete only when:

1. Its implementation is within scope.
2. Its acceptance criteria pass.
3. Required verification has been executed.
4. The diff has been reviewed.
5. The user approves the result.

## 11. Branching

- `main` is the default and integration branch.
- Feature branches are created only when a task needs isolation. They are merged into `main` after the user approves the result.
- Commits and merges to `main` require user approval.