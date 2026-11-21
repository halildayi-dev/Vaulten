# Vaulten — Claude Code Instructions

## Project Overview

Vaulten is a local-first, fully offline desktop password manager.

**Target:** Windows 10/11 x64.

**Planned stack:** Tauri 2, Rust, React, TypeScript, Vite, Tailwind CSS, SQLite.

## Core Principles

- Offline-first: no cloud, accounts, telemetry, or external network dependencies.
- Security-sensitive operations belong in Rust, not the frontend.
- Keep Tauri IPC explicit and minimal.
- Never store secrets in plaintext or expose them through logs.
- Use established cryptographic libraries. Never invent cryptographic algorithms.
- Keep the architecture modular and dependencies explicit.

## Development Rules

- Implement only the explicitly assigned task.
- Do not make unrelated changes or introduce unrequested features.
- Do not modify architecture or security decisions without approval.
- Do not delete, rename, or reorganize existing files without approval.
- Do not add dependencies unless required by the task.
- If requirements are ambiguous or contradictory, stop and ask.
- Keep documentation concise and avoid duplication.

## Testing and Verification

- Run only the checks relevant to the current task.
- Never claim a test passed unless it was actually executed.
- Report failed checks and unresolved issues.
- Security-critical changes require focused tests and additional review.

## Workflow

- Follow the project workflow documented in `docs/workflow.md` once established.
- Use `docs/architecture.md` as the architecture reference once established.
- Work only on tasks explicitly approved in `docs/backlog.md`.
- Do not use Spec Kit workflows to generate or execute tasks.
- Do not start parallel tasks or use additional agents unless explicitly requested.

## Completion Report

For every task, report:

1. Summary of changes.
2. Modified files.
3. Tests and verification results.
4. Known issues or risks.
5. Whether all acceptance criteria were met.

A task is not complete until its acceptance criteria pass and the user approves the result.

## Language Policy

- Project documentation, source code, comments, identifiers, and file names must be written in English.
- User instructions and conversations may be in Turkish.
- Respond to the user in Turkish unless explicitly requested otherwise.
- Task reports, explanations, and review findings must be written in Turkish.
- Do not translate or rewrite existing English documentation unless explicitly requested.