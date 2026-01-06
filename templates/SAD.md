# SAD: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: System Engineers and DevOps.
- **Goal**: Define the System Contract.

## Summary & Prime Directive (Rule 0)

**WHAT**: [Logical architecture description]
**WHY**: [Architectural rationale/scalability goals]
**HOW**: [Containerization, DB choices, Transport protocols]

## Analysis & Decisions (Rule 4)

- **Architectural Style**: [Monolith, Microservices, Actor model, etc.]
- **Data Flow**: [How data moves through the system]
- **Decision Record**: [Rationale for major component choices]

## Capability Traceability (Rule 5)

| Module   | Responsibility   | Infrastructure | Source Signature      |
| :------- | :--------------- | :------------- | :-------------------- |
| [Module] | [Direct utility] | [Host/Service] | [File path/Namespace] |

## Architecture Components (Rule 8)

[Mermaid diagram or structured list of components]

## Invariants & Failure Modes (Rule 6)

- **Invariant**: [Persistence requirements, state consistency]
- **Failure Mode**: [Network partition, service crash]
- **Prevention**: [Retries, Circuit-breakers, Fallbacks]
