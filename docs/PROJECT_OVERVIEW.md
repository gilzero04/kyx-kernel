# PROJECT_OVERVIEW: kyx-kernel

project_id: kyx-kernel
author: Antigravity
created_by: ai
ai_prompt: "Establishing the source of authority overview for Kyx Kernel v3.1 - English Version"
ai_confidence: 0.99
last_updated: 2026-01-05

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: Everyone (Human & Agent).
- **Next Steps**: See `PRD.md` for the core engine contract.

## Summary & Prime Directive (Rule 0)

**WHAT**: kyx-kernel is the "Heart" (Core Engine) of the Kyx Ecosystem, responsible for managing low-level logic, authentication, tenant isolation, and core business workflows.
**WHY**: To provide a stable, secure, and scalable foundation for all other products (Separation of Concerns).
**HOW**: Developed using industry-standard technologies (Node.js/TypeScript or Rust) with a strictly enforced Zero Warning Policy.

- **Decision Record**: Adopt a high-concurrency engine model with polyglot persistence.

## 📂 Project Structure (Verified)

```text
.
├── src/
│   ├── core/           # Modular Engine Core (Auth, Middlewares, Registry)
│   ├── modules/        # Feature Modules (auth, media, system)
│   └── main.rs         # Server entry point
├── migrations/         # SQL State & Feature Migrations (0001-0016)
├── docs/               # Super-Governance v3.1 Standards
├── templates/          # SDLC templates
├── knowledge_base/     # Static metadata & knowledge scripts
├── scripts/            # Local validation (validate-docs.sh)
├── Cargo.toml          # Rust build manifest
└── docker-compose.yml  # Local orchestration
```

## Analysis & Decisions (Rule 4)

- **Deep Strategic Rationale (Extensive)**:
  The architectural foundation of Kyx Kernel v3.1 is predicated on the principle of **Stateless Sovereignty**. During our analysis of high-load scenarios within the Kyx ecosystem, we identified that horizontal scalability is frequently throttled by sticky sessions and local state dependencies. Consequently, we have enforced a "Zero-Local-State" mandate across all core modules. This decision enables the Kernel to be instantly containerized and scaled across geographically diverse infrastructure (Kyx Infra) without the risk of data inconsistency or session fragmentation.

  Regarding access control, we have standardized on an **Identity-first RBAC (Role-Based Access Control) Model**. This model isn't merely a permission set; it is a central standard that governs how AI agents and human operators interact with every ecosystem project. By implementing this at the Kernel level, we ensure a unified security posture where permissions are inherited and audit trails (Rule 16) are generated natively at the source. Furthermore, the integration of **Super-Governance v3.1** from the earliest design phases represents a shift toward "Executable Governance." This means that every kernel component is self-describing and audit-ready, allowing for automated compliance checks that prevent "Governance Decay" as the system complexity increases. This investment in structural precision ensures that the Kernel remains the stable and trusted "Heart" of the entire network.

## Capability Traceability (Rule 5)

| Capability       | Technical Mechanism | Infrastructure         | Source Signature |
| :--------------- | :------------------ | :--------------------- | :--------------- |
| Tenant Isolation | DB Filtering        | project_id: kyx-kernel | core::iam        |
| Authentication   | JWT / Session       | project_id: kyx-kernel | core::auth       |
| API Gateway      | Routing / Proxy     | project_id: kyx-kernel | api::gateway     |

## Invariants & Failure Modes (Rule 6)

- **Invariant**: Cross-tenant data must never leak (Zero Leakage).
- **Failure Mode**: Kernel Downtime (Impact: Entire ecosystem functionality ceases).
- **Prevention**: Utilize health check probes and automated restart policies.
