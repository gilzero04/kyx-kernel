# PRD: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: Stakeholders and Architects.
- **Goal**: Define the Business Contract.

## Summary & Prime Directive (Rule 0)

**WHAT**: [50-200 words summarizing the product goals]
**WHY**: [The business rationale and problem being solved]
**HOW**: [High-level technical approach]

## Analysis & Decisions (Rule 4)

- **Objective**: [The "North Star" for this requirement]
- **Assumptions**: [What must be true for this to work]
- **Alternatives & Trade-offs**: [Options considered and why they were rejected]
- **Decision Record**: [Final selection rationale]

## Capability Traceability (Rule 5)

| Capability | Technical Mechanism | Infrastructure   | Source Signature |
| :--------- | :------------------ | :--------------- | :--------------- |
| [Feature]  | [How it works]      | [Where it lives] | [Link to code]   |

## Invariants & Failure Modes (Rule 6)

- **Invariant**: [What must NEVER break]
- **Failure Mode**: [What happens if X fails]
- **Prevention**: [How we mitigate this failure]
