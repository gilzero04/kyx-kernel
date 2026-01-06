# API Specification: Kyx Kernel

project_id: kyx-kernel
author: Antigravity
created_by: ai
ai_prompt: "Establishing the high-fidelity API reference for Kyx Kernel v3.1"
ai_confidence: 1.0
last_updated: 2026-01-05

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: Developers, AI Agents, Integrators.
- **Goal**: Definitive list of all available HTTP endpoints.
- **Next Steps**: See `IMPLEMENTATION_SUMMARY.md` for study guides.

## Summary & Prime Directive (Rule 0)

**WHAT**: Comprehensive API reference for the Kernel engine.
**WHY**: To enable decoupled development and system integration.
**HOW**: Based on the traits and routers implemented in `src/modules/`.

## Analysis & Decisions (Rule 4)

- **Deep API Design Rationale (Extensive)**:
  The API surface of Kyx Kernel v3.1 is designed to be the "Standard Interface of Authority" for the entire Kyx network. During the interface design phase, we analyzed the risks of "API Bloat" and decided to implement a **Strict Logic Gating Strategy**. This means that almost all endpoints (especially those in Auth and Admin modules) are protected by a mandatory JWT-based middleware that enforces multi-tenant context at the entry point. This decision ensures that no data leakage can occur between tenants, as the `tenant_id` is extracted from the secure token and injected into the request flow before reaching any business module.

  Furthermore, we have standardized on **RESTful JSON-RPC Hybrid Patterns**. While we follow REST conventions for entity management (e.g., `/media`), we use more explicit, RPC-like naming for complex system actions (e.g., `/admin/context`) to improve clarity for AI agents during tool-calling sequences. This semantic precision reduces the chance of agent-induced errors when interacting with the Kernel. We also analyzed the requirements for "Super-Governance" and decided to include a dedicated `/admin/context` endpoint. This allows AI agents to retrieve the absolute latest governance rules and architecture blueprints in real-time, ensuring their code generation is always aligned with v3.1 standards. By mandating explicit error schemas in the "Usage Handbook," we provide the necessary diagnostic data for self-healing logic, turning the API into a resilient and professional-grade bridge for the ecosystem.

## 🔐 Auth & Identity Modules

| Endpoint                | Method | Security | Description                               |
| :---------------------- | :----- | :------- | :---------------------------------------- |
| `/auth/login`           | POST   | Public   | Authenticate and retrieve JWT / Session.  |
| `/auth/refresh`         | POST   | Public   | Refresh an existing session.              |
| `/auth/logout`          | POST   | Auth     | Revoke current session.                   |
| `/auth/signup`          | POST   | Public   | Register a new user account.              |
| `/auth/users`           | POST   | Admin    | Create a new user (Privileged).           |
| `/users/me/preferences` | GET    | Auth     | Get current user's personalized settings. |

## ⚙️ System & Governance Modules

| Endpoint          | Method | Security | Description                                  |
| :---------------- | :----- | :------- | :------------------------------------------- |
| `/system/status`  | GET    | Public   | Cluster health and engine versioning.        |
| `/admin/settings` | GET    | Admin    | Retrieve global engine configurations.       |
| `/admin/context`  | GET    | Admin    | Get current AI context and governance rules. |
| `/admin/logs`     | GET    | Admin    | Access the global audit trail.               |
| `/i18n/locales`   | GET    | Public   | List supported languages and locales.        |
| `/i18n/{locale}`  | GET    | Public   | Fetch translations for a specific locale.    |

## 📁 Media & Assets Modules

| Endpoint                        | Method | Security | Description                               |
| :------------------------------ | :----- | :------- | :---------------------------------------- |
| `/media`                        | POST   | Auth     | Upload an asset to the persistence layer. |
| `/media`                        | GET    | Auth     | List/Search available assets.             |
| `/media/{id}`                   | DELETE | Auth     | Remove an asset from the system.          |
| `/public/system/tenants/{slug}` | GET    | Public   | Fetch tenant-specific public data.        |

---

## 🛠️ Usage Handbook (How to use)

### 1. Authentication Flow

To use protected routes, you must first authenticate:

```bash
curl -X POST http://localhost:8000/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "admin", "password": "..."}'
```

The response will contain a `token`. Use it in the `Authorization` header:
`Authorization: Bearer <your_token>`

### 2. Error Schemas

All errors follow the `AppError` trait (src/core/mod.rs):

```json
{
  "code": 401,
  "message": "Unauthorized"
}
```

## Capability Traceability (Rule 5)

| Subsystem  | Traceability ID  | Handler Location   |
| :--------- | :--------------- | :----------------- |
| Auth API   | core::auth::api  | src/modules/auth   |
| System API | core::sys::api   | src/modules/system |
| Media API  | core::media::api | src/modules/media  |

## Invariants & Failure Modes (Rule 6)

- **Invariant**: Every API response must include a `code` and `message` on error.
- **Failure Mode**: Unauthorized Access (Impact: Data Breach).
- **Prevention**: Mandatory JWT validation in `auth_middleware.rs`.
