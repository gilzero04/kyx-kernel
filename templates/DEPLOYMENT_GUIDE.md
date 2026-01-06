# DEPLOYMENT_GUIDE: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation

- **Target Audience**: DevOps and SRE.
- **Goal**: Define the Environment Contract.

## Summary & Prime Directive

**WHAT**: [Deployment process]
**WHY**: [Environment parity goals]
**HOW**: [Terraform, Docker, K8s]

## Analysis & Decisions

- **Infra Choice**: [Why we chose this provider/platform]
- **Decision Record**: [Environment variable strategy]

## Capability Traceability

| Role   | Mechanism | Config Path    |
| :----- | :-------- | :------------- |
| [Role] | [Tool]    | [.env/secrets] |

## Invariants & Failure Modes

- **Invariant**: [Static assets must be served]
- **Failure Mode**: [Downtime during update]
- **Prevention**: [Rolling updates, health checks]
