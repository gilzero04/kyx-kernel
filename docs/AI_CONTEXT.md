# AI_CONTEXT: kyx-kernel

project_id: kyx-kernel
author: Antigravity
created_by: ai
ai_prompt: "Establishing the AI reasoning layer for Kyx Kernel v3.1 - English Version"
ai_confidence: 1.0
last_updated: 2026-01-05

## 🧭 Reader Orientation

- **Target Audience**: AI Agents (Coding & Logic).
- **Purpose**: Defines the specific rules and reasoning process for AI Agents managing the core engine.

## Summary & Prime Directive (Rule 0)

**WHAT**: Context layer for AI Agents.
**WHY**: Prevent logic errors in the mission-critical core.
**HOW**: Strictly follow Super-Governance v3.1 and the Zero Warning Policy.

## Analysis & Decisions (Rule 4)

- **Deep Operational Rationale (Extensive)**:
  The AI Context Layer for Kyx Kernel v3.1 is the primary mechanism for preventing **Logic Desync** and **Cognitive Drift** in AI-driven development. During our analysis of Agent-Kernel interactions, we observed that agents frequently make "locally correct but globally dangerous" assumptions when network visibility is constrained (Rule 18). We have decided to mitigate this by enforcing a mandatory **Mental Schema Mapping** phase before any code edit. This ensures that the agent has a complete graph of the `SAD.md` and `TDD.md` in its memory, preventing fragments of logic from being implemented in isolation.

  Furthermore, we have codified the **Traceability Directive** (Rule 5) to require every proposal to be linked to a specific code item (e.g., `core::auth`, `core::iam`). This decision eliminates the ambiguity often found in AI-generated PRDs and ensures that the "Source Signature" is verifiable across the entire ecosystem audit trail. We also analyzed the frequency of "Silent Sync Failures" and decided to implement a mandatory **Hub-Synchronization Protocol** (Rule 19) using the `upsert-document` tool, which acts as a transactional gatekeeper for all documentation updates. By requiring a 150-word technical analysis (Rule 20) for every major task, we force the AI to explain the "Technical WHY" behind its code, turning the documentation into a living memory of system evolution. This level of professional-grade context ensures that the AI remains a reliable and governed extension of the Kyx development team, capable of maintaining the Kernel's high-precision Atomic Consistency.

## Capability Traceability (Rule 5)

| Capability    | Technical Mechanism | Infrastructure           | Source Signature |
| :------------ | :------------------ | :----------------------- | :--------------- |
| Logic Control | System Prompt       | core::ai::kernel         | AI_CONTEXT.md    |
| Safety Gate   | Linter Validation   | scripts/validate-docs.sh | Rule 3           |

## Invariants & Failure Modes (Rule 6)

- **Invariant**: Strictly no hardcoded secrets or sensitive keys in the Kernel (Rule 9).
- **Mode**: Unauthorized Access (Prevention: Mandatory middleware check on all routes/services).
- **Mode**: Logic Desync (Prevention: Hub-first documentation verification).

### 📝 Prompt Canonical (v3.1)

"Objective: Modify Kyx Kernel following v3.1 standards.
Requirements:

1. Prove Rule 9 (Zero Hardcode) compliance.
2. Minimum 150 words in PRD/SAD/TDD Analysis (Rule 20).
3. Traceability to specific modules (e.g., core::auth, core::iam).
4. Use 'upsert-document' for Hub synchronization (Rule 19)."

## ✅ Variation Checklist (Rule 20)

- [ ] **Kernel Domain Identified?** (e.g., Auth, IAM, Core-AI)
- [ ] **Zero Warning Compliance?** (Zero-lint-error policy)
- [ ] **Logical Rationale Included?** (150+ words of technical WHY)
- [ ] **Verification Proof?** (Does list-documents reflect your sync?)

## 🏁 Delivery Requirement (Rule 17)

- Changes MUST be on a new branch.
- Commit message MUST include the Rationale (WHY).
