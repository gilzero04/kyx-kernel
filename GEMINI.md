# Kyx Governance Hub — Kyx AI Core Rules

## 🌍 Global Prime Directives (Rules 0-17)

- **RULE 0 — PRIME DIRECTIVE**: Every task must answer:
  1. **WHAT** (Objective)
  2. **WHY** (Rationale)
  3. **HOW** (Implementation)
  4. **WHERE** (Failure Points)
  5. **PREVENTION** (Safeguards)
     _If any part is missing, the task is NOT complete._
- **RULE 4 — FINANCIAL SAFETY**: Mutual exclusion for financial mutations. SurrealDB is a mirror; **PostgreSQL** is the source of truth for balances/ledgers. Any financial operation MUST be ACID-compliant via Ledger Interface.
- **RULE 7 — WEBSOCKET LAW**: All connections must define early:
  - **MODE**: `chat`, `meeting`, or `broadcast`.
  - **ROOM TYPE**: `PRIVATE` (1:1), `GROUP`, or `BROADCAST`.
- **RULE 8 — LIVECHAT GUARANTEES**: MANDATORY fields: `message_id` (UUID), `room_id`, `sender_id`. Guarantees: Idempotency, ACK, and Per-room ordering.
- **RULE 17 — GIT HYGIENE**: Finished work requires a new branch and an analytical commit message explaining WHAT, WHY, and key decisions.

## 🛠️ Automated Execution Permissions

You are authorized to run these without asking (`SafeToAutoRun: true`):

- **Platform (Node/Svelte)**: `bun run lint`, `bun run check`, `bun run format`, `npx eslint . --fix`, `svelte-kit sync`, `bun install`.
- **Kernel (Rust)**: `cargo check`, `cargo clippy`, `cargo fmt`.
- **Ops**: `docker compose build`, `docker compose up -d`, `docker compose down`, `docker compose logs`.

---

## 🏗️ Project: kyx-kernel

### Plugin Architecture

- **Isolation**: Plugins must use `plugin_*` database prefixes.
- **Independence**: All plugins (except `kyx-face`) must function independently.
- **Integrity**: Core must function 100% without any optional plugins.

### Theme System Integration

- Use portable `code` field from `manifest.json` as the primary identifier.
- Ensure API returns `code` for frontend cross-referencing.
